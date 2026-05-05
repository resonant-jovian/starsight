//! Minimal crates.io API client — three endpoints, blocking, mandatory User-Agent.
//!
//! - `GET /api/v1/crates/{crate}` — version, license, MSRV, updated_at
//! - `GET /api/v1/crates/{crate}/downloads` — full daily download series
//! - `GET /api/v1/crates/{crate}/reverse_dependencies` — dependents count
//!
//! Failure mode: any network or parse error returns `Err`; callers decide whether
//! to fall back to cached / static data or abort.

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;

const API: &str = "https://crates.io/api/v1/crates/starsight";
const UA: &str =
    "starsight-chrome-regen (https://github.com/resonant-jovian/starsight; albin@sjoegren.se)";

pub struct Stats {
    pub version: String,
    pub edition: String,
    pub msrv: String,
    pub license: String,
    /// Last 30 calendar days, oldest → newest. Days before first publish are 0.
    pub downloads_30d: [u32; 30],
    pub downloads_30d_total: u32,
    pub downloads_lifetime: u64,
    pub dependents: u32,
    /// Days since `updated_at`, integer.
    pub updated_days_ago: u32,
    /// First-publish date, formatted "YYYY-MM-DD".
    pub first_publish: String,
}

pub fn fetch() -> Result<Stats> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // /crates/starsight
    let crate_info: CrateEnvelope = client
        .get(API)
        .send()?
        .error_for_status()?
        .json()
        .context("parse /crates/starsight")?;

    // /crates/starsight/downloads
    let downloads: DownloadsEnvelope = client
        .get(format!("{API}/downloads"))
        .send()?
        .error_for_status()?
        .json()
        .context("parse /crates/starsight/downloads")?;

    // /crates/starsight/reverse_dependencies
    let revdeps: RevDepsEnvelope = client
        .get(format!("{API}/reverse_dependencies?per_page=1"))
        .send()?
        .error_for_status()?
        .json()
        .context("parse reverse_dependencies")?;

    let krate = &crate_info.krate;
    let v_meta = crate_info
        .versions
        .first()
        .ok_or_else(|| anyhow!("no versions in crate response"))?;

    let downloads_30d = downsample_30(&downloads.version_downloads, &downloads.meta);
    let downloads_30d_total: u32 = downloads_30d.iter().sum();

    let updated_days_ago = days_since(&krate.updated_at)?;

    Ok(Stats {
        version: krate.max_stable_version.clone().unwrap_or_else(|| krate.max_version.clone()),
        edition: v_meta.edition.clone().unwrap_or_else(|| "2024".into()),
        msrv: v_meta
            .rust_version
            .clone()
            .unwrap_or_else(|| "1.89".into()),
        license: v_meta.license.clone().unwrap_or_else(|| "GPL-3.0-only".into()),
        downloads_30d,
        downloads_30d_total,
        downloads_lifetime: krate.downloads,
        dependents: revdeps.meta.total,
        updated_days_ago,
        first_publish: krate.created_at.split('T').next().unwrap_or("").to_string(),
    })
}

#[derive(Deserialize)]
struct CrateEnvelope {
    #[serde(rename = "crate")]
    krate: CrateInfo,
    versions: Vec<VersionInfo>,
}
#[derive(Deserialize)]
struct CrateInfo {
    max_version: String,
    max_stable_version: Option<String>,
    updated_at: String,
    created_at: String,
    downloads: u64,
}
#[derive(Deserialize)]
struct VersionInfo {
    license: Option<String>,
    rust_version: Option<String>,
    edition: Option<String>,
}
#[derive(Deserialize)]
struct DownloadsEnvelope {
    version_downloads: Vec<DayPoint>,
    meta: DownloadsMeta,
}
#[derive(Deserialize)]
struct DownloadsMeta {
    extra_downloads: Vec<DayPoint>,
}
#[derive(Deserialize, Clone)]
struct DayPoint {
    date: String,
    downloads: u32,
}
#[derive(Deserialize)]
struct RevDepsEnvelope {
    meta: RevDepsMeta,
}
#[derive(Deserialize)]
struct RevDepsMeta {
    total: u32,
}

/// Fold both per-version and `extra_downloads` (pre-1.0 endpoint quirk) into a
/// single 30-day window indexed oldest→newest.
fn downsample_30(versioned: &[DayPoint], meta: &DownloadsMeta) -> [u32; 30] {
    use std::collections::BTreeMap;
    let mut by_day: BTreeMap<&str, u32> = BTreeMap::new();
    for d in versioned.iter().chain(meta.extra_downloads.iter()) {
        *by_day.entry(d.date.as_str()).or_insert(0) += d.downloads;
    }
    let mut out = [0u32; 30];
    // The API returns up to 90 days; take the most recent 30 (newest → oldest order).
    let recent: Vec<u32> = by_day.values().rev().take(30).copied().collect();
    let n = recent.len();
    // Right-align: newest always lands at out[29], oldest available at out[30 - n].
    // If fewer than 30 days exist, the leading slots stay 0 (no data yet).
    for (i, v) in recent.into_iter().rev().enumerate() {
        out[30 - n + i] = v;
    }
    out
}

/// Best-effort day diff from an ISO-8601 timestamp ("YYYY-MM-DDTHH:MM:SS…").
fn days_since(iso: &str) -> Result<u32> {
    let date = iso.split('T').next().unwrap_or("");
    let mut parts = date.split('-');
    let y: i64 = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let m: i64 = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let d: i64 = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let then = ymd_to_jdn(y, m, d);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        / 86_400
        + 2_440_588; // unix epoch JDN
    Ok((now - then).max(0) as u32)
}

/// Julian day number for a Gregorian date (Fliegel & Van Flandern).
fn ymd_to_jdn(y: i64, m: i64, d: i64) -> i64 {
    let a = (14 - m) / 12;
    let y2 = y + 4800 - a;
    let m2 = m + 12 * a - 3;
    d + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32_045
}
