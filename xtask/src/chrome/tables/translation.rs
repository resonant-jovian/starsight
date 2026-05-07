//! "Coming from another language" — concrete syntax mappings.

use super::{Family, Table};

const HEADER: &[&str] = &["you wrote", "in starsight", "note"];
const COL_W: &[u32] = &[260, 432, 180];
const COL_ALIGN: &[&str] = &["start", "start", "start"];
const COL_FONT: &[Family] = &[Family::Mono, Family::Mono, Family::Sans];

const ROWS: &[&[&str]] = &[
    &[
        "plt.plot(x, y)  (matplotlib)",
        "Figure::new(800, 600).add(LineMark::new(x, y))",
        "no global state",
    ],
    &[
        "plt.scatter(x, y, c=c)",
        "PointMark::new(x, y).color_by(&groups)",
        "builder pattern",
    ],
    &[
        "plt.bar(labels, vals)",
        "BarMark::new(categories, values)",
        "grammar of graphics",
    ],
    &[
        "plt.boxplot([a, b])",
        "BoxPlotMark::new(vec![BoxPlotGroup::new(\"a\", a), …])",
        "label travels with data",
    ],
    &[
        "sns.violinplot(data=df, x=…, y=…)",
        "ViolinMark::new(groups).bandwidth(Bandwidth::Silverman)",
        "bandwidth is a builder",
    ],
    &[
        "plt.pie(values, labels=…)",
        "PieMark::new(values, labels).show_percent()",
        ".inner_radius(0.5) → donut",
    ],
    &[
        "mpl_finance.candlestick_ohlc",
        "CandlestickMark::new(vec![Ohlc { … }, …])",
        "inline rows; no helper crate",
    ],
    &[
        "plt.savefig(\"out.png\")",
        ".save(\"out.png\")?",
        "returns Result",
    ],
    &["plt.show()", ".show()?", "feature: interactive"],
    &[
        "sns.heatmap(data)",
        "HeatmapMark::new(data)",
        "prismatica colormaps",
    ],
    &[
        "ggplot + geom_point()",
        "Figure::new(W, H).add(PointMark::new(x, y))",
        "builder, not +",
    ],
    &[
        "px.scatter(df, x=\"a\")  (plotly)",
        "plot!(df, x=\"a\", y=\"b\")",
        "feature: polars",
    ],
];

pub fn table() -> Table<'static> {
    Table {
        stem: "translation",
        title: "starsight syntax reference for matplotlib, seaborn, ggplot2, plotly users",
        header: HEADER,
        rows: ROWS,
        col_widths: COL_W,
        col_align: COL_ALIGN,
        col_font: Some(COL_FONT),
    }
}
