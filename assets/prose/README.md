# Pre-rendered LaTeX prose cards

Each `<stem>.tex` source contains a paragraph body — prose plus inline
`$...$` math and display `\[...\]` math — that gets typeset by `latex`,
converted to a glyph-path SVG by `dvisvgm --no-fonts`, and wrapped in a
palette-themed rounded-corner card frame by `cargo xtask chrome` (or
`cargo xtask chrome --asset prose-card`). Outputs land alongside the
source as `<stem>-{light,dark}.svg`.

Why a card composite: inline math `<img>` references in the README's
markdown prose suffered viewer-dependent scaling problems (math `<img>`
heights interacted with the surrounding font's line-height differently
on crates.io vs github.com vs the GitHub mobile app). Baking the whole
paragraph into one fixed-pixel SVG card insulates layout from the
surrounding markdown flow, the same way `status_panel`,
`comparison_matrix`, and the other text-heavy chrome composites do.

Authoring conventions:

- Source body is a fragment that goes inside a `minipage` of fixed
  `\textwidth` (currently 720 pt — see `xtask/src/chrome/prose_card.rs`,
  `TEX_TEXTWIDTH_PT`). Don't include `\documentclass`, `\begin{document}`,
  the `minipage` open / close, or the preamble — `prose_card.rs` adds
  those.
- Inline math: wrap in `$...$`.
- Display math: wrap in `\[...\]` or `\begin{equation*}...\end{equation*}`.
- Use `\,` for thin spaces in numbers (`80\,000`) and large set
  delimiters (`\{13,\, 15,\, ...\}`).
- En-dashes between math variables: `$x$--$z$` (LaTeX renders `--` as an
  en-dash inside text mode).

The pipeline injects `fill="currentColor"` on the root `<svg>`, and the
xtask wrapper sets the outer `<svg>`'s `color` attribute to the palette
text colour, so glyphs inherit the right shade for `light` / `dark`
without rendering twice.

`latex` and `dvisvgm` are not hard dependencies: if either is missing
`cargo xtask chrome` warns once and skips this asset, leaving the rest
of the chrome regen intact.
