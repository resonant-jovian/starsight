# Pre-rendered LaTeX math

Each `<name>.tex` source contains a single LaTeX math expression and is
rendered to a sibling `<name>.svg` by `cargo xtask chrome` (or
`cargo xtask chrome --asset math`).

- **Inline math:** wrap in `$...$`.
- **Display math:** wrap in `\begin{equation*}...\end{equation*}`. (The
  terser `$$...$$` / `\[...\]` syntax errors with "`\dot` allowed only in
  math mode" because the `standalone` preview class wraps content in an
  `\hbox` that doesn't establish display math the same way the surrounding
  paragraph would in a regular article. `equation*` from amsmath sidesteps
  that by being its own environment.)

Rendering recipe (handled by `xtask/src/chrome/math.rs`):

1. Wrap the source in a `standalone` `\documentclass` with `border=2pt`.
2. `latex -interaction=batchmode` → tightly-cropped DVI.
3. `dvisvgm --no-fonts --exact-bbox` → single-file SVG (glyph outlines as
   `<path>` so the file works in every markdown viewer).
4. Inject `fill="currentColor"` on the root `<svg>` so the math inherits the
   surrounding text colour and one render serves both light and dark themes.

Live cron skips this step (TeXLive isn't installed on the GitHub Ubuntu
runner). Local regen requires `latex` + `dvisvgm` on `PATH` — typically a
TeXLive install. Missing tools warn once and skip; the rest of chrome regen
continues.

Why pre-render: GitHub web's markdown renderer honours `$math$` natively,
but **crates.io** and the **GitHub mobile app** render those expressions as
raw dollar-delimited text. The README references the rendered SVGs via
`<img>` tags so every surface displays the same typeset math.
