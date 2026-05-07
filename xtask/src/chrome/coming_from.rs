//! `assets/coming-from-{light,dark}.svg` — concrete syntax mappings from 15
//! source libraries / languages to starsight idioms.
//!
//! Per-library card stack. Each card has a head row (mantle bg, library name +
//! language tag + row count) and body rows of `from-code → to-code`. SVG
//! height grows with content; the result is a tall single image users can
//! zoom and scroll. Mono throughout.

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

const W: u32 = 920;
const PAD: u32 = 24;
const RADIUS: f32 = 12.0;

const CARD_HEAD_H: u32 = 36;
const CARD_GAP: u32 = 12;
const EYEBROW_H: u32 = 24;

const CELL_PAD_X: u32 = 12;
const FROM_W: u32 = 320;
const ARROW_W: u32 = 24;
const FONT_PX: f32 = 11.0;
const LINE_H: f32 = 16.0;
const ROW_VPAD: f32 = 6.0;
/// Conservative monospace char width at 11pt across the system fallback chain.
const MONO_CHAR_W: f32 = FONT_PX * 0.62;

struct Group<'a> {
    lib: &'a str,
    lang: &'a str,
    rows: &'a [(&'a str, &'a str)],
}

const GROUPS: &[Group<'_>] = &[
    Group {
        lib: "matplotlib",
        lang: "python",
        rows: &[
            ("plt.plot(x, y)", "plot!(x, y)"),
            (
                "plt.scatter(x, y, c=c)",
                "PointMark::new(x, y).color_by(&c)",
            ),
            ("plt.bar(labels, vals)", "BarMark::new(labels, vals)"),
            (
                "plt.barh(labels, vals)",
                "BarMark::new(labels, vals).horizontal()",
            ),
            (
                "plt.hist(data, bins=30)",
                "HistogramMark::new(data).method(BinMethod::Count(30))",
            ),
            (
                "plt.boxplot([a, b])",
                r#"BoxPlotMark::new(vec![BoxPlotGroup::new("a", a), …])"#,
            ),
            (
                "plt.pie(values, labels=…)",
                "PieMark::new(values, labels).show_percent()",
            ),
            ("plt.imshow(matrix)", "HeatmapMark::new(matrix)"),
            (
                "plt.fill_between(x, y1, y2)",
                "AreaMark::new(x, y1).baseline(y2)",
            ),
            (
                r#"plt.xlabel("…"); plt.ylabel("…")"#,
                r#"Figure::new().x_label("…").y_label("…")"#,
            ),
            (r#"plt.title("…")"#, r#".title("…")"#),
            (r#"plt.savefig("out.png")"#, r#".save("out.png")?"#),
            ("plt.show()", ".show()?  // feature `interactive`"),
            ("plt.subplots(2, 2)", "MultiPanelFigure::new(W, H, 2, 2)"),
        ],
    },
    Group {
        lib: "seaborn",
        lang: "python",
        rows: &[
            (
                "sns.violinplot(data=df, x, y)",
                "ViolinMark::new(groups).bandwidth(Bandwidth::Silverman)",
            ),
            ("sns.boxplot(data=df, x, y)", "BoxPlotMark::new(groups)"),
            (
                "sns.heatmap(data)",
                "HeatmapMark::new(data)  // prismatica colormaps",
            ),
            (
                "sns.kdeplot(data)",
                "Kde::new(Bandwidth::Silverman, Kernel::Gaussian).evaluate_grid(&grid, &data)",
            ),
            (
                r#"sns.set_theme(style="darkgrid")"#,
                ".theme(theme)  // chromata",
            ),
        ],
    },
    Group {
        lib: "plotly.express",
        lang: "python",
        rows: &[
            (
                r#"px.scatter(df, x="a", y="b")"#,
                r#"plot!(df, x="a", y="b")  // feature `polars`"#,
            ),
            ("px.line(df, x, y, color)", r#"plot!(df, x, y, color="g")"#),
            (
                "px.bar(df, x, y, color)",
                r#"plot!(df, x, y, color="g", kind="bar")"#,
            ),
            ("px.histogram(df, x)", r#"plot!(df, x, kind="hist")"#),
        ],
    },
    Group {
        lib: "mpl_finance",
        lang: "python",
        rows: &[(
            "candlestick_ohlc(ax, quotes)",
            "CandlestickMark::new(vec![Ohlc { … }, …])",
        )],
    },
    Group {
        lib: "plotnine",
        lang: "python",
        rows: &[
            (
                "ggplot(df) + aes(x, y) + geom_point()",
                "plot!(df, x, y) + builder chain",
            ),
            ("+ stat_smooth()", ".add(RegressionMark::new(...))  // 0.5"),
        ],
    },
    Group {
        lib: "bokeh",
        lang: "python",
        rows: &[
            (
                "p = figure(); p.line(x, y)",
                "Figure::new(W, H).add(LineMark::new(x, y))",
            ),
            (
                "p.circle(x, y, size=8)",
                ".add(PointMark::new(x, y).radius(4.0))",
            ),
            (r#"output_file("o.html")"#, r#".save("o.html")  // 0.10"#),
        ],
    },
    Group {
        lib: "altair / vega-lite",
        lang: "python · json",
        rows: &[
            ("alt.Chart(df).mark_point()", "plot!(df, x, y)"),
            (r#".encode(x, y, color="g")"#, "plot! macro: color column"),
            (r#".facet(row="…")"#, r#".facet_grid(rows="…")  // 0.4"#),
        ],
    },
    Group {
        lib: "ggplot2",
        lang: "r",
        rows: &[
            (
                "ggplot(df) + geom_point()",
                "Figure::new(W, H).add(PointMark::from(df))",
            ),
            ("+ geom_line()", ".add(LineMark::from(df))"),
            (r#"+ geom_bar(stat="identity")"#, ".add(BarMark::from(df))"),
            ("+ geom_violin()", ".add(ViolinMark::from(df))"),
            ("+ facet_wrap(~ group)", r#".facet_wrap("group")  // 0.4"#),
            ("+ scale_y_log10()", ".y_scale(LogScale)  // 0.5"),
            (
                "+ theme_minimal()",
                ".theme(rose_pine_dawn.into())  // chromata",
            ),
        ],
    },
    Group {
        lib: "observable plot",
        lang: "js",
        rows: &[
            (
                "Plot.dot(data, {x, y, fill})",
                "PointMark::new(x, y).color_by(&fill)",
            ),
            ("Plot.line(data, {x, y})", "LineMark::new(x, y)"),
            ("Plot.barY(data, {x, y})", "BarMark::new(x, y)"),
            ("Plot.rectY(bin(…))", "HistogramMark::new(data)"),
        ],
    },
    Group {
        lib: "d3.js",
        lang: "js",
        rows: &[
            (
                "d3.scaleLinear().domain().range()",
                "LinearScale::new(domain, range)",
            ),
            ("d3.axisBottom(scale)", "Axis::bottom(scale)"),
            ("d3.line()(data)", "LineMark::new(x, y)  // pre-rendered"),
        ],
    },
    Group {
        lib: "matlab",
        lang: "matlab",
        rows: &[
            ("plot(x, y)", "plot!(x, y)"),
            ("scatter(x, y)", "PointMark::new(x, y)"),
            ("surf(X, Y, Z)", "Surface3D::new(X, Y, Z)  // 0.9"),
            ("imagesc(M)", "HeatmapMark::new(M)"),
            (r#"saveas(gcf, "out.png")"#, r#".save("out.png")?"#),
        ],
    },
    Group {
        lib: "gnuplot",
        lang: "gnuplot",
        rows: &[
            (
                r#"plot "data.dat" with lines"#,
                r#"LineMark::from_csv("data.dat")"#,
            ),
            ("set logscale y", ".y_scale(LogScale)  // 0.5"),
            ("set terminal png", r#".save("out.png")?"#),
        ],
    },
    Group {
        lib: "plotters",
        lang: "rust",
        rows: &[
            (
                "ChartBuilder::on(&root).build_cartesian_2d",
                "Figure::new(800, 600)",
            ),
            (
                "chart.draw_series(LineSeries::new(…))",
                ".add(LineMark::new(x, y))",
            ),
            (
                "BitMapBackend / SVGBackend",
                r#".save("out.png") / .save("out.svg")"#,
            ),
        ],
    },
    Group {
        lib: "charming",
        lang: "rust",
        rows: &[
            (
                "Chart::new().x_axis(…).y_axis(…)",
                "Figure::new(W, H).add(LineMark::new(x, y))",
            ),
            (
                ".series(Line::new().data(data))",
                ".add(LineMark::new(x, y))",
            ),
        ],
    },
    Group {
        lib: "plotly.rs",
        lang: "rust",
        rows: &[
            (
                "Plot::new() + Scatter::new(x, y)",
                "Figure::new(W, H).add(PointMark::new(x, y))",
            ),
            (
                r#"plot.write_html("o.html")"#,
                r#".save("o.html")  // 0.10"#,
            ),
        ],
    },
];

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let svg = render(theme);
    let out = root.join(format!("assets/coming-from-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render(theme: Theme) -> String {
    let p = palette(theme);

    let total_rows: usize = GROUPS.iter().map(|g| g.rows.len()).sum();

    let card_w = W - 2 * PAD;
    let from_inner = (FROM_W - 2 * CELL_PAD_X) as f32;
    let to_inner = (card_w - FROM_W - ARROW_W - 2 * CELL_PAD_X) as f32;

    // Pre-wrap every row so each card knows its own height.
    let wrapped: Vec<Vec<(Vec<String>, Vec<String>)>> = GROUPS
        .iter()
        .map(|g| {
            g.rows
                .iter()
                .map(|(from, to)| {
                    let from_lines = wrap(from, from_inner);
                    let to_lines = wrap(to, to_inner);
                    (from_lines, to_lines)
                })
                .collect()
        })
        .collect();

    // Card heights = head + sum of (max(from_lines, to_lines) * LINE_H + 2 * VPAD).
    let card_heights: Vec<u32> = wrapped
        .iter()
        .map(|rows| {
            let body: f32 = rows
                .iter()
                .map(|(f, t)| (f.len().max(t.len()) as f32) * LINE_H + 2.0 * ROW_VPAD)
                .sum();
            CARD_HEAD_H + body.ceil() as u32 + 2
        })
        .collect();

    let content_h: u32 = card_heights.iter().sum::<u32>() + CARD_GAP * (GROUPS.len() as u32 - 1);
    let h = PAD + EYEBROW_H + content_h + PAD;

    let mut out = header(
        W,
        h,
        "starsight syntax cheatsheet — matplotlib, seaborn, ggplot2, plotly, d3, matlab, gnuplot, plotters, …",
        "starsight syntax cheatsheet",
    );

    // Outer rounded card.
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{hh}" rx="{r}" fill="{bg}" stroke="{s}" stroke-width="1"/>
"#,
        w = W - 1,
        hh = h - 1,
        r = RADIUS,
        bg = p.bg,
        s = p.border,
    ));

    // Eyebrow.
    let eyebrow = format!(
        "// coming from another language · {} mappings across {} libraries",
        total_rows,
        GROUPS.len(),
    );
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="11" fill="{c}">{txt}</text>
"#,
        x = PAD,
        y = PAD + 16,
        f = MONO_FAMILY,
        c = p.subtext,
        txt = escape(&eyebrow),
    ));

    // Card stack.
    let mut card_y: u32 = PAD + EYEBROW_H + 4;
    for (gi, g) in GROUPS.iter().enumerate() {
        let card_h = card_heights[gi];
        out.push_str(&render_card(p, card_y, card_h, g, &wrapped[gi]));
        card_y += card_h + CARD_GAP;
    }

    out.push_str("</svg>\n");
    out
}

fn render_card(
    p: &super::palette::Palette,
    y: u32,
    h: u32,
    g: &Group,
    wrapped_rows: &[(Vec<String>, Vec<String>)],
) -> String {
    let mut s = String::new();
    let card_x = PAD;
    let card_w = W - 2 * PAD;

    // Card outline.
    s.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{h}" rx="6" fill="{bg}" stroke="{c}" stroke-width="1"/>
"#,
        x = card_x,
        w = card_w,
        bg = p.bg,
        c = p.border,
    ));

    // Head row (slightly different bg).
    s.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{hh}" fill="{bg}"/>
"#,
        x = card_x,
        w = card_w,
        hh = CARD_HEAD_H,
        bg = p.card,
    ));
    s.push_str(&format!(
        r#"  <line x1="{x1}" y1="{ly}" x2="{x2}" y2="{ly}" stroke="{c}" stroke-width="1"/>
"#,
        x1 = card_x,
        x2 = card_x + card_w,
        ly = y + CARD_HEAD_H,
        c = p.border,
    ));

    // Library name (bold sans for legibility) + language tag (mono, uppercase).
    s.push_str(&format!(
        r#"  <text x="{x}" y="{ty}" font-family="{f}" font-weight="700" font-size="14" fill="{c}">{lib}</text>
"#,
        x = card_x + CELL_PAD_X,
        ty = y + 24,
        f = SANS,
        c = p.text,
        lib = escape(g.lib),
    ));
    s.push_str(&format!(
        r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="11" fill="{c}" letter-spacing="0.6">{lang}</text>
"#,
        x = card_x + CELL_PAD_X + (g.lib.chars().count() as u32) * 9 + 18,
        ty = y + 24,
        f = MONO_FAMILY,
        c = p.muted,
        lang = escape(&g.lang.to_uppercase()),
    ));
    s.push_str(&format!(
        r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="11" fill="{c}" text-anchor="end">{n} mapping{plural}</text>
"#,
        x = card_x + card_w - CELL_PAD_X,
        ty = y + 24,
        f = MONO_FAMILY,
        c = p.muted,
        n = g.rows.len(),
        plural = if g.rows.len() == 1 { "" } else { "s" },
    ));

    // Body rows — variable height per row.
    let mut row_y: f32 = (y + CARD_HEAD_H) as f32;
    for (i, (from_lines, to_lines)) in wrapped_rows.iter().enumerate() {
        let lines = from_lines.len().max(to_lines.len()).max(1);
        let row_h = (lines as f32) * LINE_H + 2.0 * ROW_VPAD;

        if i % 2 == 1 {
            s.push_str(&format!(
                r#"  <rect x="{x}" y="{ry:.1}" width="{w}" height="{rh:.1}" fill="{c}"/>
"#,
                x = card_x + 1,
                ry = row_y,
                w = card_w - 2,
                rh = row_h,
                c = p.card,
            ));
        }

        let first_baseline = row_y + ROW_VPAD + LINE_H - 4.0;

        // From cell (mono, subtext, possibly multi-line via tspans).
        let from_x = card_x + CELL_PAD_X;
        s.push_str(&format!(
            r#"  <text x="{x}" y="{ty:.1}" font-family="{f}" font-size="11" fill="{c}">{tspans}</text>
"#,
            x = from_x,
            ty = first_baseline,
            f = MONO_FAMILY,
            c = p.subtext,
            tspans = build_tspans(from_lines, from_x),
        ));

        // Arrow (single-line, vertically centred on the row).
        let arrow_y = row_y + row_h / 2.0 + 4.0;
        s.push_str(&format!(
            r#"  <text x="{x}" y="{ty:.1}" font-family="{f}" font-size="12" fill="{c}" text-anchor="middle">→</text>
"#,
            x = card_x + CELL_PAD_X + FROM_W + ARROW_W / 2,
            ty = arrow_y,
            f = MONO_FAMILY,
            c = p.muted,
        ));

        // To cell.
        let to_x = card_x + CELL_PAD_X + FROM_W + ARROW_W;
        s.push_str(&format!(
            r#"  <text x="{x}" y="{ty:.1}" font-family="{f}" font-size="11" fill="{c}">{tspans}</text>
"#,
            x = to_x,
            ty = first_baseline,
            f = MONO_FAMILY,
            c = p.text,
            tspans = build_tspans(to_lines, to_x),
        ));

        row_y += row_h;
    }
    let _ = h; // not used now that rows are variable-height
    s
}

/// Build the inner `<tspan>` chain for a multi-line cell. The first line is
/// emitted as the `<text>` content directly; subsequent lines use
/// `<tspan x dy="LINE_H">…</tspan>`.
fn build_tspans(lines: &[String], x: u32) -> String {
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            out.push_str(&escape(line));
        } else {
            out.push_str(&format!(
                r#"<tspan x="{x}" dy="{LINE_H}">{txt}</tspan>"#,
                txt = escape(line),
            ));
        }
    }
    out
}

/// Greedy word-wrap to fit `max_width_px`. Wraps at spaces; words that exceed
/// the column on their own get their own line and may overflow slightly. For
/// our content (matplotlib calls, Rust idioms) wrapping at the closest space
/// produces clean breaks.
fn wrap(s: &str, max_width_px: f32) -> Vec<String> {
    if max_width_px <= 0.0 {
        return vec![s.to_string()];
    }
    let max_chars = (max_width_px / MONO_CHAR_W).floor().max(1.0) as usize;

    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in s.split(' ') {
        if cur.is_empty() {
            cur.push_str(word);
        } else if cur.chars().count() + 1 + word.chars().count() <= max_chars {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur.push_str(word);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
