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
const CARD_ROW_H: u32 = 26;
const CARD_GAP: u32 = 12;
const EYEBROW_H: u32 = 24;

const CELL_PAD_X: u32 = 12;
const FROM_W: u32 = 360;
const ARROW_W: u32 = 28;

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
            ("plt.scatter(x, y, c=c)", "PointMark::new(x, y).color_by(&c)"),
            ("plt.bar(labels, vals)", "BarMark::new(labels, vals)"),
            ("plt.barh(labels, vals)", "BarMark::new(labels, vals).horizontal()"),
            ("plt.hist(data, bins=30)", "HistogramMark::new(data).method(BinMethod::Count(30))"),
            ("plt.boxplot([a, b])", r#"BoxPlotMark::new(vec![BoxPlotGroup::new("a", a), …])"#),
            ("plt.pie(values, labels=…)", "PieMark::new(values, labels).show_percent()"),
            ("plt.imshow(matrix)", "HeatmapMark::new(matrix)"),
            ("plt.fill_between(x, y1, y2)", "AreaMark::new(x, y1).baseline(y2)"),
            (r#"plt.xlabel("…"); plt.ylabel("…")"#, r#"Figure::new().x_label("…").y_label("…")"#),
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
            ("sns.violinplot(data=df, x, y)", "ViolinMark::new(groups).bandwidth(Bandwidth::Silverman)"),
            ("sns.boxplot(data=df, x, y)", "BoxPlotMark::new(groups)"),
            ("sns.heatmap(data)", "HeatmapMark::new(data)  // prismatica colormaps"),
            ("sns.kdeplot(data)", "Kde::new(Bandwidth::Silverman, Kernel::Gaussian).evaluate_grid(&grid, &data)"),
            (r#"sns.set_theme(style="darkgrid")"#, ".theme(theme)  // chromata"),
        ],
    },
    Group {
        lib: "plotly.express",
        lang: "python",
        rows: &[
            (r#"px.scatter(df, x="a", y="b")"#, r#"plot!(df, x="a", y="b")  // feature `polars`"#),
            ("px.line(df, x, y, color)", r#"plot!(df, x, y, color="g")"#),
            ("px.bar(df, x, y, color)", r#"plot!(df, x, y, color="g", kind="bar")"#),
            ("px.histogram(df, x)", r#"plot!(df, x, kind="hist")"#),
        ],
    },
    Group {
        lib: "mpl_finance",
        lang: "python",
        rows: &[
            ("candlestick_ohlc(ax, quotes)", "CandlestickMark::new(vec![Ohlc { … }, …])"),
        ],
    },
    Group {
        lib: "plotnine",
        lang: "python",
        rows: &[
            ("ggplot(df) + aes(x, y) + geom_point()", "plot!(df, x, y) + builder chain"),
            ("+ stat_smooth()", ".add(RegressionMark::new(...))  // 0.5"),
        ],
    },
    Group {
        lib: "bokeh",
        lang: "python",
        rows: &[
            ("p = figure(); p.line(x, y)", "Figure::new(W, H).add(LineMark::new(x, y))"),
            ("p.circle(x, y, size=8)", ".add(PointMark::new(x, y).radius(4.0))"),
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
            ("ggplot(df) + geom_point()", "Figure::new(W, H).add(PointMark::from(df))"),
            ("+ geom_line()", ".add(LineMark::from(df))"),
            (r#"+ geom_bar(stat="identity")"#, ".add(BarMark::from(df))"),
            ("+ geom_violin()", ".add(ViolinMark::from(df))"),
            ("+ facet_wrap(~ group)", r#".facet_wrap("group")  // 0.4"#),
            ("+ scale_y_log10()", ".y_scale(LogScale)  // 0.5"),
            ("+ theme_minimal()", ".theme(rose_pine_dawn.into())  // chromata"),
        ],
    },
    Group {
        lib: "observable plot",
        lang: "js",
        rows: &[
            ("Plot.dot(data, {x, y, fill})", "PointMark::new(x, y).color_by(&fill)"),
            ("Plot.line(data, {x, y})", "LineMark::new(x, y)"),
            ("Plot.barY(data, {x, y})", "BarMark::new(x, y)"),
            ("Plot.rectY(bin(…))", "HistogramMark::new(data)"),
        ],
    },
    Group {
        lib: "d3.js",
        lang: "js",
        rows: &[
            ("d3.scaleLinear().domain().range()", "LinearScale::new(domain, range)"),
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
            (r#"plot "data.dat" with lines"#, r#"LineMark::from_csv("data.dat")"#),
            ("set logscale y", ".y_scale(LogScale)  // 0.5"),
            ("set terminal png", r#".save("out.png")?"#),
        ],
    },
    Group {
        lib: "plotters",
        lang: "rust",
        rows: &[
            ("ChartBuilder::on(&root).build_cartesian_2d", "Figure::new(800, 600)"),
            ("chart.draw_series(LineSeries::new(…))", ".add(LineMark::new(x, y))"),
            ("BitMapBackend / SVGBackend", r#".save("out.png") / .save("out.svg")"#),
        ],
    },
    Group {
        lib: "charming",
        lang: "rust",
        rows: &[
            ("Chart::new().x_axis(…).y_axis(…)", "Figure::new(W, H).add(LineMark::new(x, y))"),
            (".series(Line::new().data(data))", ".add(LineMark::new(x, y))"),
        ],
    },
    Group {
        lib: "plotly.rs",
        lang: "rust",
        rows: &[
            ("Plot::new() + Scatter::new(x, y)", "Figure::new(W, H).add(PointMark::new(x, y))"),
            (r#"plot.write_html("o.html")"#, r#".save("o.html")  // 0.10"#),
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

    // Total height = pad + eyebrow + sum(card heights + gap) - last gap + pad.
    let mut content_h: u32 = 0;
    for g in GROUPS {
        let card_h = CARD_HEAD_H + CARD_ROW_H * (g.rows.len() as u32) + 2;
        content_h += card_h + CARD_GAP;
    }
    content_h = content_h.saturating_sub(CARD_GAP);
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
    for g in GROUPS {
        let card_h = CARD_HEAD_H + CARD_ROW_H * (g.rows.len() as u32) + 2;
        out.push_str(&render_card(p, card_y, card_h, g));
        card_y += card_h + CARD_GAP;
    }

    out.push_str("</svg>\n");
    out
}

fn render_card(p: &super::palette::Palette, y: u32, h: u32, g: &Group) -> String {
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

    // Body rows.
    for (i, (from, to)) in g.rows.iter().enumerate() {
        let row_y = y + CARD_HEAD_H + CARD_ROW_H * (i as u32);
        if i % 2 == 1 {
            s.push_str(&format!(
                r#"  <rect x="{x}" y="{row_y}" width="{w}" height="{rh}" fill="{c}"/>
"#,
                x = card_x + 1,
                w = card_w - 2,
                rh = CARD_ROW_H,
                c = p.card,
            ));
        }
        // From cell.
        s.push_str(&format!(
            r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="11" fill="{c}">{txt}</text>
"#,
            x = card_x + CELL_PAD_X,
            ty = row_y + 17,
            f = MONO_FAMILY,
            c = p.subtext,
            txt = escape(from),
        ));
        // Arrow.
        s.push_str(&format!(
            r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="12" fill="{c}" text-anchor="middle">→</text>
"#,
            x = card_x + CELL_PAD_X + FROM_W + ARROW_W / 2,
            ty = row_y + 17,
            f = MONO_FAMILY,
            c = p.muted,
        ));
        // To cell.
        s.push_str(&format!(
            r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="11" fill="{c}">{txt}</text>
"#,
            x = card_x + CELL_PAD_X + FROM_W + ARROW_W,
            ty = row_y + 17,
            f = MONO_FAMILY,
            c = p.text,
            txt = escape(to),
        ));
    }

    s
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
