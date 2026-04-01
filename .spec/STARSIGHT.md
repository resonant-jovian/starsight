# starsight — development reference

> Authored by Claude Opus 4.6 (Anthropic) with Albin Sjögren. Last generated: 2026-03-31.

This document has four parts. Use the one you need.

**Part 1 — Listen** is pure prose with no formatting, no code blocks, no tables. Read it or pipe it through text-to-speech as preparation before sitting down to code. It explains every concept, every decision, every tricky bit in plain sentences.

**Part 2 — Build** is the task list with code blocks. Every item has the level of detail needed to implement it without looking anything else up. When it says create a struct, it shows you the exact code.

**Part 3 — Look up** is the reference section. Type signatures, dependency APIs, conversion formulas, algorithm pseudocode. Come here when you are mid-implementation and need to check a specific detail.

**Part 4 — Navigate** is the architecture map. Tree structures showing what goes where, crate dependency graphs, module layouts. Come here when you need to know which file to create or which crate a type belongs in.

---
---

# Part 1 — Listen

Read this section or have it read to you. No code blocks, no tables, no formatting that breaks text-to-speech. Just the full story of what starsight is, how it works, and what you need to know before writing a single line.

## What starsight is

starsight is a scientific visualization library for Rust. It exists because Rust has no equivalent of Python's matplotlib. The current options are plotters (powerful but verbose and stagnating), plotly-rs and charming (which secretly bundle JavaScript engines), egui_plot (locked to the egui framework), and textplots (terminal only). Researchers working in Rust end up exporting CSV and plotting in Python. starsight fixes this.

The library provides one import, sixty chart types, and five rendering backends. A user writes "plot x y dot save chart dot png" and gets a chart. A power user writes a grammar-of-graphics figure with layered marks, custom scales, faceting, and publication-quality PDF export. Both use the same library.

starsight belongs to the resonant-jovian ecosystem. Its sister crates are prismatica, which provides 308 scientific colormaps as compile-time lookup tables, and chromata, which provides 1104 editor color themes as compile-time constants. These are not optional integrations. They are the actual color and theme systems starsight uses. When starsight needs a viridis colormap, it calls prismatica dot crameri dot BATLOW dot eval of 0.5 and gets an RGB color back. When starsight needs a dark theme background color, it reads chromata dot popular dot gruvbox dot DARK HARD dot bg and gets three bytes.

## The layer architecture

The library is organized into seven layers, each a separate crate. Layer one is the foundation. Layer seven is the roof. Each layer depends only on layers below it. This is enforced by Cargo dependencies, not by convention. starsight-layer-3 literally cannot import anything from starsight-layer-5 because it is not in its dependency list.

Layer one is the rendering abstraction. It contains geometry primitives like Point, Rect, Size, and Color. It contains the DrawBackend trait that all rendering backends implement. It contains the Scene type that accumulates drawing commands. It contains the error types. It contains the backend implementations for tiny-skia (CPU), SVG, PDF, wgpu (GPU), and terminal (Kitty, Sixel, iTerm2, half-block, Braille). Everything in starsight ultimately bottoms out at layer one.

Layer two is the scale, axis, and coordinate system. A scale maps data values to pixel positions. A linear scale maps the range zero to one hundred onto the range zero to eight hundred pixels. A log scale does the same but logarithmically. Layer two also contains the tick generation algorithm, which decides where to place axis labels. starsight uses the Wilkinson Extended algorithm, which optimizes a scoring function over simplicity, coverage, density, and legibility. No Rust crate implements this algorithm. starsight will be the first. Layer two also contains coordinate systems. Cartesian is the default. Polar wraps angles. Geographic projects latitude and longitude.

Layer three is the mark and stat system. This is the grammar of graphics layer. A mark is a visual element: a point, a line, a bar, an area, a rect, an arc. A stat is a data transform: binning, kernel density estimation, regression, boxplot summary. An aesthetic mapping connects data columns to visual properties: x position, y position, color, size, shape. Position adjustments handle overlapping marks: dodge, stack, jitter. This layer does not render anything. It describes what should be rendered.

Layer four is layout and composition. Grid layouts arrange multiple charts in rows and columns. Faceting splits data by a categorical variable and creates one chart per value. Legends map visual encodings back to data values. Colorbars show the continuous color scale. Inset axes place a small chart inside a bigger one. This layer arranges charts but does not render them.

Layer five is the high-level API. The plot macro lives here. The Figure builder lives here. Data acceptance for Polars DataFrames, ndarray arrays, and Arrow RecordBatches lives here. Auto-inference of chart types from data shape lives here. This is the layer most users interact with.

Layer six is interactivity. Hover tooltips, box zoom, wheel zoom, pan, lasso selection, linked views between multiple charts, streaming data with rolling windows. This layer requires a windowing system (winit for native, web-sys for browser) and is entirely optional.

Layer seven is animation and export. Frame recording for GIF and MP4. Transition animations between chart states. Static export to PNG, SVG, PDF. Interactive HTML export. Terminal inline output with automatic protocol detection.

## Why Point and Vec2 are different types

This is a pattern from egui and from game engine math libraries. A Point is a position in space. The pixel at x equals 100, y equals 200. A Vec2 is a displacement. Fifty pixels to the right, thirty pixels down.

They are both two floats. But the valid operations are different. Subtracting one point from another gives a displacement, a Vec2. The distance from your house to the grocery store is a displacement, not a location. Adding a displacement to a point gives a new point. Your house plus the displacement to the grocery store gives the grocery store's location. But adding two points together is meaningless. Your house plus the grocery store is not a place.

The type system enforces this. Point minus Point returns Vec2. Point plus Vec2 returns Point. Point plus Point does not compile. This catches real bugs. In chart layout code, you deal with positions (where does this axis label go) and offsets (how much margin do I add). If they are both just float tuples, nothing stops you from accidentally adding two positions together and getting garbage coordinates. With separate types, the compiler catches this.

Vec2 also supports scalar multiplication. A displacement times two is twice as far in the same direction. A position times two is nonsensical. So Vec2 implements multiplication by f32, and Point does not.

## Why Color has no alpha field in the current code

The Color struct in primitives dot rs has three fields: r, g, b, all u8. There is no alpha channel. This is deliberate for the initial implementation. Most chart elements are fully opaque. The backgrounds, the axis lines, the tick labels, the titles. Alpha becomes important later for overlapping scatter points, area fill transparency, and hover highlight overlays. When alpha is needed, it should be a separate type or an optional wrapper, not baked into the base Color struct, because premultiplied alpha and straight alpha are different things and conflating them causes bugs. Tiny-skia internally uses premultiplied alpha (each RGB channel is already multiplied by the alpha value). The image crate expects straight alpha. If you store alpha in your Color type without tracking which kind it is, you will get wrong colors when converting between libraries.

For now, the Color struct matches chromata's Color and prismatica's Color, both of which are three u8 fields with no alpha. Conversion between them is zero-cost: just move the bytes.

## How tiny-skia rendering actually works

tiny-skia is a CPU rasterizer. You create a Pixmap (a pixel buffer), you draw paths and shapes onto it, you encode it as PNG. The Pixmap stores premultiplied RGBA pixels. Every pixel is four bytes: red, green, blue, alpha, where each RGB byte has already been multiplied by the alpha value divided by 255.

To draw a line, you build a Path. You call PathBuilder new, then move to the start point, then line to the end point, then finish. The finish method returns Option of Path. It returns None if the path is empty, which happens if you called finish without adding any segments.

To actually paint the path onto the Pixmap, you need a Paint struct and a Stroke struct. The Paint holds the color (via a Shader, which defaults to solid color) and the blend mode (default SourceOver). The Stroke holds the line width, line cap (Butt, Round, or Square), line join (Miter, Round, or Bevel), and optional dash pattern.

Then you call pixmap dot stroke path, passing the path, the paint, the stroke, a Transform (use identity for no transformation), and an optional Mask (pass None for no clipping, or pass Some of a Mask to restrict drawing to a region).

The critical thing about Transform is that its rotation method takes degrees, not radians. This is unlike virtually every other math library. If you pass pi divided by two expecting a 90-degree rotation, you will get a 1.57-degree rotation instead.

For text, starsight uses cosmic-text. You create a FontSystem (which loads system fonts and takes about one second in release mode), a SwashCache (no arguments), and a Buffer (with a Metrics struct specifying font size and line height in pixels). You set the text, call shape until scroll to lay it out, then call draw with a callback that receives individual glyph rectangles. Each callback invocation gives you an x, y, width, height, and color. You paint each rectangle onto the Pixmap using fill rect.

There is a persistent myth that you need to swap the red and blue channels between cosmic-text and tiny-skia. You do not. That swap exists in the cosmic-text example code because the example renders to softbuffer, which uses a different byte order. For PNG and SVG output, pass the channels straight through.

## How prismatica colormaps work

A Colormap in prismatica is a lookup table. It stores 256 RGB triplets as a static array of u8 three-element arrays compiled into the binary. When you call eval with a float between zero and one, it scales the float to the array index, interpolates linearly between the two nearest entries, and returns a Color.

The interpolation is in sRGB space, not linear space. This matches matplotlib, ParaView, and most scientific tools. Perceptual uniformity comes from how the lookup table was constructed (by Crameri, or the CET group, or matplotlib's team), not from the interpolation method.

eval rational takes two integers, i and n, and returns the i-th of n evenly spaced samples. This is useful when you have categorical data with n categories and want n distinct colors from a sequential map.

reversed returns a ReversedColormap, which is a zero-allocation wrapper that internally calls eval with one minus t. It does not copy or reverse the lookup table.

A DiscretePalette is different from a Colormap. It stores a fixed set of distinct colors for categorical data. It has get which takes an index and wraps around if the index exceeds the palette size. It has iter which returns an iterator over all colors without allocation.

## How chromata themes work

A Theme in chromata has 29 color fields plus metadata. The bg and fg fields are always present. Everything else is Option of Color because not every source theme defines every semantic role. The accent method returns the first available accent color, checking blue, then purple, then cyan, then green, then orange, then red, falling back to fg if none are defined.

The Theme struct is non-exhaustive, meaning you cannot construct it with struct literal syntax outside the crate. Use the builder: Theme builder of name, author, bg color, fg color, then chain optional setters, then call build. The build method auto-detects variant (dark if background luminance is 0.5 or below) and contrast level (from the WCAG contrast ratio between bg and fg).

## The Wilkinson Extended tick algorithm

This is the algorithm that decides where to put tick marks on an axis. Given a data range (say 3.7 to 97.2) and a desired number of ticks (say 5 to 10), it finds the "nicest" set of tick positions. Nice means: prefer round numbers (10, 20, 30 over 13.7, 27.4, 41.1), cover the data range without too much whitespace, get close to the desired tick count, and include zero if the data range spans zero.

The algorithm searches over a preference-ordered list of step bases: 1, 5, 2, 2.5, 4, 3. These are ordered by human readability. Steps of 1 (giving ticks at 10, 20, 30) are preferred over steps of 5 (giving ticks at 5, 10, 15, 20) which are preferred over steps of 2 (giving ticks at 2, 4, 6, 8, 10). The skip factor j multiplies these: skip 2 with base 5 gives step 10, which normalizes to base 1 at the next order of magnitude.

The scoring function combines four components. Simplicity (weight 0.2) rewards earlier entries in the preference list and lower skip factors. Coverage (weight 0.25) penalizes whitespace between the data range and the label range. Density (weight 0.5, the heaviest) penalizes having too many or too few ticks compared to the target count. Legibility (weight 0.05) is simplified to a constant.

The algorithm uses nested loops over j, q, k (number of ticks), z (power of ten), and start position, with aggressive pruning. At each nesting level, it computes an upper bound on the score achievable by any remaining candidate. If that upper bound is below the best score found so far, it breaks out of the loop. This makes the average iteration count about 41, which is fast enough for real-time use.

No Rust crate implements this algorithm. D3 uses a simpler formula with only three step bases. Plotters uses basic rounding. starsight will be the first Rust implementation of the full Extended Wilkinson algorithm.

## What SVG cannot do

SVG is a text format for vector graphics. starsight generates SVG documents using the svg crate, which provides a builder API: Document new, set viewBox, add elements. Each element (Path, Rectangle, Circle, Text, Group) is built with chained set calls.

The critical limitation of SVG is that you cannot measure text width without a rendering engine. The width of the string "123.45" depends on the font, the font size, kerning tables, and ligature rules. A browser can measure this after layout. A static SVG generator cannot. starsight works around this by estimating: digits are approximately 0.55 times the font size wide, average characters approximately 0.6 times. For precise measurement when generating PNG (not SVG), cosmic-text handles measurement after shaping.

Text positioning in SVG uses the baseline, not the bounding box. The x and y attributes set where the text baseline starts. To center text horizontally, set text-anchor to middle. To center vertically, set dominant-baseline to central. To rotate a Y-axis label, apply a transform: translate to the label position, then rotate negative 90 degrees.

## Edition 2024 things that matter

Rust edition 2024 (shipped with Rust 1.85) changed several things relevant to starsight. The gen keyword is now reserved for future generators, so any identifier named gen must become r#gen. The unsafe_op_in_unsafe_fn lint is now warn by default, meaning unsafe operations inside unsafe functions need explicit unsafe blocks. RPIT (return position impl trait) lifetime capture rules changed: functions returning impl Trait now capture all in-scope lifetimes by default, which can affect public API signatures.

Resolver 3 (implied by edition 2024) adds MSRV-aware dependency resolution. If a dependency's latest version requires a newer Rust than your declared rust-version, Cargo falls back to an older compatible version. Feature unification behavior is unchanged from resolver 2.

## How a chart gets from data to pixels

This is the end-to-end pipeline. Understanding it is more important than understanding any single component, because every bug you will encounter lives at a boundary between two of these stages.

You start with data. Two arrays of floating point numbers, or a Polars DataFrame with column names, or an ndarray matrix. The data enters through layer five, either via the plot macro or the Figure builder. Layer five does not know how to render anything. It assembles a description: this data should become a line chart, with this color, at this size.

The Figure builder collects marks. A mark is a description of a visual element, not the element itself. A LineMark holds the x data, the y data, a color, and a line width. It does not hold any pixel coordinates. It does not know how big the chart will be or where the axes go.

When you call save or show, the Figure asks layer two to create scales from the data ranges. The scale computes the data domain: the minimum and maximum of the x values, the minimum and maximum of the y values. It then runs the Wilkinson Extended tick algorithm to find nice tick positions. The tick positions may extend beyond the data range. If your data goes from 3.7 to 97.2, the ticks might be 0, 20, 40, 60, 80, 100. The scale domain becomes 0 to 100, not 3.7 to 97.2. This is called "nice" bounds.

Next, the Figure computes the plot area. The full image might be 800 by 600 pixels, but the chart does not fill the entire image. There are margins for the title at the top, the axis labels on the left and bottom, the tick labels, and padding. The plot area is the remaining rectangle after subtracting all margins. Computing the margins requires knowing how wide the tick labels are, which requires measuring text, which requires the font system. This is one of the trickiest parts of the codebase because text measurement and layout computation are circular: you need to know the margins to know the plot area, but you need to know the tick labels (which depend on the scale) to know the margins.

The solution is two passes. First pass: create the scales, generate the tick labels, measure the tick labels using cosmic-text, compute the margins, compute the plot area. Second pass: now that you have the plot area, create a CartesianCoord that maps data values to pixel positions within that area. The Y axis is inverted because screen coordinates increase downward but data values increase upward.

With the coordinate system established, each mark renders itself. The LineMark iterates its data points, calls data to pixel for each one, and produces a sequence of path commands: move to the first pixel position, line to the second, line to the third, and so on. If any data value is NaN, it starts a new move to, which breaks the line at that point. The path commands are backend-agnostic. They are just a list of instructions: move here, draw a line to there, close the path.

The path commands then hit the backend. The tiny-skia backend converts them to a tiny skia Path using PathBuilder, creates a Paint with the right color and a Stroke with the right width and line cap, and calls pixmap stroke path. The SVG backend converts them to SVG path data strings and adds them to the document. Same data, different output.

Finally, the backend serializes the result. The tiny-skia backend calls encode png to get bytes, or save png to write a file. The SVG backend calls document to string or svg save. The result is a file on disk or bytes in memory.

Every step in this pipeline is a separate concern in a separate layer. Data acceptance is layer five. Scale computation is layer two. Mark description is layer three. Rendering is layer one. When something goes wrong, the layer boundary tells you where to look.

## How the DrawBackend trait works and why it must stay object-safe

The DrawBackend trait is the contract between charts and rendering engines. Any type that implements DrawBackend can turn path commands, text, and rectangles into visible output. The tiny-skia backend implements it by rasterizing to a pixel buffer. The SVG backend implements it by building an XML document. A hypothetical cairo backend would implement it by calling cairo C functions.

The trait is object-safe, meaning you can write dyn DrawBackend and use it as a trait object. This is critical because the Figure does not know at compile time which backend it will render to. The user calls save with a file path, the Figure checks the extension, and picks the backend at runtime. If the extension is png, it creates a SkiaBackend. If the extension is svg, it creates an SvgBackend. This requires dynamic dispatch, which requires the trait to be object-safe.

A trait is object-safe if none of its methods use Self as a return type, none of its methods use generic type parameters, and the trait does not require Sized. Plotters made the mistake of adding a Sized bound to their DrawingBackend trait, which prevents dynamic dispatch entirely. Every plotters function that accepts a backend must be generic over the backend type, which infects all downstream code with generic parameters. This is why extracting a plotters chart-drawing function into a helper is famously difficult, and it is one of the most common complaints in their issue tracker.

starsight avoids this by keeping DrawBackend object-safe from day one. The render method on Scene takes a mutable reference to dyn DrawBackend. No generics, no Sized bound, no monomorphization overhead.

## The Scene graph is data, not behavior

Scene is a struct that holds a vector of SceneNode values. A SceneNode is an enum with variants for Path, Text, Group (with children and a transform), and Clip (with a rect and a child). The Scene does not know how to render itself. It is pure data. You build a Scene by pushing nodes into it, and then you hand the Scene to a backend which reads the nodes and renders them.

This is the pattern used by Vello (flat encoding), egui (clipped shapes list), and every modern Rust graphics library. The alternative, used by Plotters, is to make charts call backend methods directly during construction. That approach tangles chart logic with rendering logic, makes testing harder (you cannot inspect the scene without rendering it), and prevents optimizations like batching or reordering draw calls.

With a data-based scene, you can serialize it for debugging, compare two scenes for equality in tests, render the same scene to multiple backends without re-running the chart logic, and build the scene on one thread while rendering it on another.

## How clipping works

When you draw a line chart, the line should not extend beyond the plot area. If a data point maps to a pixel position outside the chart rectangle (because the scale extends beyond the data range, or because of padding), the line segment to that point should be cropped at the boundary.

In tiny-skia, clipping uses a Mask. A Mask is a grayscale image the same size as the pixmap. White areas allow drawing, black areas block it. You create a Mask, fill a rectangle into it (the plot area), and then pass it to every draw call as the last parameter. Any pixels that fall outside the mask region are silently discarded.

This is much simpler than the alternative, which is computing the intersection of every line segment with the clipping rectangle (Cohen-Sutherland algorithm). For SVG output, clipping uses the clip-path element. You define a rectangle in a defs block, reference it via clip-path attribute on a group, and the browser handles the rest.

The key insight is that clipping is a backend concern, not a mark concern. The LineMark does not need to know about clipping. It produces path commands for all data points including those outside the plot area. The backend applies the mask. This keeps mark code simple and makes the clipping behavior consistent across all mark types.

## Anti-aliasing and why charts look bad without it

Anti-aliasing smooths the jagged edges of diagonal lines and curves by blending edge pixels with the background. Without it, a line at a slight angle shows visible staircase steps. In tiny-skia, anti-aliasing is controlled by the anti alias field on Paint, which defaults to true.

For chart rendering, anti-aliasing should be on for all geometric elements: lines, areas, bars with rounded corners, circles. It should be off for axis lines and tick marks that are exactly horizontal or exactly vertical, because anti-aliasing a perfectly aligned line makes it appear blurry (it bleeds into adjacent pixels instead of being a crisp single-pixel line). It should also be off for glyph rectangles from cosmic-text, because the text rasterizer already handles its own anti-aliasing.

The practical rule: set paint dot anti alias to true by default, set it to false when drawing horizontal or vertical lines at integer coordinates, and set it to false when compositing text glyphs.

## The color conversion pipeline

Colors in starsight flow through multiple representations on the way from user intent to rendered pixel. Understanding the pipeline prevents an entire category of subtle bugs where colors look slightly wrong.

A user specifies a color in one of several ways. They might use a named constant like Color RED. They might use a hex literal like Color from hex 0xFF8000. They might sample a prismatica colormap like BATLOW dot eval of 0.5. They might read a theme field like gruvbox DARK HARD dot keyword. All of these produce a starsight Color: three u8 values in sRGB space, no alpha.

When the SkiaBackend needs to draw with this color, it converts to tiny skia Color using from rgba8 with alpha 255. This is a straight-alpha f32 Color. Internally, tiny-skia premultiplies it: each RGB channel is multiplied by the alpha value. For fully opaque colors (alpha 255), premultiplication is a no-op because multiplying by 255/255 equals 1. But for semi-transparent colors (like scatter point alpha of 0.5), premultiplication means the stored RGB values are half what you specified. This matters when reading back pixel data for testing or compositing.

When encoding a pixmap to PNG, tiny-skia demultiplies the pixels back to straight alpha. The encode png method handles this automatically. If you ever need to read raw pixel data from the pixmap, remember that the bytes are premultiplied. The formula to recover straight alpha is: straight r equals premultiplied r times 255 divided by alpha, except when alpha is zero (transparent), where all channels are zero.

For the SVG backend, none of this matters. SVG uses CSS color strings like fill equals hash ff8000. The SVG backend converts starsight Color to a hex string and writes it directly. No alpha premultiplication, no pixel format concerns.

## How text rendering actually works end to end

Text is the hardest part of chart rendering. The reason is that text involves four separate systems that must cooperate: a font database, a shaping engine, a layout engine, and a rasterizer.

The font database (managed by cosmic-text's FontSystem) knows which fonts are available on the system. When you request "14 pixel sans-serif", the database resolves this to a specific font file, like DejaVu Sans Regular at 14 pixels. On Linux, it reads the fontconfig database. On macOS, it queries Core Text. On Windows, it reads the registry. This resolution takes about one second in release mode and up to ten seconds in debug mode, which is why FontSystem must be created once and reused, not created per draw call.

The shaping engine (harfbuzz, via cosmic-text's internal harfrust port) converts a string of Unicode characters into a sequence of positioned glyphs. Shaping handles ligatures (f plus i becoming the fi ligature), kerning (adjusting the space between specific letter pairs like AV), mark attachment (combining diacritics with base characters), and complex scripts (Arabic, Devanagari, Thai). For chart labels with Latin digits and letters, shaping mostly just assigns glyph indices and advance widths. But it still must run, because even Latin text has kerning.

The layout engine arranges the shaped glyphs into lines. For chart tick labels, this is trivial because each label is a single line. For multi-line titles or wrapped annotations, the layout engine decides where to break lines. cosmic-text's Buffer manages this. You set the text, set the maximum width, call shape until scroll, and the buffer computes line breaks and glyph positions.

The rasterizer converts glyph outlines into pixel coverage values. cosmic-text uses swash for this, accessed through SwashCache. Each glyph is rasterized once at a given size and cached. The draw callback delivers rectangular regions with alpha values representing how much of each pixel is covered by the glyph outline.

For measuring text (needed for margin calculation), you iterate layout runs after shaping. Each run has a line w field (width in pixels) and line height field. The total width is the maximum line w across all runs. The total height is the last run's line top plus its line height.

The critical integration detail: cosmic-text and tiny-skia use different color types. cosmic-text Color has r, g, b, a as u8 values. tiny-skia Color has r, g, b, a as f32 values. The conversion goes through set color rgba8 on the Paint struct, which does the division by 255 internally.

## How builder patterns work in this codebase

The Figure builder uses mutable reference returns: each setter takes and mut self and returns and mut Self. This lets you chain calls or use them separately. The chained style looks like figure dot title of "Chart" dot x label of "Time" dot size of 800 comma 600. The separate style looks like: let mut fig equals Figure new, then on the next line fig dot title "Chart", then fig dot size 800 600.

This pattern was chosen over consuming self (where each method takes self by value and returns Self) because consuming self is awkward with conditional configuration. With mutable references, you can write: if show legend then fig dot legend of true. With consuming self, you would have to write: let fig equals if show legend then fig dot legend of true else fig. The consuming style also prevents partially configuring a builder, storing it, and configuring more later.

The exception is the build or save method, which does consume self (or borrows immutably and clones what it needs). This prevents accidentally modifying a figure after it has been rendered.

For mark types like LineMark and PointMark, the types are plain structs with public fields. No builder needed. You construct them with struct literal syntax. This is simpler and appropriate for types with a small number of fields where most fields are always specified.

## Why snapshot testing is the first thing to set up

Most bugs in a visualization library are visual: a line is in the wrong position, a color is slightly off, text overlaps an axis, a bar is one pixel too wide. Unit tests cannot catch these bugs because they test individual computations, not the final rendered output.

Snapshot testing with insta solves this. You render a chart to PNG bytes, pass the bytes to assert binary snapshot, and insta stores the PNG file alongside the test. When you run the tests again, insta compares the new output to the stored snapshot byte for byte. If anything changed, the test fails and cargo insta review shows you the old and new images side by side.

The key property is determinism. The tiny-skia backend is CPU-only and deterministic: the same inputs always produce the same pixel values. There is no GPU floating point variance, no driver-dependent rounding, no font substitution. This means snapshot tests never flake. If a test fails, something actually changed.

Set up snapshot testing before writing any chart code. The very first test should render a white rectangle with a blue rect on it. This validates the entire pipeline: SkiaBackend construction, fill, fill rect, encode png, and insta integration. Every chart type you add after that gets a snapshot test before the implementation is considered complete.

## How feature gating works across the workspace

Every optional dependency must be behind a feature flag. The user who writes cargo add starsight should get CPU rendering, SVG output, and PNG export. They should not get wgpu, polars, ratatui, or nalgebra unless they ask for them.

Feature flags cascade through the workspace. When a user enables the gpu feature on the starsight facade crate, it must propagate to starsight-layer-1 (which holds the wgpu backend code) and any other layer that has GPU-specific behavior. This is done through feature forwarding in Cargo.toml: in the starsight crate, the gpu feature enables the gpu feature on starsight-layer-1.

The tricky part is feature unification. In a Cargo workspace, when you run cargo test or cargo build, Cargo builds all crates with the union of all requested features. If starsight-layer-7 depends on starsight-layer-1 and enables the terminal feature, and starsight-layer-1 depends on ratatui behind the terminal feature, then cargo test with the workspace will build ratatui even if you only wanted to test layer two. The mitigation is to test individual crates with cargo test minus p starsight-layer-2 during development, and to test the full workspace with cargo test minus minus workspace only in CI.

## Why the license is GPL-3.0 and what that means for the codebase

starsight is GPL-3.0-only. Not MIT, not Apache-2.0, not dual-licensed. This is an intentional choice. The sister crates chromata and prismatica are also GPL-3.0. The license is viral: any program that links starsight into its binary must also be distributed under GPL-3.0 or a compatible license. This means proprietary applications cannot use starsight without releasing their source code.

For the codebase, this means every dependency must be GPL-3.0 compatible. MIT, Apache-2.0, BSD, ISC, Zlib, and similar permissive licenses are all compatible. LGPL is compatible. Proprietary licenses and SSPL are not. The deny.toml file configures cargo-deny to check this: any dependency with an incompatible license will fail CI.

The practical impact during development: before adding a new dependency, check its license. All current workspace dependencies (tiny-skia is BSD-3, cosmic-text is MIT/Apache-2.0, thiserror is MIT/Apache-2.0, image is MIT/Apache-2.0, svg is MIT/Apache-2.0) are permissive and therefore GPL-compatible.

## How errors should flow through the layers

The StarsightError enum in layer one has seven variants: Render, Data, Io, Scale, Export, Config, and Unknown. The Io variant has a from attribute on std io Error, which means the question mark operator automatically converts io errors. The other variants take a String message.

The design principle is: errors cross layer boundaries as StarsightError. Within a layer, you may use layer-specific error types if the granularity helps, but any error that escapes a public function must be a StarsightError. This prevents leaking dependency-specific error types through the public API. If tiny-skia has an error type, do not expose it. Wrap it in StarsightError Render with a descriptive message.

Never panic in library code. The only exception is the show method on Figure, which may panic if no display backend is available (no GPU, no terminal, no window system). Even then, prefer returning an error and let the user decide whether to panic. The panic is a last resort for the case where the user called show in an environment where showing is physically impossible.

Never use unwrap or expect in library code. Use the question mark operator. Use ok or else to convert Options into Results. Use map err to convert foreign errors into StarsightError. The clippy configuration forbids unwrap and expect.

## What the xtask crate is for

The xtask crate is a binary that lives in the workspace but is never published. It automates development tasks that are too complex for shell scripts but do not belong in the library code. The standard Rust convention is to run these tasks with cargo xtask followed by a subcommand.

For starsight, xtask will eventually handle: generating the gallery (running all examples and collecting their PNG output into a directory), running benchmarks (rendering a standard set of charts and measuring time and memory), checking that all example files compile, and preparing release artifacts. For now, its main dot rs is an empty function. Fill it in as the need arises.

The xtask pattern works because Cargo allows running binaries from workspace members without installing them. You add a .cargo/config.toml file at the workspace root with the alias: xtask equals run minus minus manifest-path xtask/Cargo.toml minus minus. Then cargo xtask gallery runs the xtask binary with the gallery argument.

## How publishing works for a workspace with interdependent crates

When you publish starsight to crates.io, you must publish the crates in dependency order. starsight-layer-1 first (it has no internal dependencies), then layer-2 (which depends on layer-1), then layer-3, and so on up to starsight (the facade which depends on everything).

Each crate's Cargo.toml must specify both path and version for workspace dependencies. During development, Cargo uses the path. During publication, Cargo strips the path and uses the version. If starsight-layer-2 depends on starsight-layer-1 at version 0.1.0, and you have not published 0.1.0 of layer-1 yet, publishing layer-2 will fail.

Cargo 1.90 and later support cargo publish minus minus workspace, which automates the topological sort and publishes all crates in the correct order. For earlier Cargo versions, use the cargo-release tool. The release workflow in GitHub Actions already has the correct order listed (commented out, ready to uncomment when the first publish happens).

crates.io also supports Trusted Publishing via OIDC tokens, which means the GitHub Action can publish without a stored API token. The workflow generates a short-lived token via the crates-io-auth-action. This is more secure than storing a long-lived CARGO_REGISTRY_TOKEN secret.

One important detail: each published crate must have a unique, meaningful description in its Cargo.toml. The current descriptions ("Layer 1: Rendering abstraction", "Layer 2: Scale, axis, coordinate") are fine for internal use but should be expanded before the first publish. crates.io search weights the description heavily.

## What the grammar of graphics actually means

The grammar of graphics is a theory from Leland Wilkinson's 1999 book. The core idea is that every chart is a composition of independent components: data, aesthetic mappings, geometric marks, statistical transforms, position adjustments, scales, coordinate systems, and facets. Instead of having a "scatter plot function" and a "bar chart function" and a "box plot function" as separate things, you have a small set of composable pieces that combine to produce any chart.

An aesthetic mapping connects a column of data to a visual property. The mapping x equals sepal length means the sepal length column controls horizontal position. The mapping color equals species means the species column controls the color of each point. The mapping size equals population means the population column controls the radius of each circle. Aesthetics are declarations. They do not compute anything. They say "this data dimension should drive this visual dimension."

A geometric mark is the visual shape drawn for each data point. A point mark draws a circle. A line mark connects points with a line segment. A bar mark draws a rectangle from a baseline to the data value. An area mark fills the region between a line and the baseline. Marks read the aesthetic mappings to determine their visual properties: where to draw, what color, what size.

A statistical transform preprocesses data before the mark renders. A bin transform groups continuous data into histogram buckets and counts the number of points in each bucket. A KDE transform estimates a smooth probability density curve from discrete data points. A regression transform fits a line or curve to the data. A boxplot transform computes the five-number summary (minimum, first quartile, median, third quartile, maximum) from the data. The stat runs before the mark. A histogram is not a special chart type. It is a bar mark applied to data that has been bin-transformed.

A position adjustment handles overlapping marks. If you have a bar chart with two categories at the same x position, dodge places them side by side. Stack places them on top of each other. Jitter adds random noise to prevent overplotting in scatter plots where many points share the same coordinates.

This decomposition is why starsight has a marks layer (layer three) separate from a high-level API layer (layer five). Layer three provides the composable pieces. Layer five provides convenient shortcuts that assemble the pieces for common chart types. When a user writes plot of data frame comma x equals petal length comma kind equals Histogram, layer five internally creates a bin stat transform piped into a bar mark with a linear x scale and a count y scale.

## How scales map data to visual space

A scale is a function that converts a data value into a visual value. The simplest is a linear scale: given a data domain of 0 to 100 and a visual range of 0 to 800 pixels, the value 50 maps to pixel 400. The formula is output equals (input minus domain min) divided by (domain max minus domain min) times (range max minus range min) plus range min.

A log scale applies a logarithm before the linear mapping. This compresses large values and spreads out small values. A data domain of 1 to 10000 on a log scale spaces 1, 10, 100, 1000, 10000 evenly. Log scales require strictly positive data. A value of zero would produce negative infinity, and a negative value would produce NaN.

A symlog scale (symmetric log) handles data that spans zero. It applies a logarithm to the absolute value and preserves the sign. The region near zero is linear (to avoid the log singularity), and the regions far from zero are logarithmic. The transition point is controlled by a threshold parameter. Symlog is useful for data like stock returns, temperature anomalies, or any measurement that can be positive or negative with a wide dynamic range.

A categorical scale maps discrete labels to evenly spaced positions. The categories "apple", "banana", "cherry" might map to pixel positions 100, 300, 500. The spacing between categories is determined by the band width. A band scale adds the concept of a bar width within each category position, which is how grouped and stacked bar charts know how wide to make each bar.

A color scale maps data values to colors using a prismatica colormap. A sequential color scale maps the range 0 to 100 onto a gradient from dark blue to bright yellow. A diverging color scale maps negative values to one color, positive values to another, with a neutral center. A qualitative color scale assigns distinct colors from a discrete palette to each category.

## How bar charts compute their positions

Bar charts seem simple but involve subtle geometry. A vertical bar chart has a categorical x axis and a continuous y axis. Each bar sits at a category position, extends from a baseline (usually zero) to the data value, and has a width determined by the band scale.

For a basic bar chart with one series, the band scale divides the x axis into equal bands with gaps between them. Each bar fills its band minus some inner padding. The typical padding is about ten percent of the band width on each side.

For a grouped bar chart with multiple series, the bars within each category are placed side by side. The band for each category is subdivided into smaller bands, one per series. This requires a nested band scale: the outer scale positions the categories, the inner scale positions the series within each category.

For a stacked bar chart, the bars within each category are placed on top of each other. The first series draws from zero to its value. The second series draws from where the first ended to its cumulative value. This requires computing cumulative sums per category. The stack position adjustment handles this computation.

For a horizontal bar chart, swap x and y. The categorical axis becomes vertical, the continuous axis becomes horizontal. This is entirely a coordinate system concern, not a mark concern. The same bar mark works in both orientations by reading its position from the coordinate system.

## How area fills differ from stroked paths

A stroked path draws an outline. You see the line but not the interior. A filled path draws the interior. You see the solid region but not the outline (unless you also stroke it).

For area charts, you need a closed path. The line of data points goes from left to right across the chart, then the path drops straight down to the baseline, runs back along the baseline to the start, and closes. The fill rule (usually Winding) determines what counts as "inside" when paths cross themselves.

In tiny-skia, fill path and stroke path are separate calls on the pixmap. You can fill a path, then stroke it, to get a filled shape with a visible border. The fill uses the Paint's shader (color or gradient), and the fill rule (Winding or EvenOdd). There is no Stroke parameter for fills.

For semi-transparent area fills (common when overlaying multiple series), the ColorAlpha type comes into play. You set the paint color with an alpha value less than 255. The overlapping regions will appear darker because the alpha values accumulate through Porter-Duff compositing. This is the correct behavior and is one of the main reasons premultiplied alpha exists in the pixel buffer.

## How faceting splits one chart into many

Faceting takes a single dataset and creates a grid of small charts, one per value of a categorical variable. If your data has a species column with values setosa, versicolor, and virginica, facet wrap on species creates three charts side by side, each showing only the data for one species. The axes are shared so the charts are directly comparable.

Facet wrap lays out the panels in a single row that wraps to multiple rows when it exceeds the available width. You specify the number of columns. Facet grid uses two variables: one for rows and one for columns, creating a matrix of panels.

The layout system in layer four handles faceting. It divides the available space into cells, computes shared axis ranges (unless free scales are requested), and creates a CartesianCoord for each cell. Each mark renders once per cell, filtered to the subset of data belonging to that cell.

Free scales versus fixed scales is a key user choice. Fixed scales mean all panels share the same axis range, making cross-panel comparison easy. Free scales allow each panel to zoom in on its own data range, which is useful when the scales are very different across groups. Free x and free y can be set independently.

## How legends connect visual properties back to data labels

A legend is generated automatically from the aesthetic mappings. If the color aesthetic maps to the species column, the legend shows a colored swatch next to each species name. If the size aesthetic maps to population, the legend shows circles of different sizes next to representative values.

Generating a legend requires knowing three things: which aesthetic is mapped (color, size, shape), which scale transforms the data values into visual values (the color scale, the size scale), and what the unique data values or representative breakpoints are.

For categorical aesthetics (color mapped to species names), the legend shows one entry per unique value. For continuous aesthetics (size mapped to population), the legend shows a few representative values spanning the data range.

Legend placement defaults to the right side of the chart but can be moved to the top, bottom, left, or inside the plot area. When a chart has multiple aesthetic mappings (color and size both mapped), it gets multiple legends stacked vertically.

## How the terminal backend cascade works

starsight supports rendering charts directly in the terminal. Different terminals support different graphics protocols, and they vary wildly in capability. Kitty protocol can display full color images at pixel resolution. Sixel protocol (supported by mlterm, WezTerm, and some xterm builds) can display images at a reduced color depth. iTerm2 has its own inline image protocol. Terminals without any image protocol fall back to character-based rendering: half-block characters (the upper half and lower half block Unicode characters, giving twice the vertical resolution of regular characters) or Braille dot patterns (giving eight dots per character cell for line drawing).

The detection cascade queries terminal capabilities in order: try Kitty first (by sending a query escape sequence and checking the response), then Sixel (by checking the TERM and COLORTERM environment variables and sending a device attributes query), then iTerm2 (by checking the TERM_PROGRAM variable), then fall back to half-block, and finally Braille as the lowest-fidelity option.

For the Kitty and Sixel protocols, the chart is rendered to a PNG via the tiny-skia backend at a resolution appropriate for the terminal's cell size, then encoded in the protocol's format and written to stdout as escape sequences. The terminal interprets the escape sequences and displays the image inline. For half-block and Braille, the chart is rendered at low resolution and the pixel values are mapped to Unicode characters with foreground and background colors.

The terminal backend lives in layer one because it is a rendering backend, not an export format. It implements DrawBackend just like the tiny-skia and SVG backends. From the perspective of the marks and the figure builder, terminal rendering is indistinguishable from PNG rendering. The difference is only in how the final pixels reach the user's eyes.

## How GPU rendering differs from CPU rendering

The tiny-skia CPU backend rasterizes one primitive at a time, sequentially, on a single core. It processes each pixel of each path, applying paint, blend mode, and anti-aliasing. This is simple, deterministic, and correct, but it scales linearly with the number of primitives and pixels. A scatter plot with a million points takes proportionally longer than one with a thousand.

The wgpu GPU backend works fundamentally differently. Instead of rasterizing paths, it tessellates them into triangles (using lyon or a similar library), uploads the triangle vertices to GPU memory as a vertex buffer, sets up shader programs that compute color and blending on the GPU, and issues a single draw call. The GPU processes all triangles in parallel across hundreds or thousands of cores.

For charts with many primitives (large scatter plots, dense heatmaps, real-time streaming data), the GPU backend is orders of magnitude faster. For simple charts with a few dozen elements, the CPU backend is actually faster because it avoids the overhead of GPU initialization, buffer uploads, and shader compilation.

The critical design consequence is that the DrawBackend trait must accommodate both models. The CPU backend can handle draw path calls one at a time. The GPU backend needs to batch all draw calls and submit them together. The solution is the Scene graph. Instead of calling backend methods directly during chart construction, marks emit SceneNode data into a Scene. The Scene is then handed to the backend, which can process all nodes in whatever order and batching strategy it prefers. The CPU backend iterates the nodes sequentially. The GPU backend sorts them by shader, batches compatible draw calls, and submits minimal draw calls.

The wgpu backend is entirely optional, behind the gpu feature flag. It is not needed for 0.1.0 and should not distract from getting the CPU rendering pipeline correct first. But the Scene-based architecture exists specifically to make adding the GPU backend later a matter of implementing a new backend, not restructuring the entire library.

## How interactive charts handle events

An interactive chart runs inside an event loop. On native platforms, this is a winit event loop that receives keyboard, mouse, and window events from the operating system. On the web, this is a requestAnimationFrame loop that receives events from the browser DOM.

When the user moves the mouse over the chart, the event loop receives a cursor moved event with the pixel coordinates. The chart needs to determine what the cursor is hovering over: which data point, which axis, which legend entry. This is called hit testing. Hit testing converts the pixel coordinates back to data coordinates using the inverse of the CartesianCoord mapping, then checks which data points are within a threshold distance of the cursor.

When the user scrolls the mouse wheel, the chart zooms. Zooming modifies the scale domain: scrolling in narrows the domain (zooming in), scrolling out widens it (zooming out). The zoom center is the cursor position, so the point under the cursor stays fixed while everything else scales around it. This requires converting the cursor pixel position to data coordinates, adjusting the domain bounds, and re-rendering.

When the user clicks and drags, the chart pans. Panning shifts the scale domain by the amount the cursor moved, converted from pixels to data units. Click and drag can also mean box selection or lasso selection, depending on the active tool.

All interactive state (current zoom level, pan offset, selected points, hover target) lives in layer six. The marks in layer three and the figure in layer five are stateless descriptions. The interactive layer wraps a figure and maintains the mutable state. Each frame, it applies the current interactive state to the figure's scales, re-renders the scene, and presents it to the window.

## How streaming data works

Streaming charts display data that arrives continuously over time, like sensor readings, stock prices, or server metrics. The chart shows a rolling window of the most recent data and updates as new points arrive.

The streaming API is push-based, not pull-based. The user calls figure dot append with a new data point. The figure adds it to an internal buffer, removes points that have fallen outside the rolling window, updates the scale domains to reflect the new data range, and triggers a re-render.

The rolling window is defined by a time duration or a maximum number of points. A time-based window of 60 seconds shows all data from the last minute, regardless of how many points that is. A count-based window of 1000 points shows the most recent thousand, regardless of the time span.

The performance challenge is that re-rendering the entire chart for every new data point is expensive. The solution is incremental updates: when a new point arrives, instead of rebuilding the entire scene from scratch, the chart appends the new line segment to the existing path and removes the oldest segment. This requires the Scene to support mutation, which is why the Scene holds mutable vectors of nodes rather than being an immutable data structure.

## What makes a good default theme

The default theme determines what a chart looks like when the user does not customize anything. It must be readable on both light and dark backgrounds, print well in grayscale, work for colorblind users, and look professional without being boring.

The default background is white. The default foreground is a dark gray, not pure black, because pure black on white creates excessive contrast that is fatiguing to read. Axis lines and tick marks are medium gray. Grid lines are very light gray with low opacity so they are visible but do not compete with the data.

The default color cycle (the sequence of colors used for multiple series) should be distinguishable by colorblind users. The typical approach is to start with colors that vary in both hue and lightness, not just hue. Two shades of red and blue that differ only in hue look identical to someone with deuteranopia. But a dark blue and a light orange are distinguishable by hue, by lightness, and by saturation. prismatica's qualitative palettes like Set2 and Tableau10 are designed with this constraint.

The default font should be a system sans-serif font at 12 to 14 points. Chart titles are slightly larger, around 16 points. Tick labels are slightly smaller, around 10 to 11 points. These sizes work for both screen display (where charts are typically 600 to 1200 pixels wide) and print (where charts are typically 3 to 6 inches wide at 300 DPI).

When a user applies a chromata theme, starsight derives the chart theme from the editor theme. The editor background becomes the chart background. The editor foreground becomes the axis and text color. The editor accent colors become the data series color cycle. The editor selection color becomes the grid line color. Not every editor theme makes a good chart theme, but the mapping provides a reasonable starting point that the user can override.

## How PDF export differs from raster and SVG

PDF is a page description language, not a pixel format and not an XML format. It uses a stack-based drawing model similar to PostScript. You push a graphics state, set a color, draw a path, fill or stroke it, and pop the state. Text in PDF is positioned with exact glyph coordinates and references embedded font programs.

starsight uses the krilla crate for PDF export. krilla provides a high-level API for creating PDF documents with correct text handling, color management, and page geometry. The critical advantage of PDF over SVG for publication use is that PDF embeds the actual font glyphs used in the document. The recipient does not need the font installed. SVG references font names by string, and the rendering depends on whatever font the viewer's system substitutes.

PDF also supports precise color management via ICC profiles. A PDF can declare that its colors are in sRGB, Adobe RGB, or a custom profile. This matters for print workflows where color accuracy is critical. SVG has limited color management support that varies by viewer.

The PDF backend in starsight works differently from the tiny-skia and SVG backends. It does not implement DrawBackend directly (though it could). Instead, it renders the same Scene using krilla's drawing API, which outputs PDF content streams. The conversion from SceneNode to PDF operations is similar to the SVG conversion: paths become PDF path operators, text becomes PDF text operators, groups with transforms become PDF save/restore blocks.

PDF export is gated behind the pdf feature flag and is planned for version 0.10.0.

## Why starsight does not use async

Rust's async ecosystem is powerful but adds significant complexity: every async function requires an executor runtime (tokio, async-std, smol), error types must be Send and Sync, and the colored function problem means async infects every call site above it.

starsight is a visualization library, not a network service. Its operations are CPU-bound (rasterization, layout computation, text shaping), not I/O-bound (waiting for network responses, reading files). CPU-bound work does not benefit from async. An async rasterizer is just a synchronous rasterizer with extra overhead.

The one place where async might seem natural is streaming data: receiving sensor readings from an async channel and updating a chart. starsight handles this with a push-based synchronous API instead. The user calls append from their own async context (or synchronous context, or signal handler, or whatever). The figure does not know or care whether the caller is async.

This design means starsight has zero dependency on any async runtime. It works equally well in a tokio application, a bare metal embedded system, a WASM browser environment, and a simple synchronous command-line tool. Adding a tokio dependency to a visualization library would be an architectural mistake that constrains every downstream user.

## How Polars DataFrames integrate with the plot macro

Polars is a DataFrame library for Rust. A DataFrame is a table of named columns, where each column is a Series of typed values. When starsight accepts a DataFrame, it needs to extract columns by name and convert them to the internal data representation.

The plot macro has a special form for DataFrames: plot of ampersand df comma x equals "column name" comma y equals "column name". The string literals are column names. At runtime, the macro generates code that calls df dot column of "column name" and extracts the values as a slice of f64. If the column does not exist, or if it contains non-numeric data, the operation returns a StarsightError Data with a message explaining which column was missing or had the wrong type.

The DataFrame integration lives in layer five behind the polars feature flag. Layer five depends on polars only when that feature is enabled. Layers one through four have no knowledge of DataFrames. The data acceptance module in layer five converts DataFrame columns to plain Vec of f64 before passing them to marks in layer three.

This design means adding support for a new data source (like an Arrow RecordBatch or a CSV reader) is purely a layer five concern. You write a new data acceptance module that converts the external format to Vec of f64, and every mark and scale in the lower layers works automatically.

## How to debug a chart that looks wrong

When a chart renders incorrectly, the bug is at one of the pipeline boundaries. Here is how to isolate it.

First, check the data. Print the raw values. Are they what you expect? Are there NaN values or infinities? Are the x and y arrays the same length?

Second, check the scales. Print the domain min and max. Are they reasonable? Did the Wilkinson tick algorithm produce sensible tick positions? If the ticks look wrong, the scale domain is wrong, which means the data range computation is wrong.

Third, check the coordinate mapping. Pick a known data point and manually compute its expected pixel position using the formula: pixel x equals plot area left plus normalized x times plot area width, pixel y equals plot area bottom minus normalized y times plot area height. Does the actual pixel position match?

Fourth, check the path commands. Before sending them to the backend, print the PathCommand sequence. Are the move to and line to positions correct? Are there unexpected NaN values producing gaps?

Fifth, check the rendering. Render to SVG instead of PNG. Open the SVG in a browser and inspect the elements. SVG is human-readable. You can see the exact coordinates, colors, and transforms applied to each element. If the SVG looks correct but the PNG does not, the bug is in the tiny-skia backend translation.

Sixth, check clipping. Temporarily disable the mask (pass None instead of the plot area mask). If elements appear that were missing, the clipping rect is wrong, which means the margin or plot area computation is wrong.

The snapshot test approach helps here too. When you fix a visual bug, the snapshot test captures the corrected output. If the bug regresses, the snapshot comparison fails immediately.

## What a prelude should and should not export

The prelude module re-exports the types that every user needs in every program. It should contain the types that appear in the most common usage pattern: use starsight prelude star, then call plot and save.

The prelude should export: Figure (the builder everyone uses), the plot macro (the one-liner everyone starts with), Color (needed to customize colors), Point (needed for manual positioning), StarsightError and Result (needed for error handling), and whatever trait is needed for save and show to work.

The prelude should not export: backend types (SkiaBackend, SvgBackend), internal types (PathCommand, PathStyle, SceneNode), mark types (LineMark, PointMark), scale types (LinearScale), or any type that is only needed for advanced compositional use. These live in the crate's module tree and users import them explicitly when needed.

The principle is: if a type appears in the getting started example, it belongs in the prelude. If it appears only in the advanced composition example, it does not. Overstuffing the prelude pollutes the user's namespace and causes name collisions. Understuffing it forces the user to write long import lists for basic operations.

## What constitutes a breaking change in a visualization library

Semver says: increment the major version when you make incompatible API changes. But for a visualization library, visual output changes are also effectively breaking, even when the API stays identical.

If you change the default theme colors, every snapshot test downstream breaks. If you change the tick algorithm's weights, axes labels move. If you change the default line width from 2 to 1.5, every chart in every user's documentation looks different. These are not API breaks in the strict Rust sense (the code still compiles), but they are breaking changes for users who depend on reproducible output.

starsight handles this with two rules. First, visual defaults are versioned: the default theme, default colors, default sizes are documented as part of the public contract and changing them requires a minor version bump with a changelog entry. Second, the snapshot test suite captures the current visual output at every release. Any PR that changes snapshot output must be explicitly reviewed for visual acceptability.

The tool cargo-semver-checks verifies API compatibility automatically. It catches removed public items, changed function signatures, and tightened trait bounds. It does not catch visual changes, which is why the snapshot workflow exists alongside it.

## The tools you will use every day

This section covers every tool in your development workflow. Each tool is explained as if you have never used it before: what it does, why it matters for starsight specifically, how to run it, what to watch out for, and how it fits into CI. These are not abstract descriptions. They are practical operational knowledge you will need repeatedly.

## cargo-deny and why license checking matters for a GPL project

cargo-deny is a dependency governance tool that checks four things about your dependencies: their licenses, their security advisories, whether banned crates are present, and whether their source registries are trusted. You install it with cargo install cargo-deny and run it with cargo deny check.

For starsight, license checking is critical because the project is GPL-3.0. The GPL is a viral license: it requires that any program linking to starsight also be GPL-compatible. This means every dependency starsight pulls in must itself be GPL-compatible. Most Rust crates use MIT or Apache-2.0, both of which are compatible with GPL-3.0. But if a dependency uses a proprietary license, SSPL, or a GPL-incompatible copyleft license, you cannot use it.

The configuration lives in deny.toml at the workspace root. The licenses section has an allow list where you enumerate every acceptable SPDX license identifier. For a GPL project, this list includes MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, 0BSD, CC0-1.0, Unlicense, MPL-2.0, LGPL variants, and GPL variants. A common gotcha: nearly every Rust project depends on the unicode-ident crate (pulled in by proc-macro2), which uses the Unicode-DFS-2016 license. If you forget to add this to your allow list, every cargo deny check will fail with a cryptic license error on a crate you never directly depend on.

Another gotcha: the ring cryptography crate uses a mix of ISC, MIT, and OpenSSL licenses that cargo-deny cannot parse automatically. You need a clarify block in deny.toml that tells cargo-deny how to interpret ring's LICENSE file.

The advisory check queries the RustSec advisory database for known vulnerabilities in your dependency tree. This is the same database that cargo-audit uses. The difference is that cargo-deny combines it with the other three checks in a single tool. In CI, you should separate the advisory check from the license and ban checks using a matrix strategy. Advisory checks can fail unexpectedly when a new vulnerability is published for a transitive dependency, and you do not want that to block unrelated PRs. Set continue-on-error to true for the advisory job.

The ban check prevents specific crates from entering your dependency tree. For starsight, you might ban openssl in favor of rustls, or ban any crate that pulls in a C dependency you want to avoid.

The source check ensures all dependencies come from trusted registries. By default, only crates.io is trusted. If you ever need a git dependency, you must explicitly allow that git URL.

In CI, use the EmbarkStudios cargo-deny-action at version 2. It runs cargo deny check with the specified arguments. The recommended pattern is a matrix with two entries: one for advisories with continue-on-error true, and one for bans licenses sources without continue-on-error.

## cargo-audit and how it differs from cargo-deny

cargo-audit is a security-focused tool that checks your Cargo.lock against the RustSec advisory database. It overlaps with cargo-deny's advisory check but has two unique features. First, it can audit compiled binaries with cargo audit bin, which is useful for checking deployed artifacts. Second, it has an experimental cargo audit fix command that attempts to automatically update vulnerable dependencies.

For starsight, use cargo-deny for PR checks (because it combines all four checks) and cargo-audit on a daily schedule via a GitHub Actions cron job. When cargo-audit finds a vulnerability, it can automatically create a GitHub issue using the rustsec audit-check action.

## cargo-semver-checks and why you run it on every PR

cargo-semver-checks compares your current public API against the last published version on crates.io and reports any breaking changes. It works by analyzing rustdoc JSON output, which means it can detect over 120 categories of API breakage: removed public items, changed function signatures, removed trait implementations, visibility reductions, non-exhaustive additions to previously exhaustive types, and even Cargo.toml changes like removing feature flags.

The things it catches are exactly the things that are hardest to notice during code review. You rename a public method and every downstream user's code breaks. You add a new required trait method and every implementor breaks. You change a function parameter from u32 to u64 and — actually, cargo-semver-checks does not catch that one. It also misses behavioral changes, lifetime changes, and generic parameter changes. These limitations mean cargo-semver-checks is a safety net, not a guarantee.

Run it with cargo semver-checks check-release for a single crate, or cargo semver-checks for the workspace. In CI, use the obi1kenobi cargo-semver-checks-action. Run it on every PR, not just before publishing. The earlier you catch an accidental break, the easier it is to fix.

For starsight's pre-1.0 phase, every minor version bump (0.1.0 to 0.2.0) is implicitly a breaking change under semver. But running cargo-semver-checks anyway catches unintentional breaks within a minor version: if you are at 0.1.3 and accidentally remove a public method that existed in 0.1.0, it catches that.

## cargo-release and the workspace publishing dance

cargo-release automates the entire release workflow: version bumps, changelog updates, git commits, git tags, and crates.io publishing. You install it with cargo install cargo-release and configure it with release.toml at the workspace root.

The most important configuration option for starsight is dependent-version. Set it to fix. This means when you release a new version of starsight-layer-1, cargo-release updates the dependency version in starsight-layer-2 through 7 and in the facade crate, but it only bumps their patch version, not their minor or major version. The alternative, upgrade, cascades version bumps through the entire workspace, which creates unnecessary churn.

cargo-release determines the topological publish order automatically. For starsight, it will publish layer-1 first (no internal dependencies), then layer-2 (depends on layer-1), then layer-3, and so on up to the facade crate. If any publish step fails, it stops and tells you what went wrong.

The critical safety feature: cargo-release is a dry run by default. Running cargo release minor does not actually publish anything. It prints what it would do. You must pass the execute flag to actually perform the release. Always do a dry run first and read the output carefully.

For pre-releases, cargo-release supports alpha, beta, and rc commands. cargo release alpha creates a 0.2.0-alpha.1 version. Subsequent alpha releases increment the pre-release counter.

## cargo-llvm-cov and what coverage actually tells you

cargo-llvm-cov measures code coverage using LLVM's source-based instrumentation. This means it instruments the actual machine code, not the source text, giving precise per-expression coverage data.

Install it with cargo install cargo-llvm-cov. The llvm-tools-preview component must be available, but recent versions of the tool install it automatically. Run it with cargo llvm-cov for the workspace, adding the all-features flag to cover feature-gated code.

For starsight, code coverage answers a specific question: which rendering code paths are exercised by your tests? If your snapshot tests only exercise the tiny-skia backend, the SVG backend code has zero coverage. This tells you exactly where to add tests.

The tool generates several output formats. LCOV format (with the lcov flag) is what Codecov and Coveralls expect. HTML format (with the html flag) generates a browsable report you can open locally. The fail-under-lines flag fails the build if coverage drops below a threshold, which is useful for enforcing a coverage floor in CI.

Test code is excluded by default: anything in a tests directory, files matching the test suffix pattern, examples, and benchmarks. If you have utility modules that should also be excluded, use the ignore-filename-regex flag.

A critical detail: cargo-llvm-cov uses a separate target directory, not the standard target directory. This means coverage builds do not invalidate your normal build cache, and normal builds do not invalidate coverage data. But it also means the first coverage run does a full rebuild.

For CI, the workflow is: install the tool with taiki-e install-action, run cargo llvm-cov with lcov output, then upload to Codecov. The coverage workflow in starsight is already set up and runs weekly.

## cargo-insta and the snapshot testing workflow

insta is a snapshot testing library. You render something to a string or bytes, call an assertion macro, and insta stores the output as a reference file. On subsequent runs, it compares against the stored reference and fails if anything changed.

For starsight, snapshot testing is the primary mechanism for catching visual regressions. You render a chart to PNG bytes using the tiny-skia backend, pass the bytes to the binary snapshot assertion macro, and insta stores the PNG file. If a code change makes the chart look different (even one pixel), the test fails.

The workflow has three commands. cargo insta test runs all tests and creates pending files (with a dot snap dot new extension) for any mismatches. cargo insta review opens an interactive terminal interface where you see the old and new snapshots side by side and can accept or reject each change. cargo insta accept bulk-accepts all pending changes without review.

In CI, you run cargo insta test with the check flag, which fails immediately on any mismatch instead of creating pending files. You also pass the unreferenced reject flag, which fails if there are orphaned snapshot files from deleted tests. These two flags together ensure that CI catches both regressions and stale snapshots.

The snapshot files live in a snapshots directory next to the test file. For binary snapshots like PNG images, the actual binary file is stored alongside a metadata dot snap file. Both must be committed to version control.

A practical detail: snapshot names should be descriptive. If your test renders a blue rect on white background, name the snapshot blue rect on white, not test1. When a snapshot fails in CI, the name tells you exactly what regressed.

For SVG output, use the string snapshot macro instead of binary. SVG is text, so insta can show a readable diff. This is often more useful for debugging than a PNG diff because you can see exactly which element changed.

## cargo-mutants and when mutation testing is worth the cost

cargo-mutants modifies your source code in small ways (called mutations) and checks whether your tests catch the change. If you replace a plus with a minus and all tests still pass, that means your tests are not actually verifying the addition. This is stronger than coverage: coverage tells you the code was executed, mutation testing tells you the behavior was verified.

For starsight, mutation testing is most valuable on mathematical code: scale transforms, the tick algorithm, coordinate conversions, and layout computations. These are functions with clear numeric outputs where a flipped operator produces a wrong but plausible result. It is less valuable on rendering code because a mutated draw call might produce identical output (if the mutation happens to be invisible) or a completely garbled image (which snapshot tests catch anyway).

The practical barrier is time. cargo-mutants generates hundreds to thousands of mutants, and each one requires a full incremental build and test run. For a nine-crate workspace with a thirty-second test suite, expect five to ten hours for a full run. The mitigation is to run it only on changed files using the in-diff flag with a git diff, which is fast enough for PR checks.

Results come in four categories. Caught means the mutation caused a test failure, which is good. Missed means all tests passed despite the mutation, which reveals a testing gap. Unviable means the mutation did not compile, which is neutral. Timeout means the tests took too long, which usually indicates an infinite loop.

Install with cargo install cargo-mutants. Configure exclusions in a dot cargo slash mutants dot toml file. Exclude Display and Debug implementations (mutating formatting does not indicate real testing gaps) and rendering functions where snapshot tests are the appropriate verification mechanism.

## cargo-nextest and why it is faster than cargo test

cargo-nextest is a replacement test runner that executes each test as a separate process. The standard cargo test runner compiles tests into a single binary per crate and runs them on a thread pool. cargo-nextest compiles them the same way but then runs each test in its own process, which enables true parallelism across all CPU cores, isolation between tests (a panic in one test cannot corrupt another), and better output formatting.

For starsight, the speed improvement is modest during early development (not many tests yet) but becomes significant as the test suite grows. A hundred snapshot tests that each render a chart take noticeable time, and parallelizing them across cores helps.

Install with cargo install cargo-nextest. Run with cargo nextest run for the workspace. Configure test profiles in a dot config slash nextest dot toml file. The CI profile should disable fail-fast (run all tests even after the first failure) and enable retries for flaky tests (set to zero for starsight since our tests should be deterministic, but useful if you add integration tests that touch the filesystem).

The critical limitation: cargo-nextest does not run doc-tests. This is a deliberate design choice by the nextest team because doc-tests have fundamentally different compilation semantics. You must always pair cargo nextest run with cargo test minus minus doc in CI to ensure doc examples compile and run.

cargo-nextest also supports test archiving and sharding for distributed CI. You can build once, create an archive, then split the test execution across multiple CI runners. This is overkill for starsight now but good to know about.

## cargo-watch and bacon for rapid feedback loops

cargo-watch automatically re-runs a command whenever source files change. You install it with cargo install cargo-watch and run it with cargo watch minus x followed by the command. For example, cargo watch minus x check continuously rebuilds as you edit files and reports errors immediately.

The original cargo-watch is now dormant. The maintainer recommends bacon as the successor. bacon provides a terminal user interface that continuously watches your project, runs cargo check or cargo test or cargo clippy, and displays results in a clean scrollable view. Install with cargo install bacon and run it with just bacon in the project root.

For starsight development, the recommended workflow is: keep bacon running in one terminal with clippy mode, and run cargo test manually in another terminal when you want to check correctness. bacon catches compilation errors and lint warnings instantly as you save files. Tests run on demand because rendering tests take non-trivial time.

## cargo-expand for debugging the plot macro

cargo-expand shows the output of macro expansion. When you write the plot macro and something does not work, cargo expand tells you exactly what code the macro generated. Install with cargo install cargo-expand (requires nightly to be installed as a toolchain, but not as the default).

Run cargo expand with the minus p flag to target a specific crate and optionally a module path. For example, cargo expand minus p starsight-layer-5 followed by the macro module name shows the expanded output of that module. You can also expand a specific test file or example.

The expanded output is a debugging aid, not compilable code. Macro hygiene may produce identifiers that look odd. But it tells you whether the macro generated the correct structure, whether it captured the right expressions, and whether the type annotations are correct.

For the plot macro, the most common bug is incorrect expression capture. If the DataFrame form of the macro (with x equals and y equals literal tokens) does not match properly, cargo expand shows you which arm of the macro actually matched and what code it generated. This is much faster than trying to reason about macro matching rules in your head.

## cargo-udeps and cargo-machete for finding unused dependencies

Over time, dependencies accumulate. You add a crate to experiment with something, the experiment does not work out, you delete the code but forget to remove the dependency. Or a refactoring eliminates the use of a dependency without anyone noticing. Unused dependencies increase compile times and expand the attack surface.

cargo-machete is the fast option. It scans your source files with regular expressions looking for imports of each dependency. If a dependency name does not appear in any source file, it flags it. Install with cargo install cargo-machete and run with cargo machete. It takes about one second regardless of project size because it does not compile anything.

The downside of cargo-machete is false positives. It looks for the crate name as a string, which misses dependencies used through re-exports, proc macros, or build scripts. For example, if you depend on thiserror but only use it through the derive macro (which appears as the attribute thiserror Error, not as a thiserror import), cargo-machete might flag it as unused.

cargo-udeps is the accurate option. It actually compiles the project and uses the compiler's dead code analysis to find unused dependencies. This is strictly more accurate than regex scanning but requires the nightly compiler and takes a full compile cycle. Install with cargo install cargo-udeps and run with cargo plus nightly udeps.

For starsight CI, use cargo-machete on every PR (instant, catches obvious cases) and cargo-udeps on a weekly schedule (thorough, catches subtle cases). Suppress cargo-machete false positives with a metadata section in Cargo.toml listing ignored dependencies.

## git-cliff for generating changelogs from conventional commits

git-cliff reads your git history, parses commit messages that follow the Conventional Commits format, and generates a changelog. If your commits look like feat colon add linear scale support and fix colon correct Y axis inversion, git-cliff groups them under Features and Bug Fixes headings.

Install with cargo install git-cliff and initialize with git cliff minus minus init, which creates a cliff.toml configuration file. The configuration defines how commit types map to changelog sections, what to include or exclude, and the output template.

For starsight, the template should produce a Keep a Changelog format: each release has a date and groups for Added, Changed, Deprecated, Removed, Fixed, and Security. The Conventional Commits mapping is: feat maps to Added, fix maps to Fixed, perf maps to Changed, refactor maps to Changed, and chore commits are skipped.

In the release workflow, git-cliff generates release notes for the GitHub Release page. The command git cliff minus minus latest minus minus strip header extracts just the changes since the last tag. Always use fetch-depth zero in the checkout step of the GitHub Action, because shallow clones omit the git history that git-cliff needs.

For workspace changelogs, you can generate per-crate changelogs using the include-path flag. This filters commits to only those touching files within a specific crate's directory. Whether you want a single monorepo changelog or per-crate changelogs is a project decision. For starsight, a single changelog is simpler because releases are coordinated across all layers.

## taplo for formatting Cargo.toml files

taplo formats TOML files the same way rustfmt formats Rust files. You install it with cargo install taplo-cli and run it with taplo fmt. In CI, run taplo check to verify formatting without modifying files.

Why does TOML formatting matter? Because Cargo.toml files in a nine-crate workspace tend to drift in style. One person alphabetizes dependencies, another does not. One person uses inline tables, another uses dotted keys. taplo enforces consistency.

The configuration goes in a taplo.toml file at the workspace root. The most useful settings: enable key reordering for dependency sections (so dependencies are always alphabetical) but disable it for package sections (where the logical order name, version, edition, description is more readable than alphabetical).

taplo also validates TOML syntax, catching common errors like duplicate keys or invalid value types. This is occasionally useful when editing deny.toml or cliff.toml by hand.

## rust-toolchain.toml and pinning the Rust version

The rust-toolchain.toml file at the project root tells rustup which Rust version to use. When anyone runs any cargo command in the project directory, rustup reads this file, checks if the specified toolchain is installed, downloads it if necessary, and uses it. This means every contributor (and every CI runner) uses exactly the same Rust version.

For starsight, the file specifies channel 1.85.0 (the first stable release with edition 2024 support) and the components rustfmt, clippy, and llvm-tools-preview. Pinning the exact version prevents a class of bugs where one developer's nightly compiler accepts code that another developer's stable compiler rejects.

In CI, the dtolnay rust-toolchain action reads this file automatically when you specify toolchain as stable. Some CI actions like actions-rust-lang setup-rust-toolchain also read it. Having the file means your CI workflow and your local development environment are always in sync.

A gotcha: if both rust-toolchain.toml and the legacy rust-toolchain file (without the dot toml extension) exist in the project, the legacy file takes precedence. Delete the legacy file if it exists.

## cargo-msrv and verifying the minimum supported Rust version

cargo-msrv finds and verifies the minimum supported Rust version for your project. The find subcommand does a binary search across Rust versions, compiling your project with each one until it finds the oldest version that succeeds. The verify subcommand checks that the project compiles with the declared rust-version in Cargo.toml.

For starsight, the MSRV is 1.85 because that is when edition 2024 became available. cargo-msrv find would discover this automatically, but since you already know it, the useful command is cargo msrv verify, which confirms that the declared version still works.

In practice, the simplest MSRV verification in CI is to just include the MSRV in the test matrix. If your CI tests against stable and 1.85, and 1.85 passes, the MSRV is verified. This avoids installing cargo-msrv as a separate tool.

The MSRV policy for starsight is to track the latest stable minus two releases, consistent with wgpu and ratatui. This means when Rust 1.90 ships, the MSRV may advance to 1.88. Each MSRV advance is a breaking change in the strictest reading of semver, so it should be noted in the changelog.

## proptest for finding edge cases in mathematical code

proptest is a property-based testing library. Instead of writing individual test cases with specific values, you describe the properties your code should satisfy and proptest generates random inputs to check those properties. When it finds a failing input, it automatically shrinks it to the smallest reproducing case.

For starsight, the most valuable property tests target the mathematical foundations. The scale roundtrip property says: for any value in the domain, mapping to pixel space and back should return the original value within floating point tolerance. The tick monotonicity property says: for any data range and target count, the returned tick positions must be strictly increasing. The coordinate mapping property says: mapping a point at the domain minimum should land at the plot area left edge, and the domain maximum should land at the right edge.

The key practical detail with proptest is avoiding degenerate inputs. The default any f64 strategy generates NaN, infinity, negative infinity, and subnormal values. These are all valid f64 values but they produce nonsensical results for scales and coordinates. Use bounded ranges like minus 1e6 to 1e6 instead of any, and consider using the NORMAL strategy from proptest's num module which generates only finite non-zero values.

When proptest finds a failing input, it stores it in a regression file in the proptest-regressions directory. Commit these files to version control. They serve as permanent regression tests: even if proptest's random seed changes between runs, the known failing inputs will always be re-tested.

Reduce the number of test cases for expensive operations. The default is 256 cases, which is fine for fast pure functions but slow for tests that render charts. Set the number of cases to 50 or even 20 for rendering property tests using the proptest config macro attribute.

## criterion for benchmarking render performance

criterion is a statistical micro-benchmarking framework. It runs your code many times, measures the execution time, performs statistical analysis to detect regressions, and generates HTML reports with plots.

For starsight, the critical benchmarks are rendering performance at different data scales. How long does it take to render a line chart with 100 points? With 1000? With 10000? With 100000? This scaling behavior determines whether starsight is usable for real-world datasets.

Set up benchmarks by adding criterion as a dev-dependency with the html_reports feature, then creating a bench file with harness equals false in the bench directory. The benchmark function creates the test data, and the iter method runs the rendering code in a tight loop while criterion collects timing data.

The most useful criterion feature for starsight is throughput reporting. By declaring the throughput (number of data points), criterion reports not just absolute time but also time per element. This makes scaling behavior obvious: if 1000 points takes 5 milliseconds and 10000 points takes 50 milliseconds, the scaling is linear and the per-point cost is 5 microseconds.

Use iter_batched when the benchmark setup should not be timed. For example, creating the tiny-skia Pixmap and loading fonts are setup costs that should not be included in the render time measurement.

In CI, benchmark comparison is possible but tricky because GitHub-hosted runners have variable performance. A benchmark might run 10 percent faster on one run and 10 percent slower on the next due to CPU throttling or noisy neighbors. For reliable regression detection, either use self-hosted runners with dedicated hardware, or use instruction-count tools like Iai-Callgrind which count CPU instructions instead of wall time.

## cargo-flamegraph for finding rendering bottlenecks

cargo-flamegraph generates flamegraph visualizations that show where your program spends its CPU time. It works by sampling the call stack at high frequency (typically 1997 times per second) and aggregating the results into a hierarchical chart where wider boxes indicate more time.

Install it with cargo install flamegraph. On Linux, it requires the perf tool (install with sudo apt install linux-tools-generic). The critical setup step: enable debug symbols in release builds by adding debug equals true to the release profile in Cargo.toml. Without debug symbols, the flamegraph shows memory addresses instead of function names.

Create a dedicated profiling example that generates a realistic workload. For starsight, this might render a scatter plot with 100000 points to a PNG, or render a complex multi-panel chart with faceting and legends. The workload should run for at least a few seconds to collect enough samples.

Run it with cargo flamegraph minus minus example followed by the example name. The output is an SVG file that you open in a web browser. Click on boxes to zoom in on specific call stacks. Look for wide boxes near the top of the flame (these are the functions that directly consume the most CPU time) and narrow but tall stacks (these indicate deep call chains that might benefit from inlining).

For tiny-skia rendering, the typical hotspots are fill_path (filling shapes with color), stroke_path (drawing outlines), and the alpha blending pipeline. If fill_path dominates, consider reducing path complexity by pre-culling off-screen geometry. If alpha blending dominates, consider reducing the number of overlapping semi-transparent elements.

An alternative tool is samply, which opens the Firefox Profiler web UI with an interactive timeline, call tree, and flame chart. It is more powerful for exploratory profiling than static SVG flamegraphs.

## cargo-hack for testing feature flag combinations

cargo-hack is a tool for testing your project with different feature flag combinations. This is critical for starsight because the crate has 18 feature flags, and bugs can hide in specific combinations.

The most important command is cargo hack check minus minus workspace minus minus each-feature minus minus no-dev-deps. This checks the project once with each individual feature enabled, catching cases where a feature-gated module fails to compile in isolation. The no-dev-deps flag works around a Cargo issue where dev-dependencies can mask missing regular dependencies.

For more thorough testing, use the feature-powerset flag with a depth limit. cargo hack check minus minus workspace minus minus feature-powerset minus minus depth 2 tests every combination of up to two features. Full powerset testing (without a depth limit) is exponential: 18 features produce over 260000 combinations, which is not practical.

In CI, run each-feature on every PR and feature-powerset on a weekly schedule.


## Do's and don'ts for writing Rust code in starsight

These are rules born from experience in graphics libraries, Rust ecosystem conventions, and the specific needs of a visualization library. They are not abstract style preferences. Each one prevents a specific category of bug or maintenance problem.

## Do derive the standard traits on every public type

Every public struct and enum should derive Debug, Clone, and PartialEq at minimum. Debug is required for readable test failure messages and for users to println their chart configurations. Clone is required because users will want to create a chart configuration, modify it slightly, and render both versions. PartialEq is required for assertions in tests.

For types that represent values (colors, points, sizes), also derive Copy, Eq, and Hash. Copy is appropriate because these types are small (under 32 bytes) and there is no ownership semantic. Eq is appropriate because bitwise equality is meaningful for u8 color channels and f32 coordinates (with the caveat that NaN does not equal itself, but we handle that separately). Hash is needed for using colors as HashMap keys when batching draw calls by color.

For types that hold heap data (Figure, LineMark with Vec data), derive Debug and Clone but not Copy. Implement PartialEq if meaningful comparison exists.

Do not derive Default on types where the default is not useful. A default Point at zero zero is sensible. A default Figure with no data and no marks is not: it produces an empty chart with no axes and no content, which is never what anyone wants. If Default does not produce something useful, force the user to call a constructor.

## Do not use unwrap or expect in library code

The clippy configuration forbids unwrap_used and expect_used. These are panicking operations. A library should never crash the caller's program because a color string was malformed or a path was empty.

Use the question mark operator to propagate errors. Use ok_or_else to convert Option to Result. Use map_err to convert external error types to StarsightError. If an operation truly cannot fail (because you have already validated the inputs), use a comment explaining why and use the match or if-let pattern instead of unwrap.

The only permitted exception is in tests. Test code may use unwrap because a panic in a test is an expected failure mode. But even in tests, prefer the question mark operator with a test function that returns Result, because the error message from a propagated error is more informative than the generic "called unwrap on a None value" message.

## Do use the non_exhaustive attribute on every public enum and config struct

This is the single most important API design decision for a pre-1.0 library. Adding non_exhaustive to a type is a semver-breaking change. But having non_exhaustive already on a type lets you add new variants and fields in minor versions without breaking downstream code.

For enums, non_exhaustive forces downstream match statements to include a wildcard arm. When you add a new chart type or error variant in version 0.3.0, existing code that matches on the enum will not break because the wildcard handles the new variant.

For structs, non_exhaustive prevents downstream code from constructing the struct with literal syntax. This means you can add new fields with default values in minor versions. The tradeoff is that users must use a constructor function or builder pattern. This is why every config struct needs a new function or builder.

Apply non_exhaustive to: StarsightError, any ChartType enum, any RenderOptions struct, any ThemeConfig struct, any ScaleType enum. Do not apply it to pure mathematical types like Point, Vec2, Rect, and Color where the fields are the complete definition and will never change.

## Do write doc comments on every public item

This is enforced by the warn missing_docs lint. Every public function, method, struct, enum, trait, type alias, and constant needs a doc comment. The doc comment should explain what the item does, not how it is implemented. It should include an example for anything in the prelude.

Doc comments are also tests. Rust compiles and runs code blocks in doc comments as part of cargo test minus minus doc. This means your examples must compile, your imports must be correct, and your error handling must work. Use the question mark operator in doc examples and end with a hidden line containing Ok of unit type so the example compiles as a function returning Result.

A common mistake is writing doc comments that say "creates a new Point" on a function called new on the Point struct. This adds no information. Instead, describe the semantics: "creates a point at the given screen coordinates, where x increases rightward and y increases downward."

For trait methods, the doc comment should describe the contract: what the implementor must guarantee, what the caller can assume. For the DrawBackend trait, each method should document whether it is safe to call concurrently, whether it may block, and what errors it may return.

## Do not expose dependency types in the public API

If your DrawBackend trait has a method that takes a tiny_skia Point, you have coupled your public API to tiny-skia's versioning. When tiny-skia releases a breaking change, your API breaks too, even if your code is unchanged. This forces a major version bump for something you did not control.

Wrap external types in your own types. starsight has its own Point, Rect, Color, and Transform types specifically for this reason. The DrawBackend trait takes starsight types. The backend implementation internally converts to tiny-skia types. This insulates the public API from dependency churn.

The same principle applies to error types. StarsightError variants contain Strings, not tiny_skia::png::EncodingError or cosmic_text::SomeError. When a backend encounters a dependency-specific error, it wraps it in a StarsightError with a descriptive message. The dependency error type never leaks through the public API.

## Do not use println or eprintln in library code

Library code should be silent by default. The user decides what to log, when, and how. If starsight prints to stdout, it might interfere with the user's own output, corrupt pipe-based workflows, or produce unwanted noise in server environments.

Use the log crate for diagnostic messages. The log crate's macros (trace, debug, info, warn, error) produce no output unless the user installs a log subscriber. If no subscriber is installed, all log calls are compiled away to nothing.

For development debugging, use the tracing crate, which is compatible with log but adds structured data and spans. But do not add tracing as a required dependency. Use it behind a feature flag or just use log for the initial release.

## Do not panic in library code

Panics unwind the stack and crash the program (or abort, depending on the panic strategy). A library must never do this. If a rendering backend cannot allocate a pixel buffer because the dimensions are too large, it should return StarsightError::Render, not panic.

The only permitted panic is in the show method, which attempts to display a chart in a window. If no window system is available (no GPU, no display server, running in a headless CI environment), show has no meaningful fallback. In this specific case, a panic with a clear message ("no display backend available; use save() instead of show()") is acceptable. Even here, prefer returning an error and letting the caller decide.

The clippy configuration should be set to forbid panics in library code. The allow-panic-in-tests option lets test code panic (which is normal for tests).

## Do use feature flags for every optional dependency

The user who writes cargo add starsight should get a working library with CPU rendering, SVG output, and PNG export. They should not be forced to compile wgpu, polars, ratatui, nalgebra, or any other heavyweight dependency they do not need.

Every optional dependency goes behind a feature flag. The feature flag is defined in the starsight facade crate's Cargo.toml and forwarded to the appropriate layer crate. When the user enables the gpu feature, the facade crate enables the gpu feature on starsight-layer-1, which activates the wgpu dependency and compiles the wgpu backend code.

Feature flags must be additive. Enabling a feature must never remove functionality. A crate compiled with all features enabled must work exactly the same as one compiled with the default features, plus additional capabilities. This means feature flags should never be used for exclusive choices (either wgpu or tiny-skia, but not both). Both backends are always available; the user chooses at runtime which to use.

## Do use impl Into for string parameters

Any function that takes a string should accept impl Into String, not String or and str. This lets the user pass a string literal (which converts via From str for String) or an owned String (which converts via the identity From impl) without explicit conversion.

The same pattern applies to color parameters: accept impl Into Color, which enables passing a chromata Color, a prismatica Color, or a raw Color value. It applies to path parameters: accept impl AsRef Path, which enables passing a string literal, a PathBuf, or a Path reference.

The general rule: accept the most general type that does not lose information. impl Into String accepts both borrowed and owned strings. and str accepts only borrowed strings, which forces the function to clone if it needs to own the string. String accepts only owned strings, which forces the caller to allocate even when a static string literal would suffice.

## Don'ts for documentation

Do not write doc comments that restate the function name. If the function is called width on Rect, do not write "returns the width of the rect." Write "the horizontal extent of the rectangle: right minus left. Always non-negative for a valid rect."

Do not use generic verbs like "handles" or "processes" or "manages." These words are meaningless. Instead of "handles the rendering of paths," write "converts path commands to tiny-skia Path objects and strokes them onto the pixmap."

Do not omit units. If a function takes a font_size parameter, the doc comment must say whether it is in pixels, points, or ems. If a function returns a duration, say whether it is in seconds, milliseconds, or nanoseconds. If a function takes an angle, say whether it is in degrees or radians.

Do not hide important behavior in long paragraphs. If a function has a surprising behavior (like clamping out-of-range values instead of returning an error), document it in the first sentence, not the fourth paragraph.

Do not copy-paste doc comments. If two functions have similar behavior, write distinct doc comments that explain the differences. If a method on Vec2 does the same thing as a method on Point, reference the other method with a see-also link rather than duplicating the text.

## Don'ts for code structure

Do not put multiple public types in one file. Each public struct, enum, or trait should be in its own file or, for closely related types, in a small module with a clear name. A file called types.rs that defines Point, Vec2, Rect, Size, Color, ColorAlpha, Transform, and PathStyle is too large. Split it into geom.rs (Point, Vec2, Rect, Size), color.rs (Color, ColorAlpha), transform.rs (Transform), and path.rs (PathStyle, PathCommand).

Do not create deep module nesting. Three levels is the maximum: starsight-layer-1, backend, skia. A path like starsight layer 1 backend skia raster headless is too deep. Flatten the hierarchy by using more descriptive module names at a shallower depth.

Do not create circular dependencies between modules within a crate. If scale.rs needs types from axis.rs and axis.rs needs types from scale.rs, extract the shared types into a common module that both depend on.

Do not use glob re-exports (pub use crate star) in library code. Glob re-exports hide the origin of types and make it hard for users to find documentation. Explicitly list every re-exported type.

## Don'ts for API design

Do not return mutable references from builder methods if the builder will be consumed later. If the Figure builder returns and mut Self from title() but then save() takes self by value, the user has to call save on a temporary, which is syntactically awkward. Either make all methods take and mut self and have save take and self, or make all methods take self by value and have save also take self by value.

Do not use type aliases to hide complexity. If a function returns Result of Vec of Box dyn Mark plus Send plus Sync, StarsightError, do not create a type alias MarkList that hides the Box dyn part. Users need to see the boxed trait object to understand the ownership and dynamic dispatch implications. Type aliases are appropriate for Result T StarsightError (because every function in the crate uses this pattern) but not for application-specific composed types.

Do not add a method to a trait when a free function or a blanket impl would work. Every method on the DrawBackend trait requires every backend to implement it. If a method can be implemented in terms of other trait methods (like drawing a dashed rect by drawing four dashed lines), provide a default implementation so backends get it for free.

## How to write good commit messages

starsight uses Conventional Commits. Every commit message starts with a type, an optional scope, and a description. The type determines how git-cliff categorizes the commit and how cargo-release determines the version bump.

The type feat indicates a new feature. It maps to a minor version bump under semver. The type fix indicates a bug fix. It maps to a patch version bump. The type perf, refactor, docs, test, and chore are informational and do not trigger version bumps. The type feat with an exclamation mark (feat bang) or a BREAKING CHANGE footer indicates a breaking change, which maps to a major version bump (or minor in pre-1.0).

The scope is the area of the codebase affected. For starsight, useful scopes are layer-1, layer-2, primitives, scale, backend, skia, svg, tick, and ci. The scope appears in parentheses after the type: feat layer-2 colon implement log scale.

The description is imperative mood, lowercase, no period. "add linear scale support" not "added linear scale support" and not "adds linear scale support." The description should complete the sentence "this commit will" followed by the description.

Bad commit messages: "fix stuff", "wip", "updates", "more changes." These tell you nothing about what changed or why. Good commit messages: "fix layer-2 colon correct Y axis inversion in CartesianCoord," "feat layer-1 colon implement SVG backend fill_rect," "test layer-1 colon add snapshot for blue rect on white."

## How to write a good CONTRIBUTING.md

The CONTRIBUTING.md file tells potential contributors how to participate. For starsight, it should cover: how to set up the development environment (clone, install Rust 1.85, run cargo check), the branching strategy (fork plus PR to main), the commit message format (Conventional Commits), the PR checklist (tests pass, clippy clean, snapshot update, changelog entry), and the code review process.

It should also mention the tools contributors need: cargo-insta for snapshot review, cargo-deny for license checking, and optionally cargo-nextest for faster tests. Do not require contributors to install every tool in the toolchain. The CI catches anything they miss.

The tone matters. Contributors are donating their time. Thank them for their interest. Make the setup steps as simple as possible. Provide a one-command way to verify that everything works (cargo test minus minus workspace).

## How to handle deprecation before 1.0

In a pre-1.0 library, every minor version bump (0.1 to 0.2) can contain breaking changes. But you should still deprecate rather than abruptly remove. This gives users a migration path.

The deprecated attribute takes a since field and a note field. The since field is the version where the deprecation was introduced. The note field tells the user what to use instead.

When you deprecate a function, keep it working for at least one release. In version 0.2.0, deprecate old_function with a note saying "use new_function instead." In version 0.3.0, remove old_function. This gives users one release cycle to update their code.

In the changelog, list deprecated items under the Deprecated heading and removed items under the Removed heading. This makes migration clear.

## How to use non_exhaustive correctly

The non_exhaustive attribute has different effects on enums and structs, and you need to understand both.

On an enum, non_exhaustive means downstream match statements must include a wildcard arm. The enum can be constructed and pattern-matched within the crate, but externally, exhaustive matching is not possible. This lets you add variants in minor versions.

On a struct, non_exhaustive means the struct cannot be constructed with literal syntax outside the crate, and fields cannot be exhaustively destructured. You can still access fields by name (if they are public), but you cannot write MyStruct with field1 colon value1 comma field2 colon value2. This lets you add fields with default values in minor versions.

Where to use it: on StarsightError (you will add new error variants as development progresses), on any configuration struct (you will add new options), on any enum of chart types or scale types or coordinate types.

Where not to use it: on mathematical value types like Point, Vec2, Rect, and Color. These types have a fixed set of fields that are their complete mathematical definition. A Point is two coordinates, forever. Adding a non_exhaustive attribute would force users to use a constructor instead of struct literal syntax, which is unnecessarily verbose for a type that is constructed hundreds of times in rendering code.

## Understanding the Rust API Guidelines checklist

The Rust API Guidelines is an official document maintained by the Rust project. It lists conventions that well-designed Rust libraries follow. You should read the checklist version before starting serious implementation work and revisit it before each release.

The key items for starsight: types eagerly implement common traits (Debug, Clone, Display, Send, Sync). Conversions use standard traits (From, Into, TryFrom, TryInto, AsRef). Error types implement std Error. Iterators are lazy and composable. Builder methods take and mut self and return and mut Self (or take self and return Self). Public dependencies are re-exported. Feature flags are additive and well-documented.

The guidelines also cover naming: conversion methods follow the as, to, into prefix conventions. as indicates a cheap borrow (as_slice), to indicates an expensive copy (to_string), into indicates an ownership transfer (into_vec). starsight's to_tiny_skia method follows this convention: it creates a new tiny_skia Color from an existing starsight Color.

## Understanding Cargo semver rules for pre-1.0 crates

Cargo's semver for pre-1.0 crates is different from post-1.0. For crate version 0.x.y: the x is the "major" version, the y is the "minor" version, and there is no "patch" in the semver sense. Bumping from 0.1.0 to 0.2.0 is a major change (breaking API permitted). Bumping from 0.1.0 to 0.1.1 is a minor change (no breaking API).

This means: a user who depends on version 0.1 (which Cargo interprets as greater than or equal to 0.1.0 and less than 0.2.0) will automatically receive 0.1.1 and 0.1.2 but not 0.2.0. If you accidentally publish a breaking change in 0.1.1, you break everyone who depends on 0.1 without their opt-in.

The practical implication: be very careful with patch releases in pre-1.0. Use cargo-semver-checks before every publish, including patch releases.

## How insta handles binary snapshots in detail

When you call the binary snapshot assertion macro with a dot png extension and a Vec of u8, insta does the following. It computes a hash of the binary data. It checks if a snapshot file with the test name exists in the snapshots directory. If it does, it reads the stored binary and compares it byte for byte. If the data matches, the test passes. If it differs, or if no snapshot exists, the test fails and insta writes the new data to a dot snap dot new file.

The snapshot directory contains two files per binary snapshot: a dot snap metadata file (which stores the snapshot name, the assertion location, and the binary file reference) and the actual binary file (like a dot png file).

When you run cargo insta review, the tool shows you both the old and new binary files. For PNG images, it cannot display them in the terminal, but it tells you the file paths so you can open them in an image viewer. Some terminal editors and diff tools support image display.

The critical insight for starsight: binary snapshots are exact. A single pixel difference causes a failure. This is what you want for a deterministic CPU renderer: if the output changed, something in the code changed, and you need to review it. If you use a GPU renderer (where floating point rounding varies by driver), binary snapshots will flake. This is why all snapshot tests must use the tiny-skia backend.

## How to structure tests in a workspace

Unit tests go in the same file as the code they test, in a tests module at the bottom of the file guarded by cfg test. This is the standard Rust convention. Unit tests have access to private functions and fields, which is useful for testing internal algorithms.

Integration tests go in a tests directory at the crate root. Each file in the tests directory is compiled as a separate crate. Integration tests can only access the crate's public API. For starsight, integration tests are useful for testing the full rendering pipeline: create data, create a Figure, render to PNG, assert the output.

Snapshot tests are a special case of integration tests. They live in the tests directory and use insta's assertion macros. Each snapshot test produces a reference file in a snapshots directory.

Doc tests live in doc comments and are run by cargo test with the doc flag. They serve double duty as documentation and as tests. For starsight, the most important doc tests are in the prelude module and on the Figure type, because these are the first things users see.

For the starsight workspace, each layer crate has its own tests. starsight-layer-1 tests the primitives, the tiny-skia backend, and the SVG backend. starsight-layer-2 tests scales, ticks, and coordinate mapping. starsight-layer-3 tests marks. The facade crate has integration tests that exercise the full pipeline through the public API.

## How to handle floating point comparison in tests

Floating point arithmetic is not exact. The expression 0.1 plus 0.2 does not equal 0.3 in IEEE 754. For starsight, this matters in every test that involves scale mapping, coordinate conversion, or color interpolation.

Never use assert_eq with floating point values. Use approximate comparison instead. The simplest approach is to compute the absolute difference and check that it is less than a tolerance. For f32, a tolerance of f32 EPSILON (about 1.2e-7) is appropriate for single operations. For accumulated operations (like mapping through a scale and back), a larger tolerance of 1e-5 is more realistic.

The approx crate provides assert_abs_diff_eq and assert_relative_eq macros that handle this cleanly. However, adding a dependency just for tests is debatable. A simple helper function that returns a bool is often sufficient.

For color operations, floating point precision manifests as off-by-one errors in u8 channels. The lerp of Color 0 and Color 255 at t equals 0.5 might produce 127 or 128 depending on the rounding method. Decide on a rounding convention (round-half-up, round-half-even, or truncate) and document it. Then test for exact equality on the u8 values, not approximate equality on the f32 intermediates.

## How the workspace lints configuration works

The workspace Cargo.toml has a workspace lints section that applies to all member crates. Each crate opts in by adding lints workspace equals true to its own Cargo.toml. This ensures consistent lint configuration across all nine crates.

For starsight, the workspace lints forbid unsafe_code (across the entire workspace) and enable clippy pedantic at the warn level. Pedantic clippy is aggressive: it warns about many things that are technically fine but could be clearer. Some pedantic lints are too noisy and should be allowed. Common ones to allow: clippy module_name_repetitions (which fires when a type name contains the module name, like scale Scale), clippy must_use_candidate (which fires on every function that returns a value), and clippy missing_errors_doc (which fires on every function returning Result).

Allow specific lints in the code with allow attributes, not in the workspace configuration. This keeps the workspace configuration strict and makes exceptions visible at the point of use.

## How to profile tiny-skia rendering specifically

When starsight renders slowly, the bottleneck is almost certainly in tiny-skia's rasterization. The rendering pipeline in tiny-skia has three phases: path processing (tessellation), pixel filling (rasterization), and alpha blending (compositing).

To identify which phase is slow, generate a flamegraph (using cargo-flamegraph as described in the tools section) and look for these function signatures: fill_path and stroke_path are the entry points, path_to_stroke and path_to_fill handle tessellation, raster_pipeline and pipeline execute handle per-pixel work, and blend_src_over handles compositing.

Common performance issues and fixes. Many small paths: batching multiple shapes into a single path (using push_circle instead of separate from_circle calls) reduces the per-path overhead. Anti-aliasing on axis-aligned lines: disable anti-aliasing for horizontal and vertical lines by setting paint dot anti_alias to false, which avoids the sub-pixel blending cost. Large transparent areas: premultiplied alpha blending still costs per-pixel even when the alpha is very low; reduce the area of transparent fills. Text rendering: creating a FontSystem for every text draw call is extremely expensive; reuse it across the entire rendering pass.

## Understanding the difference between stable and unstable clippy lints

Clippy has four lint groups: correctness (on by default, catches bugs), style (on by default, enforces conventions), complexity (on by default, simplifies code), and pedantic (off by default, enforces stricter conventions). There are also restriction (off by default, potentially controversial) and nursery (off by default, may have false positives) groups.

starsight enables pedantic at the warn level, which is more aggressive than most Rust projects. This catches things like missing documentation, overly complex boolean expressions, and unnecessary closures. It also produces many warnings that are correct but noisy: the must_use_candidate lint fires on every function that returns a value, suggesting you add the must_use attribute. For builder methods that return Self (where the return value is always used), this is just noise.

The strategy is: enable pedantic globally, then selectively allow specific lints either in the workspace configuration (for truly noisy lints) or with per-function allow attributes (for specific exceptions). Document why each allow is necessary.

The restriction group contains lints that are too opinionated for most projects but useful for specific cases. For starsight, the most useful restriction lints are: print_stdout (catches accidental println), unwrap_used (catches accidental unwrap), and dbg_macro (catches leftover debug macros).

## How wasm-bindgen and web-sys work for the WASM target

When starsight compiles to WebAssembly for browser deployment, it uses wasm-bindgen to bridge between Rust and JavaScript. wasm-bindgen generates JavaScript glue code that converts between Rust types and JavaScript types. web-sys provides bindings to browser Web APIs like the Canvas API and WebGPU.

The WASM target is feature-gated behind the web flag. When enabled, the wgpu backend uses WebGPU (the browser's native GPU API) instead of Vulkan or Metal. The rendering code is the same; only the GPU backend initialization differs.

For the WASM target, starsight needs to handle several differences from native: there is no filesystem (save functions write to an in-memory buffer and trigger a browser download), there is no windowing system (the chart renders into an HTML Canvas element), and font loading works differently (system fonts are not available; fonts must be bundled or loaded from URLs).

This is all planned for version 0.10.0 and should not distract from the native rendering pipeline. But the architecture accommodates it: the DrawBackend trait is generic enough that a WebGPU backend can implement it without changes to the marks, scales, or figure layers.

## How the docs.rs configuration works

When you publish a crate to crates.io, docs.rs automatically builds the documentation and hosts it. The configuration in Cargo.toml under package.metadata.docs.rs controls how the documentation is built.

The most important setting is all-features equals true, which ensures that feature-gated items appear in the documentation. Without this, the docs would show only the default feature set, missing the GPU, terminal, Polars, and other optional functionality.

The cfg_attr docsrs feature doc auto_cfg attribute in lib.rs automatically annotates feature-gated items with a badge showing which feature enables them. This is how users discover that the wgpu backend requires the gpu feature.

For workspace crates, each crate needs its own docs.rs metadata section because docs.rs builds each crate independently. The facade crate should have the most comprehensive configuration since it is the primary documentation entry point.

## What makes a good example program

Example programs serve three purposes: they are documentation (showing users how to use the API), they are integration tests (validating that the API actually works), and they are gallery material (generating images for the README and docs).

A good example is self-contained: it does not require external data files, network access, or special environment setup. It produces visible output: a PNG file, an SVG file, or a terminal display. It demonstrates one concept clearly rather than cramming many features into one program.

For starsight, the example set should cover: quickstart (the simplest possible chart in five lines), scatter (scatter plot with color and size aesthetics), statistical (box plot or violin plot), surface3d (3D surface with a colormap), terminal (rendering in the terminal), interactive (windowed chart with hover and zoom), polars_integration (loading data from a DataFrame), streaming (real-time updating chart), faceting (faceted scatter plot), custom_theme (applying a chromata theme), recipe (defining a custom chart type), and gallery (generating all example charts for documentation).

Each example file should start with a doc comment explaining what it demonstrates. The main function should be short and readable. Move data generation into helper functions so the chart-creation code stands out.

## How to manage breaking changes across workspace crates

When you change a type in starsight-layer-1 that is used by starsight-layer-3, you have created a cross-crate breaking change. Both crates need to be updated and released together.

starsight uses lockstep versioning: all nine crates share the same version number, defined in the workspace Cargo.toml. When any crate has a breaking change, all crates bump their version together. This simplifies dependency management and avoids version matrix problems where layer-3 version 0.2.0 is incompatible with layer-1 version 0.1.0.

The downside of lockstep versioning is that a change in one crate forces a new version of all crates. Users who depend on starsight-layer-2 and never use anything from layer-7 still see a new version of layer-7 appear. This is the accepted tradeoff: simplicity of the release process outweighs the minor inconvenience of version churn.

## How to read compiler errors in a workspace context

When cargo check reports an error in a workspace, the error message includes the crate name in the file path. An error in starsight-layer-1 slash src slash primitives.rs is obviously in layer 1. But when the error involves cross-crate types, the messages can be confusing.

If you get a "trait not implemented" error, check which crate defines the trait and which crate defines the type. If both are yours, add the impl in whichever crate owns the type (Rust's orphan rule requires the impl to be in the crate that defines either the trait or the type, or both).

If you get a "type mismatch" error where two types look identical but the compiler says they are different, check if you have accidentally imported the type from the wrong crate. In a workspace, the same struct name might exist in multiple crates. The compiler treats them as different types even if their definitions are identical.

If you get a "feature not enabled" error, check the Cargo.toml of the crate being compiled. Feature flags must be explicitly forwarded through each crate in the dependency chain.

## How cfg attributes work for feature gating

Feature gating in Rust uses the cfg attribute. The attribute cfg feature equals terminal on a module declaration means the module is only compiled when the terminal feature is enabled. The attribute cfg_attr feature equals terminal comma derive Serialize means the derive is only added when the terminal feature is enabled.

In starsight, feature gating happens at the module level, not at the function level. If the terminal feature is disabled, the entire backend terminal module is excluded from compilation. This is cleaner than scattering cfg attributes throughout individual functions.

The cfg not syntax is used to provide compile-time error messages. If someone tries to use a function that requires the gpu feature without enabling it, a cfg not gpu block can define a stub function that fails with a clear error message at compile time rather than a confusing "module not found" error.

Feature flags must be documented. Each feature in Cargo.toml should have an inline comment explaining what it enables and which dependencies it pulls in. The README should have a feature flags table with the same information.

## How to think about the MVP for 0.1.0

The exit criteria for 0.1.0 is: plot exclamation of array 1.0, 2.0, 3.0 comma array 4.0, 5.0, 6.0 dot save "test.png" produces a correct line chart. This is not a full visualization library. It is the minimum vertical slice that proves the architecture works.

To get there, you need: the primitive types (Point, Vec2, Rect, Color, Transform), the tiny-skia backend (creating a pixmap, drawing paths, filling rects, rendering text, saving PNG), the SVG backend (at least fill_rect and save_svg), a linear scale, the Wilkinson tick algorithm, a Cartesian coordinate system, axis rendering (tick lines, tick labels, axis labels), a line mark, the Figure builder, the plot macro, and snapshot tests proving it all works.

You do not need: log scales, categorical scales, bar charts, histograms, box plots, faceting, legends, GPU rendering, terminal rendering, interactivity, streaming data, PDF export, WASM, Polars integration, ndarray, Arrow, or any of the 60 chart types beyond basic lines and points.

Resist the temptation to add features before the vertical slice is complete. A library that renders one chart type correctly and has tests is more valuable than a library that has stubs for 60 chart types and renders nothing.

## How to stay motivated on a large solo project

Building a visualization library is a multi-year project. You will not finish in a weekend or a month. The scope described in this document is enormous: 66 chart types, 5 rendering backends, GPU acceleration, terminal rendering, WASM, interactivity, streaming data.

The key is momentum. Commit every day, even if it is just a small refactor or a single test. Each commit is visible progress. The git history tells the story of continuous forward motion, which is motivating both for you and for potential contributors who are evaluating whether the project is alive.

Focus on the vertical slice first. Get plot save to produce a PNG. Then get it to produce an SVG. Then add a second chart type. Then add axis labels. Then add colors. Each addition is a visible improvement that you can share and celebrate.

Do not worry about the later milestones (GPU, WASM, 3D) until the foundation is solid. The layer architecture ensures that later work adds to the codebase without restructuring it. Layer 6 (interactivity) builds on top of layers 1 through 5 without modifying them. You can add GPU rendering at 0.6.0 without touching the marks, scales, or figure code that has been stable since 0.1.0.

Share your progress publicly. Post screenshots of rendered charts. Write blog posts about the tick algorithm or the tiny-skia integration. Show benchmarks comparing render times to plotters. Public visibility attracts contributors and creates accountability.

## How testing should evolve through the milestones

At 0.1.0, you need: unit tests for every method on Point, Vec2, Rect, Color, and Transform. Unit tests for LinearScale map and inverse. Unit tests for extended_ticks. One snapshot test for a rendered line chart. One snapshot test for a rendered SVG.

At 0.2.0, add: snapshot tests for bar, area, histogram, and heatmap charts. Property tests for scale roundtrips.

At 0.3.0, add: snapshot tests for all statistical chart types. Reference tests comparing output to known-good values from matplotlib (render the same data in both, compare visually).

At 0.4.0, add: layout tests verifying that faceted charts have the correct number of panels, that legends have the correct number of entries, that colorbars span the correct range.

At 0.5.0, add: property tests for all scale types (log scale roundtrip, symlog symmetry, categorical scale bijectivity).

At 0.6.0, add: interaction tests that simulate mouse events and verify that zoom, pan, and selection produce correct results.

The test suite should grow proportionally to the code. A crate with 1000 lines of code should have at least 500 lines of tests. Coverage should stay above 80 percent. Run cargo-mutants periodically to find untested code paths.

## What the CI pipeline looks like at each stage

At 0.1.0, CI runs: cargo fmt check, cargo clippy, cargo test on three platforms (Linux, macOS, Windows) and two Rust versions (stable and 1.85), cargo insta test with check, cargo deny check, cargo doc with no-deps. This is what the current ci.yml already does.

At 0.3.0, add: cargo-semver-checks on PRs (once the crate is published to crates.io), coverage reporting with cargo-llvm-cov.

At 0.5.0, add: cargo-hack each-feature check to verify all feature combinations compile, WASM target check (cargo build with target wasm32-unknown-unknown and the web feature).

At 0.8.0, add: terminal rendering smoke tests (if possible in CI; terminal protocol tests may need a pseudo-terminal).

At 1.0.0, add: cargo-release dry-run in PRs to verify the release process works, full feature-powerset testing.

Keep CI fast. The total CI time should be under 15 minutes. If it exceeds that, split slow jobs (coverage, mutation testing, feature-powerset) into a separate workflow that runs on a schedule rather than on every PR.

## What happens after 1.0.0

Once starsight reaches 1.0.0, the rules change. Semver becomes strict: patch versions (1.0.1) may only fix bugs. Minor versions (1.1.0) may add new features but must be backward-compatible. Major versions (2.0.0) may break the API.

In practice, a well-designed 1.0.0 API rarely needs a 2.0.0. The non_exhaustive attribute, the builder pattern, and feature flags all provide extension points that accommodate new functionality without breaking changes. New chart types are added as new Mark implementations. New backends are added as new DrawBackend implementations. New scales, color palettes, and themes are added as new types behind feature flags. None of these require breaking the existing API.

The main risk of needing a 2.0.0 is a fundamental change to the trait design: if the DrawBackend trait needs a new required method, every backend breaks. This is mitigated by providing default implementations on all trait methods from the start. A new method with a sensible default does not require backend authors to update their code immediately.

Post-1.0.0, the focus shifts from API design to performance optimization, chart type coverage, and ecosystem integration. The architecture you build now determines whether that shift is smooth or painful. Invest in the right abstractions now, and the years after 1.0.0 will be spent adding features, not fighting the architecture.

## How Cargo.toml workspace inheritance actually works

Workspace inheritance is the mechanism that lets you define common metadata once in the root Cargo.toml and reference it from member crates. When you write version.workspace equals true in a member crate's Cargo.toml, Cargo reads the version field from the workspace.package section of the root.

The fields that can be inherited are: version, edition, description, license, authors, repository, documentation, readme, keywords, categories, publish, and rust-version. Each member crate chooses which fields to inherit and which to override. For starsight, all crates inherit version and edition (so they stay in sync), but each has its own description (because "Layer 1: Rendering abstraction" is more useful than the project-level description in a search result).

Dependencies can also be inherited. The workspace.dependencies section in the root Cargo.toml defines dependency versions once. Member crates reference them with dependency.workspace equals true. This ensures all crates use the same version of tiny-skia, thiserror, and every other shared dependency.

The key constraint: workspace dependency inheritance works for dependencies in the dependencies, dev-dependencies, and build-dependencies sections, but the member crate can override features. If the workspace declares tiny-skia at version 0.12.0, a member crate can reference it with workspace equals true and add features equals png to enable additional features for just that crate.

Lints can also be inherited. The workspace.lints section defines clippy and rustc lint levels that all crates share. Each crate opts in with lints.workspace equals true. This is how starsight enforces unsafe_code equals forbid and clippy pedantic equals warn across all nine crates.

Profile settings (release and dev) are always workspace-level. You cannot have different optimization settings for different member crates within the same profile. The opt-level of 1 in the dev profile applies to all starsight crates.

## How the workspace resolver interacts with features

Resolver version 3 (implied by edition 2024) changes how Cargo handles features in a workspace. When you build or test the workspace, Cargo creates a single dependency graph for all member crates and unifies their feature requirements. If starsight-layer-7 enables the terminal feature and starsight-layer-2 does not, the terminal feature is still enabled in the unified graph because some crate needs it.

This feature unification is a source of subtle bugs. If you test starsight-layer-2 in the workspace context, it compiles with the terminal feature enabled even though it does not depend on terminal functionality. This means a test might pass in the workspace but fail when starsight-layer-2 is compiled standalone. The mitigation is to test individual crates with cargo test minus p starsight-layer-2 during development, and to use cargo-hack with the each-feature flag in CI to verify that each feature compiles independently.

Resolver 3 adds MSRV-aware resolution on top of resolver 2's feature behavior. If a dependency's latest version requires Rust 1.90 but your declared rust-version is 1.85, Cargo will select an older version of the dependency that is compatible with 1.85. This only applies to user-initiated resolution (cargo update, adding new dependencies) and does not retroactively change existing Cargo.lock entries.

## How Cargo.lock works in a workspace

The Cargo.lock file records the exact versions of every dependency in the workspace. It lives at the workspace root and applies to all member crates. When you run cargo build, Cargo reads the lock file and uses exactly those versions, ignoring newer versions that might be available on crates.io.

For libraries (which starsight is), the convention is to not commit Cargo.lock to version control. This is because downstream users will use their own Cargo.lock with potentially different dependency versions, and your lock file does not help them. However, some library projects do commit Cargo.lock for CI reproducibility. The starsight gitignore currently excludes it with the star dot lock pattern. If you want reproducible CI builds, remove that pattern and commit the lock file.

The cargo update command refreshes the lock file to the latest compatible versions. You should run this periodically (weekly or before each release) to pick up bug fixes and security patches in dependencies.

## How to think about compile times in a nine-crate workspace

Compile time is the tax you pay on every code change. In a nine-crate workspace, the tax is higher because Cargo must compile each crate separately, check dependencies between them, and potentially recompile downstream crates when an upstream crate changes.

The good news: Cargo's incremental compilation means that changing a file in starsight-layer-3 only recompiles layer-3 and the crates that depend on it (layers 4 through 7 and the facade). It does not recompile layers 1 and 2 because they are upstream.

The bad news: some dependencies are heavy. tiny-skia compiles reasonably fast. cosmic-text with its font loading infrastructure is slower. wgpu, polars, and nalgebra are very slow. This is why they are behind feature flags: the default build only compiles tiny-skia, thiserror, and the svg crate, which is relatively fast.

Strategies to reduce compile time: use the Swatinem rust-cache action in CI (already configured). Set opt-level to 1 in the dev profile (already configured, gives tiny-skia reasonable performance without full optimization). Use cargo check instead of cargo build during development (skips code generation and linking). Consider using mold (on Linux) or lld (on all platforms) as the linker, which is significantly faster than the default linker for large projects.

## How to use the xtask pattern effectively

The xtask pattern is a convention where a binary crate in the workspace provides development automation tasks. The binary is never published. It is invoked via cargo run or, with a Cargo alias, via cargo xtask.

To set up the alias, create a dot cargo directory at the workspace root containing a config.toml file. In that file, define an alias: xtask equals the string run minus minus manifest-path xtask/Cargo.toml minus minus. Now cargo xtask gallery runs the xtask binary with the gallery argument.

For starsight, the xtask crate will handle: gallery generation (running all example programs, collecting their PNG and SVG output, resizing them for thumbnails, and writing an HTML gallery page), benchmark orchestration (running criterion benchmarks and generating comparison reports), documentation generation (running cargo doc and copying the output to a docs directory for GitHub Pages), and release preparation (running all checks, generating the changelog, and printing the release commands).

The xtask main function parses command-line arguments with clap and dispatches to subcommand functions. Each subcommand is a function that runs shell commands using std process Command, reads and writes files, and prints status messages.

The advantage of xtask over shell scripts is that it is cross-platform (works on Windows without bash), type-checked (compile-time errors instead of runtime failures), and can share code with the main library (reusing types and functions).

## How to choose between generic and concrete types in the API

A common question when designing a Rust API is whether to make a function generic. For starsight, the answer depends on the layer.

In layer 1 (rendering), use concrete types. The DrawBackend trait methods take specific types: Path, PathStyle, Color, Rect, Point. Making them generic would add complexity without benefit. There is one Path type, one Color type, one Rect type. The backend implementations need to know exactly what they are receiving.

In layer 3 (marks), use trait objects where needed. The Mark trait is object-safe and marks are stored as Box dyn Mark in the Figure. This allows different mark types (LineMark, PointMark, BarMark) to coexist in the same marks vector without generics.

In layer 5 (high-level API), use generics on entry points. The data acceptance functions should accept impl Into DataSource, which enables passing a Polars DataFrame, a pair of slices, or an ndarray without the user explicitly converting. The builder methods should accept impl Into String for labels and titles.

The general rule: concrete types at the bottom (where implementation details matter), generic types at the top (where user ergonomics matter), trait objects in the middle (where heterogeneous collections are needed).

## How Display and Debug differ and when to implement each

Debug is for developers. It shows the internal structure of a value in a way that is useful for debugging. Derive it on every type. The derived implementation shows the struct name and all field values.

Display is for users. It shows a value in a human-readable format suitable for output. Implement it manually on types that have a natural textual representation. Color should display as a hex string like hash ff8000. Point should display as parenthesized coordinates like (100.0, 200.0). Rect should display as its bounds. StarsightError already derives Display through thiserror.

Not every type needs Display. Internal types like PathStyle, SceneNode, and SkiaBackend do not have a natural textual form. Implementing Display on them would be misleading because the output would be arbitrary, not meaningful. Debug is sufficient.

For the prelude types (Figure, Color, Point), Display is expected because users will format them into error messages and log lines. For the internal types, Debug is sufficient.

## How Send and Sync affect the architecture

Send means a value can be transferred between threads. Sync means a value can be shared (by reference) between threads. Most Rust types are automatically Send and Sync if all their fields are Send and Sync.

For starsight, Send and Sync matter for two reasons. First, users might want to render charts on a background thread to avoid blocking the UI. Second, the wgpu backend requires Send plus Sync for GPU resources.

The tiny-skia Pixmap type is Send but not Sync (it contains mutable state). This means you can move a SkiaBackend to another thread, but you cannot share it between threads without a mutex. This is fine for starsight's architecture because the rendering pipeline is sequential: build the scene, then render it. There is no need for concurrent access to the backend.

The Scene type should be Send and Sync because it is immutable data. Once built, it can be shared between threads. This enables a pattern where the scene is built on one thread and rendered on another.

The Figure type should be Send but does not need to be Sync because it is a builder that accumulates mutable state. You build a Figure on one thread and render it on the same thread or move it to another.

Make sure all public types are Send by default. Check with a compile-time assertion: const _ colon fn of unit where Figure colon Send equals open close curly braces. This is a zero-cost way to verify Send bounds.

## How memory allocation works in the rendering pipeline

Understanding allocation patterns helps you avoid unnecessary copies and heap pressure during rendering.

The Scene is allocated on the heap. The Vec of SceneNode grows as marks emit drawing commands. Each SceneNode variant contains owned data: paths own their command vectors, text nodes own their strings. This means scene construction allocates memory proportional to the complexity of the chart.

The Pixmap in the tiny-skia backend is a large contiguous allocation: width times height times four bytes (for RGBA). For an 800 by 600 chart at 300 DPI (2400 by 1800 pixels), this is about 17 megabytes. The allocation happens once when the backend is created and the memory is reused for the entire rendering pass.

The cosmic-text FontSystem allocates when loading fonts. On a typical system, loading all system fonts takes 10 to 50 megabytes of memory. This is why FontSystem must be a long-lived object: creating it for every text draw call would allocate and deallocate this memory repeatedly.

Path construction allocates for the Vec of PathCommand. For a line chart with N points, this is proportional to N. For a scatter plot with N circles, this is proportional to N times the number of segments per circle (typically 4 cubic bezier arcs, so 16 commands per circle).

The PNG encoding allocates a buffer for the compressed output. The size depends on the image content: highly compressible charts (solid backgrounds, few colors) compress well, while detailed charts with gradients compress less.

For 0.1.0, do not optimize memory allocation. Use straightforward Vec and String allocations. Profile first (with cargo-flamegraph), optimize only the hotspots. The most likely optimization targets after profiling are: reusing Path buffers between draw calls (instead of allocating new Vecs for each path), pre-allocating the Scene vector to the expected number of nodes, and caching shaped text (so the same tick label is not shaped repeatedly if it appears on multiple axes).

## How to handle DPI and physical versus logical pixels

Charts need to render at different resolutions depending on the output target. A screen display might be 96 DPI. A retina display is 192 DPI. A print PDF is 300 DPI. A poster is 600 DPI.

starsight separates logical size from physical size. The user specifies the chart size in logical pixels (800 by 600). The rendering pipeline multiplies by a scale factor to get physical pixels. A scale factor of 1.0 gives 800 by 600 physical pixels (for screen). A scale factor of 3.75 gives 3000 by 2250 physical pixels (for 300 DPI print at the same logical size).

Font sizes, line widths, and point radii are all specified in logical units and scaled by the same factor. A 12-pixel font at scale factor 1.0 is 12 physical pixels. At scale factor 3.75, it is 45 physical pixels. This ensures charts look the same at all resolutions, just sharper at higher DPI.

The tiny-skia backend creates the Pixmap at the physical size and applies a Transform that scales all drawing operations by the scale factor. This is transparent to the marks and layout system, which always work in logical coordinates.

For SVG output, DPI does not apply because SVG is resolution-independent. The viewBox is set to the logical size, and the SVG renderer handles scaling to the display resolution.

## How color spaces affect rendering correctness

Colors in starsight travel through several color spaces on their journey from specification to pixel. Understanding these spaces prevents subtle color errors.

sRGB is the standard color space for the web, for most monitors, and for PNG images. It uses a non-linear transfer function (gamma) that compresses bright values and expands dark values. When you specify Color from hex 0x808080, you are specifying a mid-gray in sRGB, which is perceptually mid-gray to the human eye.

Linear RGB is the physically accurate color space where light values are proportional to physical intensity. A value of 0.5 in linear RGB is half the physical light of 1.0. In sRGB, a value of 0.5 is approximately 0.214 in linear space (because of the gamma curve). Linear RGB is the correct space for blending, compositing, and interpolation. If you lerp between red and blue in sRGB, the midpoint looks darker than expected. If you lerp in linear space, the midpoint looks correct.

tiny-skia's Pixmap stores pixels in sRGB by default. When you create a Paint, the color is specified in sRGB and blending happens in sRGB. This is standard behavior and matches user expectations. For perceptually uniform color interpolation (like sampling a colormap), prismatica handles the math in sRGB space, which matches what matplotlib and other tools do.

The palette crate provides conversions between sRGB, linear RGB, Oklab, Oklch, and many other color spaces. For starsight, the palette crate is in the dependency list for future use in gradient creation and color manipulation. For 0.1.0, direct sRGB operations are sufficient.

## How the starsight name was chosen and why it matters

The name starsight was chosen through an exhaustive search of crates.io availability. Over 500 names were checked across physics, astronomy, art, Latin, and compound word categories. The criteria were: available on crates.io, a single word, Latin or Greek scientific register matching the resonant-jovian aesthetic, and a natural reading that suggests visualization or sight.

starsight works because it embeds "rs" (Rust) naturally within the word, it belongs to the astronomical register alongside "resonant-jovian" and the sister crate names (prismatica, chromata, caustic, phasma), and it suggests the act of seeing clearly, which is what a visualization library enables.

The name also determines the documentation URL (docs.rs/starsight), the import path (use starsight::prelude::*), and the crates.io search ranking. A distinctive, memorable name that is easy to type and pronounce matters more than most developers realize.

## How open source maintenance works long-term

Building starsight is a commitment measured in years, not months. The Rust visualization ecosystem has seen many abandoned efforts: crates that published a 0.1.0, received some attention, and then went silent. plotters itself has a "status of the project" issue because maintenance has slowed.

The antidote is sustainable pace. Commit to a regular cadence: one meaningful commit per day, one release per month, one blog post per quarter. Do not sprint to a release and then burn out. The consistent pace is more important than the speed.

Accept contributions carefully. A contributed PR that adds a new chart type is great, but if it does not have snapshot tests, documentation, and clippy-clean code, it creates maintenance burden. The CONTRIBUTING.md should set clear expectations: every PR must pass CI, every new feature must have tests, every public API must have docs.

Monitor the issue tracker actively. Unanswered issues signal an inactive project. Even if you cannot fix a bug immediately, acknowledging it with a comment shows the project is alive.

Plan for burnout. Have a co-maintainer who can handle merges and releases when you need a break. If the project grows beyond solo maintenance, consider the governance model early.

## How the resonant-jovian ecosystem fits together

The resonant-jovian organization on GitHub hosts four published crates and two in development. Understanding how they connect helps you make architectural decisions in starsight.

prismatica provides colormaps. It is a compile-time dependency of starsight for color scales. When a user maps data values to colors using Scale sequential with a prismatica colormap, starsight calls colormap dot eval with a normalized value and gets back a prismatica Color. That Color has the same three-byte structure as starsight's Color, so conversion is zero-cost.

chromata provides themes. It is a compile-time dependency of starsight for the theming system. When a user applies a chromata theme to a chart, starsight reads the theme's bg, fg, and accent colors and derives a chart theme. The mapping is: theme bg becomes chart background, theme fg becomes axis and text color, theme accent colors become the data series color cycle.

caustic is a Vlasov-Poisson solver for astrophysical simulation. It is not a dependency of starsight, but it is a consumer. caustic will use starsight to visualize simulation results: phase-space density plots, potential field contours, particle distributions. This consumer relationship informs starsight's API design: the API should work well for large scientific datasets with millions of data points.

phasma is a terminal UI for caustic, built with ratatui. It will use starsight's terminal backend to render inline charts within the TUI. This consumer relationship informs the terminal backend design: the charts must render correctly within ratatui's layout system and respond to terminal resize events.

The licensing chain matters. prismatica and chromata are GPL-3.0. starsight is GPL-3.0. caustic and phasma are also GPL-3.0. This means the entire ecosystem is GPL-consistent. A user who depends on starsight already accepts the GPL, so depending on prismatica and chromata creates no additional licensing constraint.

## Final thoughts before you start coding

You now have the complete mental model for building starsight. The architecture is seven layers, each a separate crate, with strict dependency direction. The rendering pipeline goes from data to marks to scales to coordinates to path commands to backend to pixels. The color pipeline goes from user specification to sRGB Color to tiny-skia premultiplied pixels. The text pipeline goes from string to cosmic-text shaped glyphs to per-pixel callback to pixmap fill_rect. The testing strategy is snapshot tests for visual output, property tests for mathematical invariants, and unit tests for everything else.

The tools are: rustfmt for formatting, clippy for linting, cargo-deny for dependency governance, cargo-semver-checks for API compatibility, cargo-insta for snapshot testing, cargo-llvm-cov for coverage, cargo-nextest for fast test execution, cargo-hack for feature flag verification, git-cliff for changelogs, taplo for TOML formatting, criterion for benchmarks, and cargo-flamegraph for profiling.

The rules are: no unsafe in layers 3 through 7, no panics in library code, no println or eprintln, no async, no JavaScript dependencies, no C dependencies in the default feature set, no nightly-only features. Every public type derives Debug and Clone. Every public item has a doc comment. Every error is a StarsightError. Every feature-gated module is behind a cfg attribute at the module level.

Start with the vertical slice. Get plot save to produce a PNG. Everything else follows from there.

## How cargo-deny's four checks work in detail

The licenses check reads every dependency's license metadata and compares it against your allow list. Cargo.toml has a license field that holds an SPDX expression. Common expressions are MIT, Apache-2.0, MIT OR Apache-2.0, and BSD-3-Clause. cargo-deny parses these expressions and checks that every term is in your allow list.

Some crates do not declare their license in Cargo.toml metadata. Instead, they only have a LICENSE file in the source tree. For these, cargo-deny can fall back to file analysis, but this is unreliable. The clarify section in deny.toml lets you manually specify the license for these crates. The ring cryptography crate is the most common case: its LICENSE file contains a mix of ISC, MIT, and OpenSSL license texts that automated parsers cannot decompose.

The allow list for a GPL-3.0 project is longer than for an MIT project. Every permissive license is GPL-compatible. MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0, and Unlicense are all fine. The Mozilla Public License 2.0 (MPL-2.0) is also compatible because it is a weak copyleft that allows linking with GPL code. The LGPL in all its variants (2.1-only, 2.1-or-later, 3.0-only, 3.0-or-later) is compatible. And GPL-3.0 itself is obviously compatible. What is not compatible: AGPL-3.0 (the network clause is stricter than GPL), SSPL (Server Side Public License, used by MongoDB), and proprietary licenses.

A subtle gotcha: cargo-deny version 0.18.4 and later treats GNU license identifiers pedantically. GPL-3.0, GPL-3.0-only, and GPL-3.0-or-later are three distinct identifiers. If your allow list only contains GPL-3.0-only and a dependency uses GPL-3.0, the check fails. Add all three forms to be safe.

The advisories check downloads the RustSec advisory database (a git repository hosted by the Rust security team) and checks your Cargo.lock against it. Each advisory has an ID (like RUSTSEC-2024-0001), affected crate names and version ranges, and a severity level. If any of your dependencies fall within an affected range, the check fails.

You can ignore specific advisories with the ignore list in deny.toml. This is useful when no fix is available yet, or when the vulnerability does not affect your use of the crate. Each ignore entry should include a reason explaining why it is safe to ignore.

The bans check prevents specific crates from appearing in your dependency tree. The multiple-versions setting controls whether having two different versions of the same crate is allowed. Setting it to deny catches diamond dependency situations where different parts of your tree pull in incompatible versions of the same crate. The skip-tree setting exempts specific dependency subtrees from the multiple-versions check, which is necessary for crates like windows-sys that often appear in multiple versions due to ecosystem churn.

The sources check verifies that all dependencies come from known registries. By default, only crates.io is allowed. Git dependencies, local path dependencies outside the workspace, and dependencies from alternative registries are denied unless explicitly allowed. For starsight, this is a security measure: it ensures that a compromised dependency cannot redirect to a malicious git repository.

## How cargo-semver-checks analyzes your API

cargo-semver-checks works by generating rustdoc JSON for two versions of your crate: the baseline (the last published version on crates.io) and the current version (your working tree). It then compares the two JSON representations using a graph query language called Trustfall, which lets it express complex semantic checks like "find all public functions that existed in the baseline but are missing in the current version."

The tool has over 120 lint rules. Each rule checks for a specific category of breaking change. Some examples: function_missing fires when a public function is removed. method_parameter_count_changed fires when a method gains or loses parameters. trait_method_added fires when a new required method is added to a public trait (which breaks all external implementors). enum_variant_added fires when a variant is added to an enum that is not non_exhaustive. struct_pub_field_missing fires when a public field is removed from a struct.

What cargo-semver-checks does not catch: changes to the types of function parameters (changing u32 to u64 is breaking but involves type analysis that the tool does not perform), changes to lifetime parameters, changes to generic parameters, behavioral changes (a function that returns different values for the same inputs), and cross-crate type analysis (changing a type alias that points to a type from another crate).

The tool can be configured per-crate and per-lint. In Cargo.toml, you can set workspace-level lint severities and override them in individual crates. This is useful if a specific lint is too strict for your project (for example, function_must_use_added fires when you add the must_use attribute to a function, which is technically a breaking change because it introduces a new warning in downstream code, but is widely considered acceptable).

## How cargo-release handles the publish order

When you run cargo release for a workspace, it needs to figure out which crates to publish first. It reads the dependency graph from the workspace Cargo.toml and performs a topological sort. Crates with no internal dependencies are published first, then crates that depend only on already-published crates, and so on until the facade crate (which depends on everything) is published last.

For starsight, the order is: starsight-layer-1 (depends on no internal crates), then starsight-layer-2 (depends on layer-1), then layer-3, layer-4, layer-5, layer-6, layer-7, and finally the starsight facade crate. The xtask crate is never published (it has publish equals false in its Cargo.toml).

Between each publish, cargo-release waits for the crate index to update. crates.io has a propagation delay: after publishing layer-1, it may take 10 to 30 seconds before layer-2 can successfully resolve its dependency on the new layer-1 version. cargo-release handles this with retry logic.

If a publish fails midway (for example, crates.io is down, or a dependency version was not found yet), cargo-release stops and prints the error. You can resume from where it stopped by re-running the command. It detects which crates have already been published and skips them.

The consolidate-commits setting in release.toml controls whether cargo-release creates one commit per crate or one commit for the entire workspace. For starsight, consolidate equals true creates a single commit titled "chore: Release 0.2.0" that bumps all version numbers at once. This is cleaner in the git history than nine separate commits.

## How cargo-llvm-cov instruments your code

When you run cargo-llvm-cov, it passes the minus C instrument-coverage flag to rustc. This flag inserts counter increments at strategic points in the compiled code (branch points, function entries, and expression boundaries). When the program runs, these counters record which code paths were executed and how many times.

After the test run, cargo-llvm-cov uses llvm-profdata to merge the raw counter data into a profile, then uses llvm-cov to map the profile back to source lines. The result is a coverage report showing which lines were executed (green), which were not (red), and which are not instrumentable (gray, like type definitions and attribute macros).

The tool generates several output formats. LCOV is a line-oriented text format that coverage services like Codecov and Coveralls understand. HTML generates a browsable report with syntax-highlighted source files annotated with coverage data. JSON provides machine-readable data for custom analysis.

A critical detail for starsight: test code is excluded from coverage by default. This means the lines in your test functions do not count toward or against the coverage percentage. The coverage numbers reflect only library code.

Branch coverage (as opposed to line coverage) is available with the minus minus branch flag. Branch coverage reports whether both arms of each conditional were executed. This is stricter than line coverage: a function might have 100 percent line coverage but only 50 percent branch coverage if an if-else always takes the same path.

## How cargo-insta stores and manages snapshots

When you call the snapshot assertion macro in a test, insta checks if a matching snapshot file exists. The file is named after the test module and the snapshot name (or an auto-generated number if you do not provide a name). For text snapshots, the file has a dot snap extension and contains YAML-like metadata plus the snapshot content. For binary snapshots, there are two files: a dot snap metadata file and the actual binary file (like a dot png).

Snapshot files live in a snapshots directory at the same level as the test file. For a test in starsight-layer-1/tests/snapshot_basic.rs, the snapshots go in starsight-layer-1/tests/snapshots/. The directory is created automatically on the first run.

When a snapshot changes, insta writes the new content to a dot snap dot new file alongside the existing snapshot. This pending file persists until you explicitly accept or reject it. Running cargo insta review shows a side-by-side comparison and lets you accept each change individually. Running cargo insta accept accepts all pending changes. Running cargo insta reject deletes all pending files without updating snapshots.

The check flag changes this behavior: instead of creating pending files, it fails immediately if any snapshot does not match. This is what CI uses, because CI should not create pending files (there is nobody to review them).

The unreferenced reject flag handles a different problem: orphaned snapshots. If you delete a test but leave its snapshot file behind, the orphaned file wastes space and creates confusion. The unreferenced reject flag fails if any snapshot file exists that is not referenced by an active test. This only works correctly when running the full test suite, because running a subset of tests makes the snapshots for the unrun tests appear unreferenced.

For redactions, insta can replace non-deterministic values (timestamps, random IDs) with placeholders before comparison. The redaction syntax uses JSON-pointer-like paths. This is useful for snapshot testing data structures that contain generated values.

## How proptest shrinking works

When proptest finds a failing input, it does not immediately report it. Instead, it tries to find a smaller input that still fails. This process is called shrinking. Shrinking is critical because randomly generated inputs are often large and complex, making it hard to understand why they fail. A shrunk input shows the essential condition that triggers the bug.

Shrinking works by modifying the failing input in small ways: replacing a large number with a smaller one, removing elements from a vector, replacing a string with a shorter string. After each modification, proptest re-runs the test. If the test still fails, the smaller input becomes the new candidate for further shrinking. If the test passes, the modification was too aggressive and proptest tries a different modification.

For starsight's property tests, shrinking has practical implications. If a scale roundtrip test fails for the input 98765.4321, proptest might shrink it to 1.0 or 0.0 or minus 1.0, revealing that the bug is actually about the sign or about zero handling rather than about large values. This drastically reduces debugging time.

The shrinking process can be slow for complex types. If your test generates a vector of 10000 floats and the test fails, proptest might try removing elements one by one, which is 10000 attempts per shrinking round. For these cases, limit the generated size (use vectors of at most 100 elements in property tests) and set a timeout per test case.

Regression files store the exact inputs that caused failures. They are stored in proptest-regressions directories alongside the test files. Commit these to version control. They serve as permanent regression tests: even if proptest's random seed changes, the known failing inputs are always re-tested as part of the regular test suite.

## How criterion statistical analysis works

criterion does not just time your code once. It runs your code hundreds of times, collects the measurements, and applies statistical analysis to determine the typical execution time and the confidence interval.

The warm-up phase runs the code for a configurable duration (default 3 seconds) to let the CPU caches, branch predictors, and JIT-like optimizations stabilize. Measurements during warm-up are discarded.

The measurement phase runs the code in iterations, grouped into samples. Each sample is a set of iterations run back-to-back. The number of iterations per sample is chosen so that each sample takes at least the target time (default 5 seconds divided by the sample count). criterion collects at least 100 samples by default.

After collection, criterion uses bootstrapping (a non-parametric statistical technique) to estimate the mean execution time and the 95 percent confidence interval. It also runs a linear regression to detect measurement overhead. If the time per iteration is not constant across samples (indicating measurement noise or interference), criterion increases the confidence interval accordingly.

When you run criterion a second time, it compares the new measurements against the saved baseline. It reports whether the performance changed (and by how much) with statistical significance. A change is significant if the confidence intervals of the old and new measurements do not overlap.

For starsight, the most useful criterion features are benchmarking with inputs (testing at 100, 1000, 10000, 100000 data points), throughput measurement (reporting per-point cost), and comparison between runs (detecting performance regressions in PRs).

The HTML reports generated by criterion include interactive plots showing the distribution of measurements, the estimated PDF of the execution time, and the performance comparison against the baseline. These reports are saved to target/criterion/ and can be served from a GitHub Pages site for ongoing performance tracking.

## How cargo-flamegraph interprets the call stacks

A flamegraph is a visualization of aggregated stack traces. The horizontal axis represents the proportion of total CPU time. The vertical axis represents the call stack depth. Each box is a function call. The width of the box is proportional to how often that function appeared in the stack samples.

Reading a flamegraph for starsight rendering: the bottom of the stack is the main function. Above it is the save or show method on Figure. Above that is the render method. Above that are the individual mark render calls. Above those are the DrawBackend method calls. And at the very top are the tiny-skia internal functions: fill_path, stroke_path, the rasterization pipeline, and the alpha blending operations.

Wide boxes at the top mean direct CPU consumption: the function itself (not its callees) is taking time. For tiny-skia, the wide boxes are typically in the inner rasterization loops where per-pixel color computation happens. You cannot optimize these directly (they are in the dependency), but you can reduce how much work they do by simplifying paths, reducing the rendered area, or disabling anti-aliasing where it is not needed.

Wide boxes in the middle mean a function is calling many expensive children. If the Figure render method is wide, look at which of its children are widest: is it axis rendering (text shaping is expensive), mark rendering (path construction or draw calls), or layout computation (margin and tick calculation)?

Narrow but tall stacks indicate deep call chains. These often benefit from inlining: if function A calls B which calls C which calls D, and each call adds a few nanoseconds of overhead, collapsing the chain can help. The inline hint attribute can encourage the compiler to inline small functions, though it is not guaranteed.

The sampling frequency matters. The default is usually 99 or 997 times per second. Using a prime number avoids aliasing with periodic system activity. For starsight, increase the frequency to 1997 or 4999 for short-running benchmarks to collect enough samples. For long-running workloads, the default frequency is fine.

## How git-cliff parses conventional commits in detail

git-cliff reads the git log and parses each commit message according to the Conventional Commits specification. A conventional commit has the form: type, optional scope in parentheses, optional exclamation mark for breaking changes, colon, space, description, optional body, optional footers.

The type determines the changelog section. feat maps to Added (or Features), fix maps to Fixed (or Bug Fixes), perf maps to Changed (or Performance), docs maps to Documentation, refactor maps to Changed (or Refactor), test maps to Testing, build and ci map to Build (or CI), and chore is typically skipped.

The scope is an optional identifier in parentheses after the type. For starsight, useful scopes include: layer-1, layer-2, layer-3, layer-4, layer-5, layer-6, layer-7, primitives, scale, tick, skia, svg, pdf, wgpu, terminal, figure, and ci. The scope appears in the changelog entry: "scale: implement log scale" under the Features heading.

The exclamation mark after the type (or scope) indicates a breaking change. feat bang colon remove DrawBackend save_svg generates a Features entry marked as breaking. A BREAKING CHANGE footer in the commit body has the same effect.

The body and footers are used for additional context. The body can contain a detailed description of the change. Footers like Closes #42 link to issues. Reviewed-by records reviewer names. git-cliff can include or exclude these in the changelog based on the template configuration.

The cliff.toml template uses the Tera templating language, which is similar to Jinja2. The template iterates over commits grouped by type, formats each entry, and wraps everything in the release header with the version number and date. The header and body sections of cliff.toml control the overall structure and per-release content.

For workspace releases, git-cliff can generate a single combined changelog or per-crate changelogs. The include-path flag filters commits by the files they touch. A commit that modifies files in starsight-layer-2 appears in layer-2's changelog but not in layer-7's. The downside is that cross-cutting changes (like a workspace-wide version bump) appear in every crate's changelog.

## How taplo validates and formats TOML

TOML (Tom's Obvious Minimal Language) is the configuration format for Rust projects. Every Cargo.toml, deny.toml, cliff.toml, and other configuration file uses it. taplo ensures these files are syntactically valid, consistently formatted, and free of common mistakes.

Formatting rules include: consistent indentation, key ordering, trailing commas in arrays, and quoting style for strings. The most useful formatting rule for starsight is alphabetical ordering of dependencies. When nine crates each have a dependencies section with 5 to 15 entries, keeping them alphabetical makes it easy to find and compare entries across crates.

The formatting can be configured per-section. You might want dependencies alphabetized but the package section left in logical order (name first, version second, edition third). The taplo.toml rule system supports this with pattern-based section matching.

Validation catches real bugs. Duplicate keys in a TOML table silently overwrite the first value, which can cause mysterious behavior if you accidentally define the same dependency twice with different versions. taplo catches this and reports it as an error.

In CI, taplo check runs the formatter in check mode: it reports files that would change without modifying them. If any file is not formatted according to the rules, the check fails. This prevents formatting drift across contributors.

## How the rust-toolchain.toml file interacts with CI

The rust-toolchain.toml file is read by rustup, the Rust toolchain manager. When rustup detects this file in the project directory (or any parent directory), it automatically installs and uses the specified toolchain and components.

In CI, the dtolnay/rust-toolchain action reads this file when you specify toolchain as the channel from the file. This means your CI runs the same compiler version that developers use locally. No more "works on my machine" issues caused by compiler version differences.

The components field specifies additional toolchain components beyond the default. For starsight, the essential components are: rustfmt (code formatting), clippy (linting), llvm-tools-preview (code coverage instrumentation), and rust-src (needed for some tools that analyze the standard library source). Without specifying components, a bare toolchain installation might miss clippy or rustfmt, causing CI steps to fail.

The targets field can specify additional compilation targets. For starsight, this would include wasm32-unknown-unknown when the web feature is implemented. Adding targets here means contributors automatically get the WASM target installed when they set up the project.

## How cargo-msrv finds the minimum version

cargo-msrv find performs a binary search across Rust versions. It starts with the most recent stable release and the oldest supported release (configurable, default 2 years back). It tries the midpoint: if the project compiles, it moves the lower bound up; if it fails, it moves the upper bound down. After about 6 to 8 iterations (log2 of the number of stable releases in the search range), it reports the minimum version.

For each candidate version, cargo-msrv downloads the toolchain via rustup (if not already cached), runs cargo check, and interprets the result. This is slow because downloading toolchains takes time and compiling the project takes time. A full search might take 10 to 30 minutes.

The verify subcommand is much faster: it only checks one version (the declared rust-version from Cargo.toml). If the check passes, the MSRV is verified. If it fails, you know the declared MSRV is wrong and needs to be bumped.

For starsight, the recommended CI approach skips cargo-msrv entirely and includes the MSRV version in the test matrix. The ci.yml already tests against 1.85.0 alongside stable and beta. If the 1.85.0 job passes, the MSRV is implicitly verified.

## How cargo-hack tests feature combinations

cargo-hack extends Cargo with subcommands for testing feature flag combinations. The each-feature flag runs a command once for each feature, enabling only that feature. The feature-powerset flag runs a command for every possible combination of features. The exclude-features flag skips specific features (useful for mutually exclusive features or very expensive ones).

For starsight, cargo hack check with the each-feature flag verifies that each of the 18 feature flags compiles independently. This catches a common bug: feature A depends on code that is only compiled when feature B is enabled, but A does not declare a dependency on B. In the workspace context (where all features are unified), both A and B happen to be enabled, so the bug is hidden. cargo-hack reveals it by testing each feature in isolation.

The no-dev-deps flag is critical when using cargo-hack in check mode. Without it, dev-dependencies are included in the build, and they can mask missing dependencies for the same reason as feature unification. With no-dev-deps, only the declared dependencies are available, giving a true picture of each feature's compilation requirements.

The feature-powerset with depth 2 tests every combination of up to 2 features. For 18 features, this is approximately 18 times 17 divided by 2, or 153 combinations, plus the 18 single-feature combinations, plus the zero-feature case: about 172 checks. Each check is a cargo check (no tests, no linking), so the total time is roughly 172 times the single-check time. At 10 seconds per check, that is about 30 minutes. This is feasible for a weekly CI job but too slow for every PR.

Full powerset testing (all 2 to the 18th power combinations, or about 262000) is not practical. The depth limit is the key to making powerset testing usable.

## How the Rust API Guidelines apply to starsight specifically

The Rust API Guidelines checklist has about 70 items. Here are the ones most relevant to starsight, with specific guidance on how to apply them.

Types eagerly implement common traits. For starsight: every public struct should implement Debug (for print debugging and error messages), Clone (for users who want to modify a copy of a configuration), and Display where meaningful (for colors, points, errors). Send and Sync should be implemented or verified for types that users might want to move between threads.

Conversions use the standard From, Into, TryFrom, and TryInto traits. For starsight: Color implements From for chromata Color and prismatica Color. Point implements From for two-element arrays and tuples. Rect implements TryFrom for tiny_skia Rect (TryFrom because the conversion can fail if bounds are invalid). Use Into in function signatures for ergonomic callers.

Error types implement std Error. For starsight: StarsightError implements Error through thiserror's derive macro. The Display implementation provides human-readable messages. The source method links to underlying errors.

Builder methods are named well. For starsight: methods that create a new modified copy use the with prefix (with_alpha on Color). Methods that mutate in place use the set prefix or take and mut self. Methods that convert use the to or into prefix.

Public dependencies are re-exported. If starsight's public API exposes a type from tiny-skia (it should not, but if it ever does), the type must be re-exported so users do not need to add tiny-skia as a separate dependency. This is one more reason to wrap external types in your own types: it avoids the re-export requirement entirely.

Sealed traits prevent external implementations. If the DrawBackend trait should only be implemented by starsight's own backends (not by external crates), use the sealed trait pattern: add a method that returns a private type. External crates cannot implement the private method, so they cannot implement the trait. However, for starsight, DrawBackend should probably be implementable externally (a user might want to implement a custom backend for their own use), so do not seal it.

## How to think about the non_exhaustive attribute in depth

Adding non_exhaustive to a type that is already published is a breaking change. This is because downstream code that exhaustively matches on the enum or constructs the struct with literal syntax will no longer compile. Removing non_exhaustive is also a breaking change for structs (because it changes the struct's constructibility). For enums, removing non_exhaustive is technically not breaking (it only makes matches easier), but cargo-semver-checks may still flag it.

The practical rule for starsight: add non_exhaustive to every public enum and every public struct that might gain fields, before the first publish. Once it is on the type, adding new variants or fields is a non-breaking change in any future version.

The exception is types whose fields are their complete mathematical definition. Point (x, y), Vec2 (x, y), Color (r, g, b), and Size (width, height) have fields that are fundamental to what the type is. Adding a third field to Point would change it from a 2D point to something else entirely, which would be a redesign, not an incremental change. These types should not have non_exhaustive.

For configuration structs like RenderOptions or ThemeConfig, non_exhaustive is essential. You will definitely want to add fields like dpi, background_color, or font_family in future versions. With non_exhaustive, these additions are non-breaking.

For error enums like StarsightError, non_exhaustive is essential. You will discover new error conditions as you implement more backends and chart types. Adding a new variant like Gpu(String) or Font(String) should not break downstream match statements.

The tradeoff: non_exhaustive makes the API slightly less ergonomic. Users cannot construct the struct with literal syntax, so they need a constructor function. Users cannot exhaustively match, so they need a wildcard arm. But this tradeoff is overwhelmingly worthwhile for a pre-1.0 library that will evolve rapidly.

## How thiserror makes error types work

thiserror is a derive macro that generates implementations of the standard Error trait, the Display trait, and optionally the From trait for your error enum. It takes the boilerplate out of error type definitions.

The error attribute on each variant defines the Display format string. The from attribute on a variant field generates a From implementation that converts the source error type into your error type. The source attribute marks a field as the underlying error (for the Error source method) without generating a From implementation.

For starsight, thiserror generates all the Display text for StarsightError. The from attribute on the Io variant means any function returning Result of StarsightError can use the question mark operator on standard io errors: the conversion happens automatically. The other variants (Render, Data, Scale, Export, Config) take Strings, which requires manual construction: return Err of StarsightError Render of "message".

A good practice for error messages: write them in lowercase, do not include trailing periods, and include enough context to identify the failed operation. Instead of "failed" write "failed to create pixmap at 800 by 600: out of memory." Instead of "error" write "scale domain is empty: min equals max equals 5.0."

If a variant needs both a message and a source error, use a struct variant with named fields. The display attribute format string can reference the fields by name. The source attribute on the source field implements the chain correctly without generating a From that would conflict with other variants using the same source type.

## How deprecation works in practice for a pre-1.0 library

Rust has built-in deprecation support via the deprecated attribute. When you mark a function as deprecated, any code that calls it produces a compiler warning. The warning includes the deprecation message, which should tell the user what to use instead.

For starsight, the deprecation cycle works like this. In version 0.2.0, you realize that the draw_path method on DrawBackend should take a reference to a PathStyle, not an owned PathStyle. You cannot just change the signature because that breaks all existing backend implementations. Instead: in 0.2.0, add a new method draw_path_ref that takes a reference. Mark the old draw_path as deprecated with a note saying "use draw_path_ref instead; draw_path will be removed in 0.4.0." Provide a default implementation of draw_path that calls draw_path_ref. In 0.4.0, remove draw_path.

This gives users two full releases to migrate. The deprecation warning is visible but not blocking (it is a warning, not an error, unless the user has turned warnings into errors). The migration path is clear: find all calls to draw_path, change them to draw_path_ref.

In the changelog, deprecations appear under the Deprecated heading. Removals appear under the Removed heading in the version where the deprecated item is finally removed. Each removal entry should reference the version where the item was deprecated and the replacement.

## How to write tests that actually catch bugs

The purpose of a test is to fail when something is wrong. A test that always passes is worthless. A test that fails intermittently is worse than worthless. Every test should have a clear failure condition and a clear connection to a specific behavior.

For unit tests: test one thing per test function. If the test name says "test color from hex" then the test should only exercise the from_hex function, not from_hex and to_hex and luminance. Test both the happy path and the error path. If from_hex returns Option, test that it returns Some for valid input and None for invalid input.

For snapshot tests: choose deterministic inputs. Use fixed data, fixed dimensions, fixed fonts (embed a font in the test rather than relying on system fonts), and fixed random seeds. If any part of the rendering pipeline uses randomness (like jitter positioning), fix the seed in tests.

For property tests: state the property clearly in the test name and in a comment. "scale roundtrip" is a property: for any input, scale forward then inverse returns the input. "ticks are monotonic" is a property: for any data range, the tick positions are strictly increasing. Properties that are difficult to state are often signs that the code's behavior is not well-defined.

For integration tests: test the full pipeline from user input to file output. Create data, build a Figure, render to a file, and assert on the file's contents (via snapshot) or metadata (file exists, file size is non-zero, file is valid PNG).

Do not test implementation details. If the LineMark internally uses a Vec of PathCommand to construct the path, do not test the intermediate Vec contents. Test the rendered output. If the internal representation changes, the test should not break as long as the output is correct.

## How to think about performance from the start

Performance optimization should not happen before the code works correctly. But performance-aware architecture decisions should happen from the start, because they are expensive to retrofit.

The key architecture decision for starsight's performance is the Scene graph. By accumulating SceneNode data instead of calling backend methods directly, the architecture enables batching (grouping similar draw calls), reordering (drawing all fills before all strokes to reduce state changes), and culling (skipping nodes that are entirely outside the visible area). None of these optimizations are implemented in 0.1.0, but the architecture supports them without changes to the mark or figure layers.

The second key decision is object reuse. The FontSystem and SwashCache must be created once and reused across all text rendering operations. The Pixmap should be created once per render call, not per mark. The PathBuilder should be reused (via clear and rebuild) rather than allocated fresh for each path.

The third key decision is avoiding unnecessary allocation. Use slices (and f64) instead of Vec of f64 for read-only data access. Use Cow for strings that are usually static but occasionally owned. Preallocate Vecs with with_capacity when the size is known.

After correctness is established, use criterion benchmarks to measure baseline performance and cargo-flamegraph to identify hotspots. Optimize only the hotspots. A 10x speedup on a function that takes 1 percent of the total time saves 0.9 percent. A 2x speedup on a function that takes 50 percent of the total time saves 25 percent.

## How to read and understand tiny-skia's source code

When you encounter a rendering bug that you suspect is in tiny-skia rather than in your code, you need to be able to read tiny-skia's source. The crate is well-structured and relatively small (about 15000 lines of Rust). Understanding its layout helps you navigate quickly.

The core rendering pipeline lives in the pipeline module. The entry point is the fill or stroke method on Pixmap. These methods convert the path to a set of scanlines (horizontal spans of pixels), apply the paint (which determines the color at each pixel via a shader), and blend the result into the existing pixel buffer.

The path processing happens in the path module. PathBuilder accumulates move, line, quad, cubic, and close commands. The finish method validates the path (checking for degenerate segments and empty paths). The stroking module converts a path and a stroke specification into a filled path that represents the stroked outline.

The shader module determines the color at each pixel. The simplest shader is SolidColor, which returns the same color everywhere. LinearGradient computes a color based on the pixel's position along a line. RadialGradient computes a color based on the distance from a center point. Pattern tiles an image.

The blend module handles compositing: combining the new color with the existing pixel buffer. The default blend mode is SourceOver, which is the standard alpha compositing mode. Other modes (Multiply, Screen, Overlay, and so on) are available but rarely needed for chart rendering.

The mask module handles clipping. A Mask is a grayscale image where white pixels allow drawing and black pixels block it. The mask is applied during the blend step: pixels outside the mask are not modified.

When debugging, the most common question is "why does this pixel have this color?" The answer is always: the shader computed a color, the blend mode combined it with the existing pixel, and the mask allowed or blocked the result. If the color is wrong, check the shader. If the pixel should be there but is not, check the mask. If the blending looks wrong, check the blend mode and the alpha values.

## How cosmic-text handles font fallback and missing glyphs

When cosmic-text shapes text, it looks up each character in the specified font. If the character is not in that font (for example, a Japanese character in a Latin font), it falls back to another font that contains the character. The fallback order is determined by the font database, which is configured differently on each operating system.

For chart rendering, font fallback rarely matters because chart text consists of digits, Latin letters, and a few symbols (parentheses, commas, decimal points, minus signs). These are in every font. But if users can supply custom labels with arbitrary Unicode (which they can via the title, x_label, and y_label methods), font fallback becomes relevant.

If no font in the system contains a character, cosmic-text inserts a replacement glyph (the "tofu" rectangle or a question mark). For starsight, this means: if a user puts a Chinese character in a chart title and the system does not have a Chinese font installed, the character appears as a rectangle. This is acceptable behavior (it is what every other application does), but it should be documented.

The FontSystem also handles font weight and style. If you request bold text but the system does not have a bold variant of the font, cosmic-text attempts to synthesize bold by applying a stroke to the outlines. Similarly, it can synthesize italic by applying a shear transform. The synthesized versions look worse than true bold and italic but are better than falling back to the regular weight.

For the embedded font scenario (where starsight bundles a font for deterministic rendering in tests), you load the font data into the FontSystem with the db_mut().load_font_data method, then specify the font family name when setting text on the Buffer. The loaded font takes priority over system fonts with the same name.

## How SVG rendering libraries handle starsight's output

When starsight generates an SVG file, that file will be rendered by a variety of SVG implementations: web browsers (Chrome, Firefox, Safari), image viewers (Eye of GNOME, Preview.app), vector editors (Inkscape, Illustrator), and programmatic rasterizers (resvg, librsvg).

Each implementation has slightly different behavior. The most common differences are in text rendering (different fonts, different kerning, different text-anchor interpretation), gradient rendering (different interpolation methods at the edges), and filter effects (different blur implementations). For chart SVGs, which use simple shapes, solid colors, and basic text, these differences are usually invisible.

The biggest practical issue is text width. When starsight creates an SVG, it positions text elements based on estimated character widths (because it cannot measure actual rendered width without a rendering engine). If the user opens the SVG in a browser that uses a different font than starsight estimated for, the text might be slightly too wide or too narrow, causing overlaps or excessive whitespace.

The mitigation for production use is to embed the font in the SVG. SVG supports the font-face element and the url() CSS function for embedding fonts. starsight can optionally embed the font data as a base64-encoded data URI within a style block. This ensures that every SVG renderer uses the same font and produces the same layout.

For resvg-based rasterization (converting SVG to PNG without a browser), starsight can use the resvg crate (behind the resvg feature flag). resvg is a Rust SVG renderer that handles most of the SVG specification. The advantage over browser rendering is determinism: resvg always produces the same output for the same input, making it suitable for snapshot testing of SVG output.

## How the palette crate integrates with starsight's color system

The palette crate provides a comprehensive color management library with support for dozens of color spaces. For starsight, the most relevant capabilities are: sRGB to linear RGB conversion (needed for correct blending), Oklab/Oklch (perceptually uniform color spaces for generating pleasant gradients), and color mixing (blending colors in perceptually uniform space).

Currently, starsight's Color type is a simple three-byte sRGB struct. The palette crate is listed in the workspace dependencies but is not used in the 0.1.0 implementation. It becomes important when starsight implements:

Color gradient fills for area charts and heatmaps, where blending should happen in linear RGB or Oklab space for perceptual uniformity. Creating custom colormaps where the user specifies control points and starsight interpolates between them. Color accessibility tools that convert a chart's color scheme to simulate how it appears to colorblind users.

The integration path is: starsight Color converts to palette's Srgb type via From, palette performs the color space operations, and the result converts back to starsight Color. All conversions go through the f32 representation. The conversion chain is: starsight Color (u8 sRGB) to palette Srgb of f32 (f32 sRGB) to palette LinSrgb of f32 (linear RGB) to palette Oklab of f32 (perceptually uniform) and back.

For 0.1.0, the Color lerp method does linear interpolation in sRGB space. This is slightly incorrect perceptually (the midpoint between two colors appears too dark in sRGB) but matches what matplotlib and most other tools do. Correct perceptual interpolation via Oklab can be added as an option in a later version without breaking the existing API.

## How nalgebra integrates for three-dimensional visualization

nalgebra is a linear algebra library that provides vectors, matrices, and transforms for arbitrary dimensions. For starsight's 3D chart types (Surface3D, Scatter3D, Wireframe3D, Isosurface), nalgebra provides the camera model, projection matrices, and transform operations.

The camera model determines how 3D data points map to 2D screen coordinates. The two common projections are perspective (distant objects appear smaller, giving a sense of depth) and orthographic (all objects appear the same size regardless of distance, useful for technical drawings).

The camera state includes the position (where the camera is in 3D space), the target (what point the camera looks at), the up direction (which way is up), and the field of view (how wide the camera's view angle is). Orbit controls let the user rotate the camera around the target by dragging the mouse, zoom by scrolling, and pan by right-dragging.

For 3D marks, each data point has three coordinates (x, y, z). The mark converts these to screen coordinates by applying the model-view-projection matrix. The model matrix positions the chart in world space. The view matrix positions the camera. The projection matrix converts 3D to 2D. The result is a 2D point in normalized device coordinates, which is then mapped to pixel coordinates using the viewport transform.

nalgebra is behind the 3d feature flag and is not needed until version 0.7.0. The layer-1 architecture does not need to know about 3D: the 3D mark types in layer 3 perform the projection and emit 2D PathCommand sequences to the DrawBackend. From the backend's perspective, a 3D scatter plot is just a collection of 2D circles at projected positions.

## How winit creates native windows for interactive charts

winit is a cross-platform window creation library. It handles the platform-specific details of creating a window, receiving input events, and managing the event loop on Windows, macOS, Linux, and the web.

For starsight's interactive mode (version 0.6.0), the show method on Figure creates a winit window, initializes a rendering backend (wgpu for GPU, or tiny-skia for CPU with manual buffer presentation), and enters the event loop. The event loop receives events (window resize, mouse move, mouse click, keyboard input, close requested) and dispatches them to the chart's interaction handlers.

The event loop is blocking: once you call show, control does not return to the caller until the window is closed. This is why the show method is the last call in an interactive session. For non-blocking use (embedding a chart in an existing GUI application), the user would use a lower-level API that renders to a texture or pixel buffer, which the user's GUI framework displays.

winit's API changed significantly between versions. starsight depends on winit 0.31, which uses the ApplicationHandler trait model. The handler receives events through trait method callbacks. The most relevant events for chart interaction are: WindowEvent::CursorMoved (for hover), WindowEvent::MouseWheel (for zoom), WindowEvent::MouseInput (for click and drag), and WindowEvent::Resized (for responsive layout).

## How ratatui renders charts in the terminal

ratatui is a terminal UI framework that provides a widget system, a layout engine, and a rendering abstraction over terminal escape sequences. starsight's terminal backend (version 0.8.0) provides a widget that integrates with ratatui's rendering model.

The widget implements ratatui's Widget trait, which has a single method: render, taking a Rect (the available area in terminal cells) and a Buffer (the terminal character grid). The widget converts the chart to a character representation and writes it into the Buffer.

For terminals that support graphics protocols (Kitty, Sixel, iTerm2), the widget renders the chart to a PNG using the tiny-skia backend at a resolution matched to the terminal's cell pixel size, then encodes the PNG in the appropriate protocol and writes the escape sequences into the Buffer using ratatui-image.

For terminals without graphics support, the widget uses half-block characters (the upper half and lower half block Unicode characters) to achieve roughly double the vertical resolution of plain characters, or Braille dot patterns (which provide 2 by 4 dots per character cell) for line charts. The color is applied using ANSI 24-bit color escape sequences.

The terminal backend's unique challenge is dynamic sizing. Terminal windows can be resized at any time. The widget must handle arbitrary Rect dimensions, re-layout the chart for the new size, and re-render. This is the same layout computation as for static charts, just triggered by a resize event instead of by a save call.

## How the interactive HTML export works

Interactive HTML export (version 0.10.0) produces a self-contained HTML file that includes the chart data, the rendering logic (as JavaScript), and the interaction handlers. The file can be opened in any web browser and provides hover, zoom, and pan without a server.

The approach is: render the chart's visual structure to SVG, embed the data as a JSON object in a script tag, and include JavaScript code that attaches event listeners to the SVG elements. When the user hovers over a data point, the JavaScript reads the corresponding data value from the JSON and displays a tooltip. When the user scrolls, the JavaScript modifies the SVG viewBox to zoom.

This is similar to how plotly.js works, but without the 3 megabyte plotly.js bundle. starsight generates minimal JavaScript that handles only the specific interactions the chart needs. A simple chart with hover and zoom might have 2 kilobytes of JavaScript. A complex dashboard with linked views might have 10 kilobytes.

The HTML export does not use starsight's Rust rendering code at runtime. The chart is pre-rendered to SVG, and the JavaScript only handles interaction overlays. This means the HTML file works without a Rust runtime, without WASM, and without any server-side component.

## How the GIF and video animation export works

Animated charts (version 0.10.0) produce GIF or MP4 files showing a chart changing over time. The simplest animation is a line chart that draws from left to right. The most complex is a scatter plot where points appear, move, change color, and disappear over a time range.

The animation system works by rendering the chart at multiple time steps and encoding the frames. Each frame is a full chart render to a tiny-skia Pixmap. The frames are then encoded to GIF using the image crate's GIF encoder, or to MP4 using an external encoder like ffmpeg (invoked as a subprocess).

The animation API uses a builder pattern: Figure animate with a frame count, a frame rate, and a callback that receives the current time and modifies the chart state. The callback might update the data visible range (to animate a sliding window), modify the color scale (to animate a heatmap), or adjust the camera position (to rotate a 3D chart).

For GIF encoding, the main challenge is palette quantization. GIF supports only 256 colors per frame. The image crate handles this by finding the best 256-color palette for each frame using a median-cut algorithm. For charts with many colors (like heatmaps with continuous color scales), the quantization can produce visible banding. The mitigation is to use dithering, which distributes the quantization error across neighboring pixels.

For MP4 encoding, starsight writes raw RGBA frames to a pipe connected to ffmpeg's stdin. This requires ffmpeg to be installed on the system, which is a system dependency that violates the "no C dependencies" rule. Therefore, MP4export might use a pure-Rust encoder like rav1e (for AV1) or x264 bindings (if GPL-compatible alternatives exist). This is a version 0.10.0 concern and does not affect the current architecture.

## How coordinate systems beyond Cartesian work

Cartesian coordinates (x right, y up) are the default for most charts. But several chart types need different coordinate systems, and understanding how they work helps you implement them correctly.

Polar coordinates use an angle (theta, measured from the right horizontal axis, increasing counterclockwise) and a radius (r, measured from the center). To convert polar to Cartesian for rendering: x equals r times cosine of theta, y equals r times sine of theta. Polar coordinates are used for radar charts, wind rose charts, and polar scatter plots. The axis is circular: tick marks appear along the circumference and along radii.

Geographic coordinates use longitude (degrees east or west of the prime meridian) and latitude (degrees north or south of the equator). Rendering geographic data requires a map projection: a mathematical function that converts the curved surface of the earth onto a flat plane. The simplest projection is equirectangular (longitude maps directly to x, latitude maps directly to y), which distorts areas far from the equator. The Mercator projection preserves angles but grossly distorts areas (Greenland appears as large as Africa). The Robinson and Natural Earth projections are compromises used in most atlases.

For starsight, the geo feature flag enables geographic chart types. The proj crate provides the actual projection functions. The coordinate system in layer 2 becomes polymorphic: CartesianCoord, PolarCoord, and GeoCoord all implement a common interface that maps data coordinates to pixel positions. The mark types do not need to know which coordinate system they are in; they call data_to_pixel and get a Point back.

## How logarithmic and symlog scales handle edge cases

A logarithmic scale computes the logarithm of the data value before mapping to pixel space. This compresses large values and expands small values, making it possible to visualize data that spans several orders of magnitude on a single axis. Stock prices, earthquake magnitudes, and sound levels are commonly plotted on log scales.

The edge cases are: zero (the logarithm of zero is negative infinity), negative values (the logarithm of a negative number is undefined for real numbers), and values very close to zero (the logarithm produces extremely large negative numbers that dominate the axis).

starsight's LogScale should reject data ranges that include zero or negative values by returning a StarsightError::Scale error. This is different from silently clamping or filtering, which can produce misleading charts. If the user's data contains zeros, they should use a symlog scale instead.

A symmetric log (symlog) scale handles data that crosses zero. It applies a logarithm to the absolute value and preserves the sign. Near zero, it transitions to a linear region to avoid the log singularity. The transition point is controlled by a threshold parameter C: values with absolute value less than C are mapped linearly, and values with absolute value greater than C are mapped logarithmically.

The formula is: if the absolute value of x is less than or equal to C, then the result is x divided by C. If the absolute value of x is greater than C, then the result is sign of x times (1 plus the logarithm base 10 of the absolute value of x divided by C). The factor of 1 ensures continuity at the transition point.

For ticks on a log scale, the standard positions are powers of 10: 1, 10, 100, 1000, and so on. Within each decade, minor ticks can appear at 2, 3, 4, 5, 6, 7, 8, 9 times the power of 10. For a symlog scale, the tick positions mirror around zero: minus 1000, minus 100, minus 10, minus 1, 0, 1, 10, 100, 1000.

## How datetime scales determine tick granularity

A datetime scale maps time values to pixel positions. The challenge is choosing appropriate tick positions. If the time range is 24 hours, ticks should appear at each hour. If the range is 30 days, ticks should appear at each day or every few days. If the range is 10 years, ticks should appear at each year.

The algorithm considers a hierarchy of time granularities: years, months, weeks, days, hours, minutes, seconds. It selects the finest granularity where the number of ticks falls within the target range (typically 5 to 10). If months produce 8 ticks for the given time range, months are used. If months produce 36 ticks, the algorithm steps up to quarters or years.

The formatting changes with the granularity. Yearly ticks display "2024", "2025". Monthly ticks display "Jan", "Feb" or "2024-01" depending on the space available. Daily ticks display "Mon 15" or "15" depending on space. Hourly ticks display "14:00", "15:00". Mixed granularity (showing both dates and times) requires two rows of labels: dates on one row, times below.

The datetime scale is one of the more complex components in the scale system. It needs to handle time zones (UTC, local, named zones), calendar irregularities (months have different lengths, leap years, daylight saving time transitions), and locale-specific formatting (English month abbreviations versus Swedish, date order). For 0.1.0, datetime scales are out of scope. They are planned for 0.5.0.

## How categorical and band scales position discrete data

A categorical scale maps discrete labels (like "Apple", "Banana", "Cherry") to evenly spaced positions along an axis. Unlike continuous scales (linear, log), there is no interpolation between positions. The categories are spaced at fixed intervals.

A band scale extends the categorical scale with the concept of a band width. Each category occupies a band of pixels, not just a point. The band width is the total available space divided by the number of categories, minus padding. The inner padding is the gap between bands (between adjacent bars). The outer padding is the gap at the edges (before the first bar and after the last bar).

For a grouped bar chart, a nested band scale subdivides each category's band among the groups. If there are 3 categories and 2 groups, each category's band is split into 2 sub-bands, one per group. The bars are drawn within these sub-bands.

The band scale's output is not a single position but a range: the start and end of the band. The bar mark reads this range and draws a rectangle from the start to the end. The point mark reads the center of the band for positioning a point at the category midpoint.

## How contour generation works for 2D scalar fields

A contour chart draws isolines (lines of constant value) through a 2D scalar field. Given a grid of z-values (like a terrain elevation grid or a temperature field), the contour algorithm finds the lines where z equals a specific threshold.

The standard algorithm is marching squares. It processes the grid one cell at a time (a cell being the square formed by four adjacent grid points). For each cell, it classifies the four corners as above or below the threshold. There are 16 possible configurations (2 to the 4th power). Each configuration determines which edges of the cell the contour line crosses. The algorithm linearly interpolates the exact crossing position along each edge.

The output is a set of polylines (open or closed). Closed polylines form contour rings around peaks and valleys. Open polylines extend to the grid boundary.

For filled contours (where the regions between isolines are filled with colors from a colormap), the algorithm needs to produce polygons instead of polylines. This requires closing the contour paths and handling the boundary correctly. The contour crate in starsight's dependency list implements this algorithm.

Contour levels (the threshold values) can be specified manually or computed automatically. Automatic computation typically uses the same Wilkinson Extended tick algorithm that starsight uses for axis ticks, applied to the range of z-values.

## How kernel density estimation produces smooth distributions

Kernel density estimation (KDE) is a statistical technique that estimates a smooth probability density curve from discrete data points. Given a set of values (like the ages of survey respondents), KDE produces a smooth curve showing the probability density at each value.

The algorithm places a kernel function (typically a Gaussian bell curve) at each data point, then sums all the kernels. The result is a smooth curve that peaks where data points are dense and approaches zero where data points are sparse.

The bandwidth parameter controls the smoothness. A small bandwidth produces a spiky curve that closely follows the individual data points. A large bandwidth produces a smooth curve that may obscure important features of the distribution. The default bandwidth is computed using Silverman's rule of thumb: bandwidth equals 0.9 times the minimum of (standard deviation, interquartile range divided by 1.34) times n to the power of minus 0.2, where n is the number of data points.

For starsight, KDE is implemented as a stat transform in layer 3. The input is a series of values. The output is a pair of arrays: x positions (evenly spaced across the data range) and y values (the estimated density at each position). The output feeds into a LineMark or AreaMark for rendering.

The ViolinMark is a mirrored KDE: the density curve is reflected vertically around a center line, creating a shape that resembles a violin. The width of the violin at each y-position represents the density at that value. Violin plots are used for comparing distributions across categories, replacing or supplementing box plots.

## How hexbin aggregation handles overplotted scatter data

When a scatter plot has thousands or millions of points, individual dots overlap and become an indistinguishable mass. This is called overplotting. Hexagonal binning (hexbin) aggregates nearby points into hexagonal cells and colors each cell by the count of points it contains.

Hexagons are used instead of squares because hexagons tessellate the plane with the smallest perimeter-to-area ratio among regular polygons that tile, meaning the centers of adjacent hexagons are equidistant. This avoids the visual artifacts of square grids where diagonal neighbors are farther apart than horizontal or vertical neighbors.

The algorithm divides the plot area into a grid of hexagonal cells. Each data point is assigned to the nearest hexagon center. The count (or sum, or mean, or any other aggregation) of points in each cell is computed. The cells are then rendered as filled hexagons with a color determined by a colormap.

For starsight, hexbin is implemented as a combination of a stat transform (hexagonal binning) and a mark (filled hexagons). The stat transform computes the hexagon positions and counts. The mark draws each hexagon as a filled path with six line segments. The color comes from a prismatica colormap applied to the normalized count values.

## How box plots compute their statistics

A box plot displays the five-number summary of a dataset: the minimum, the first quartile (Q1, the 25th percentile), the median (Q2, the 50th percentile), the third quartile (Q3, the 75th percentile), and the maximum. It also identifies outliers: points that fall more than 1.5 times the interquartile range (IQR equals Q3 minus Q1) below Q1 or above Q3.

The computation is a stat transform. The input is a series of values. The output is a struct containing: the median line position, the box edges (Q1 and Q3), the whisker endpoints (the most extreme non-outlier values, not the theoretical 1.5 IQR limits), and a list of outlier positions.

The whisker computation is subtle. The whiskers do not extend to Q1 minus 1.5 IQR and Q3 plus 1.5 IQR. They extend to the most extreme data values within those limits. If all data points are within the limits, the whiskers extend to the actual minimum and maximum. If some data points are beyond the limits, those points become outliers (drawn as individual dots) and the whiskers stop at the last non-outlier value.

The box is rendered as a filled rectangle from Q1 to Q3. The median is rendered as a line across the box. The whiskers are rendered as thin lines extending from the box edges to the whisker endpoints, with a short horizontal cap at each end. Outliers are rendered as individual points.

For grouped box plots (comparing distributions across categories), the band scale positions each group's box within a category band, similar to grouped bar charts.

## How the recipe system enables custom chart types

The recipe proc macro (planned for version 0.11.0) lets users define new chart types as compositions of existing marks, stats, and scales. A recipe is a function that takes data and configuration parameters and returns a Figure with the appropriate marks and scales already configured.

Without the recipe system, creating a custom chart type requires manually constructing a Figure, adding marks, configuring scales, and applying statistical transforms. This is verbose and error-prone. With the recipe system, the user annotates a function with starsight recipe, and the macro generates the boilerplate: it creates the Figure, adds the marks in the right order, configures the scales based on the data, and applies the statistical transforms.

For example, a volcano plot (used in genomics to display differentially expressed genes) combines a scatter mark with specific axis scales (log2 fold change on x, minus log10 p-value on y), threshold lines (significance cutoff as a horizontal line, fold change cutoff as two vertical lines), and color mapping (up-regulated genes in red, down-regulated in blue, non-significant in gray). Without a recipe, setting this up requires about 30 lines of code. With a recipe, it is a single function call.

The recipe system is an API convenience, not a fundamental architectural component. starsight works perfectly well without it. But it significantly reduces the barrier to creating and sharing custom chart types, which is important for adoption in specialized scientific communities.

## How the Polars integration handles lazy and eager frames

Polars has two DataFrame modes: eager (data is computed immediately) and lazy (data is computed only when collected). starsight needs to handle both because users may pass either.

The eager integration is straightforward: given a reference to a DataFrame, starsight extracts columns by name using the column method, which returns a reference to a Series. The Series is then cast to a ChunkedArray of f64 using the f64 method, which returns Result. Each chunk is accessed as a slice for zero-copy access to the underlying data.

The lazy integration requires materializing the LazyFrame before extracting data. starsight calls collect on the LazyFrame to get a DataFrame, then proceeds as above. The collect operation can fail (if the lazy computation is invalid), so the result must be propagated as a StarsightError Data.

The plot macro's DataFrame form detects which type it received. If the first argument implements the DataFrameOps trait (which both DataFrame and LazyFrame implement), it enters the DataFrame codepath. Otherwise, it falls back to the array codepath.

A performance consideration: for lazy frames with expensive computations, starsight should only extract the columns it needs, not materialize the entire frame. The select method on LazyFrame lets you specify which columns to keep before collecting. The plot macro generates a select call with only the referenced column names.

## How arrow RecordBatch integration works

Apache Arrow is a columnar data format for in-memory analytics. A RecordBatch is Arrow's equivalent of a DataFrame: a collection of named, typed columns. The arrow crate provides Rust bindings.

starsight accepts RecordBatch through the arrow feature flag. The data acceptance module extracts columns by name using the column method, which returns an ArrayRef (a reference-counted reference to an Array). The Array is downcast to Float64Array using as_any and downcast_ref. The values are accessed as a slice of f64.

The Arrow integration is simpler than the Polars integration because Arrow arrays are always contiguous in memory. There is no chunked array indirection. The downside is that Arrow has less ergonomic APIs for data manipulation (no built-in filter, group_by, or join), so users typically use a higher-level library like DataFusion on top of Arrow.

The ndarray integration is similarly straightforward. An ndarray Array2 of f64 provides column access via the column method, which returns a one-dimensional view. The view is contiguous in memory, so starsight can borrow the data without copying.

All three data source integrations (Polars, Arrow, ndarray) converge to the same internal representation: a pair of references to contiguous f64 slices for x and y data. This convergence happens in layer 5's data acceptance module. Everything below layer 5 (marks, scales, coordinates, backends) is agnostic to the original data source.

## How the starsight gallery generation works

The gallery is a collection of rendered chart images, one per chart type, that serves as both a visual reference and a test suite. The gallery is generated by the xtask crate and published as part of the documentation.

Each gallery entry is an example program that renders a specific chart type to PNG. The xtask gallery subcommand runs each example, collects the output files, generates a thumbnail for each (by rendering at a smaller size), and writes an HTML index page with all thumbnails linking to full-size images.

The gallery serves as a visual regression suite: if any chart type changes appearance, the gallery images change, and the gallery workflow in CI uploads the new images as artifacts. Reviewers can compare the old and new galleries to assess whether the visual changes are acceptable.

The gallery examples should use synthetic but realistic data. For a line chart, use a sine wave with noise. For a scatter plot, use clustered Gaussian data with clear separation. For a bar chart, use a small set of labeled values. For a heatmap, use a 2D Gaussian or a Mandelbrot set. For a 3D surface, use a mathematical surface like z equals sine of x times cosine of y. The data should be visually interesting and should exercise the chart type's key features (axis labels, legends, color scales, etc.).

The gallery HTML should be deployable to GitHub Pages. The gallery workflow generates the images and the HTML, then a deployment step copies them to the gh-pages branch. This makes the gallery available at resonant-jovian.github.io/starsight/gallery/.

## How different chart types map to the mark and stat system

Understanding how common chart types decompose into marks and stats is the key to implementing them without special-casing. Each chart type is a combination of one or more marks, zero or more stat transforms, and a coordinate system.

A scatter plot is a PointMark with x and y aesthetic mappings. No stat transform. Cartesian coordinates. Optionally, a color aesthetic maps a third variable to point color, and a size aesthetic maps a fourth variable to point radius. This is the simplest chart type and the one you should implement after LineMark.

A line chart is a LineMark with x and y aesthetic mappings. No stat transform. Cartesian coordinates. Multiple lines (series) are distinguished by the color aesthetic. The x data is typically ordered (monotonically increasing), and the LineMark connects points in order. If the data is not ordered, the line may cross itself, which is usually a user error but not something the library should prevent.

A histogram is a BarMark with a Bin stat transform. The Bin transform takes a single series of continuous values, divides them into bins (using Sturges' rule, the Freedman-Diaconis rule, or a user-specified bin count), and produces two arrays: bin center positions and bin counts. The BarMark renders each bin as a vertical bar from the baseline to the count value. The x scale is continuous (bin positions), and the y scale is continuous (counts).

A bar chart (as opposed to a histogram) is a BarMark with categorical x data. No stat transform is needed because the data already represents categorical values and their associated bar heights. The x scale is a BandScale (categorical positions with band widths). The y scale is continuous.

A stacked bar chart is a BarMark with a Stack position adjustment. The Stack transform computes cumulative sums within each category. Each series' bar starts where the previous series' bar ended. The rendering uses multiple BarMarks, each with a different color, layered from bottom to top.

An area chart is an AreaMark with x and y aesthetic mappings. The AreaMark is a closed path: the line goes from left to right, then drops to the baseline, runs back along the baseline, and closes. The fill color is set with some transparency to see overlapping areas.

A stacked area chart is multiple AreaMarks with a Stack position adjustment. Each area's baseline is the top of the previous area. The areas are rendered from bottom to top so that earlier series are behind later ones.

A heatmap is a RectMark (or a special HeatmapMark) with x, y, and color aesthetic mappings. The x and y are typically both categorical or both discrete grid positions. The color maps a continuous value to a prismatica colormap. Each cell is a filled rectangle.

A box plot is a BoxPlotMark with a Boxplot stat transform. The Boxplot transform computes Q1, median, Q3, whisker endpoints, and outlier positions from each group's data. The mark renders the box, whiskers, median line, and outlier dots. The x scale is categorical (one box per group) and the y scale is continuous.

A violin plot is a ViolinMark with a KDE stat transform. The KDE transform computes a density curve for each group. The mark renders the density curve mirrored around a center line. The x scale is categorical and the y scale is continuous.

A pie chart is an ArcMark on a polar coordinate system. The data values are normalized to sum to one, and each value maps to an angular extent. The ArcMark renders each slice as a filled arc. A donut chart is the same but with a hole in the center (inner radius greater than zero).

A contour plot is a ContourMark with a Contour stat transform. The Contour transform runs the marching squares algorithm on a 2D grid and produces a set of polylines at specified threshold values. The mark renders each polyline, colored according to the threshold value via a colormap.

A surface plot is a Surface3DMark on a 3D coordinate system. The data is a 2D grid of z-values. The mark renders a mesh of colored quadrilaterals, where the color at each vertex is determined by the z-value via a colormap. The 3D coordinate system handles the perspective projection to 2D screen coordinates.

A candlestick chart is a CandlestickMark with open, high, low, and close aesthetic mappings. Each data point represents one time period. The mark renders a vertical line from low to high (the wick) and a filled rectangle from open to close (the body). The body is green if close is greater than open (price went up) and red if close is less than open (price went down).

A dendrogram is a tree layout with lines connecting parent and child nodes. This is a specialized chart type that requires a hierarchical data structure, not just tabular data. The layout algorithm positions nodes in layers and draws orthogonal connector lines. This is one of the more complex chart types and is planned for the later milestones.

A Sankey diagram shows flows between nodes. The nodes are positioned in columns, and curved bands connect them, with the band width proportional to the flow magnitude. The layout algorithm positions nodes to minimize crossing, which is an optimization problem.

A treemap shows hierarchical data as nested rectangles. The squarified treemap algorithm divides a rectangle into sub-rectangles proportional to the data values, optimizing for aspect ratios close to 1 (squares are easier to compare than long thin rectangles).

Each of these chart types is a composition of marks, stats, scales, and coordinates. The grammar of graphics framework means you do not need 66 separate chart type implementations. You need about 15 marks, 10 stats, 8 scales, 3 coordinate systems, and a composition system that lets the user combine them.

## How to think about accessibility in chart design

Accessibility in data visualization means ensuring that charts communicate information to people with visual impairments, including color blindness, low vision, and blindness.

For color blindness (affecting about 8 percent of men and 0.5 percent of women), the primary mitigation is using color palettes that are distinguishable by people with all common forms of color vision deficiency. The most common form, deuteranopia, makes red and green appear similar. Protanopia has a similar effect but with different wavelengths. Tritanopia makes blue and yellow appear similar.

prismatica's perceptually uniform colormaps are designed with color vision deficiency in mind. The viridis colormap (and its variants inferno, magma, plasma) were specifically created to be distinguishable by people with all common forms of color blindness. starsight should default to these colormaps rather than rainbow colormaps (like jet) which are notoriously bad for accessibility.

Beyond colormaps, starsight should support redundant encoding: using both color and shape, or both color and pattern, to distinguish data series. A scatter plot where series A is blue circles and series B is orange squares is accessible to color-blind users because the shape distinction is sufficient even if the colors look similar.

For low vision, starsight should support configurable font sizes, line widths, and point sizes. The default values should be large enough to read at typical viewing distances (14 pixel font minimum for screen, 10 point minimum for print). High-contrast modes (black on white, white on black) should be available.

For blindness, the most accessible approach is to provide the underlying data table alongside the chart. This is straightforward for HTML export (include a hidden table element that screen readers can access) but not possible for static image formats. An alternative is to generate a text description of the chart: "Line chart showing temperature from January to December. The minimum is 5 degrees in January, the maximum is 32 degrees in July."

These accessibility features are not planned for 0.1.0 but should inform design decisions from the start: do not hardcode colors that are only distinguishable by people with full color vision, do not hardcode font sizes that are too small, and design the API so accessibility options can be added later without breaking changes.

## How to approach testing for a visualization library specifically

Testing a visualization library is different from testing a data processing library or a web framework. The output is visual, which means many bugs are invisible to standard assertions. You cannot assert that a chart "looks correct" — you can only assert that it matches a known-good reference, or that its numerical properties are correct.

The testing pyramid for starsight has four levels. At the base: unit tests for pure functions (scale mapping, tick generation, color conversion, coordinate math). These are fast, deterministic, and catch logic errors in the mathematical foundations. They do not catch rendering bugs.

The second level: snapshot tests for rendered output. These catch visual regressions: changes to how charts look. They are deterministic when using the CPU backend (tiny-skia) and should cover every chart type at a fixed size. The weakness is that they cannot distinguish between an intentional change (you improved the layout algorithm) and an unintentional change (you broke the layout algorithm). Human review is required when snapshots change.

The third level: property tests for mathematical invariants. These catch edge cases that unit tests miss: what happens when the data range is zero, when all values are NaN, when the array has one million elements, when the font size is 0.001, when the chart dimensions are 1 by 1 pixel. Property tests generate random inputs and check that invariants hold.

The fourth level: reference tests that compare output to other visualization libraries. Render the same data in starsight and matplotlib, and compare the results visually. This catches systematic errors where starsight's implementation diverges from established conventions (for example, if the Y axis is accidentally not inverted, all charts will be upside down). Reference tests are manual and run infrequently, but they validate that starsight produces charts that match user expectations formed by experience with other tools.

Do not aim for 100 percent code coverage. Some code paths (error handling for unlikely conditions, fallback rendering for missing fonts, format detection for file extensions) are difficult to test and unlikely to contain complex bugs. Aim for 80 percent coverage on library code and 100 percent coverage on the mathematical core (scales, ticks, coordinates, color conversion).

## How to manage the psychological complexity of a large codebase

A nine-crate workspace with a 66-chart-type goal can feel overwhelming. The file tree has hundreds of entries. The dependency graph has dozens of edges. The roadmap has hundreds of checkboxes. Managing this complexity without becoming paralyzed is a skill.

The first technique is to ignore most of the codebase most of the time. When you are implementing the LinearScale, the only files that matter are scale.rs and its tests. Layers 3 through 7, the backends, the examples, the CI configuration — none of these are relevant. Close every file except the one you are working on.

The second technique is to work in vertical slices, not horizontal layers. Do not implement all of layer 1 before starting layer 2. Instead, implement the minimum of layer 1 needed for layer 2, then the minimum of layer 2 needed for layer 3, and so on up to the first working chart. This gives you a working system at every step, which provides tangible feedback and motivation.

The third technique is to defer decisions. If you are not sure whether a parameter should be f32 or f64, pick one and move on. If you are not sure whether the Mark trait should have a data_range method or whether the Figure should compute ranges externally, pick one and move on. You can refactor later. The cost of a wrong decision that is refactored in 0.2.0 is much less than the cost of agonizing for a week.

The fourth technique is to keep a running list of "things I noticed but will not fix right now." As you work on one part of the codebase, you will notice potential improvements in other parts. Write them down in a TODO.md or in issue tracker tickets, then continue with your current task. This prevents the scope creep of "while I am here, I might as well also fix this" which leads to sprawling, never-finished PRs.

The fifth technique is to celebrate milestones. When plot save produces the first PNG, that is a milestone worth celebrating. When the first snapshot test passes, that is a milestone. When the second chart type renders correctly, that is a milestone. Share these moments publicly. They sustain motivation across the long timeline of a multi-year project.

## How starsight fits into the broader Rust ecosystem

The Rust ecosystem for data science and visualization is growing but fragmented. starsight positions itself as the comprehensive solution that bridges the gap between quick-and-dirty plotting (textplots, plotters) and full-featured interactive dashboards (plotly-rs, which bundles JavaScript).

The closest competitors are plotters (the most mature Rust plotting library, with good API documentation but limited chart types, stagnating development, and the Sized bound issue described earlier), plotly-rs (which generates Plotly.js charts and requires a JavaScript runtime or opens a browser tab), charming (which generates ECharts configurations and has the same JavaScript dependency), and egui_plot (which is excellent but locked to the egui framework).

starsight's differentiator is: no JavaScript runtime, no C dependencies in the default build, 66 chart types from a single library, both static export and interactive native windows, terminal rendering, GPU acceleration, and deep integration with the Rust data science stack (Polars, ndarray, Arrow). No existing Rust library offers all of these.

The risk is scope. Building a library this comprehensive takes years. Many Rust visualization projects have been abandoned after the initial enthusiasm. starsight mitigates this risk with a narrow initial scope (0.1.0 is just line charts and scatter plots), a sustainable development pace, and an architecture that allows incremental expansion without restructuring.

The opportunity is timing. The Rust data science ecosystem is maturing rapidly. Polars is approaching feature parity with pandas. ndarray is stable and widely used. Arrow support is standardized. The missing piece is visualization. The first Rust visualization library that reaches maturity will become the default choice for the ecosystem, just as matplotlib became the default for Python. starsight aims to be that library.

## How to handle NaN values throughout the pipeline

NaN (Not a Number) is a floating point value that represents undefined or missing data. It propagates through arithmetic: any operation involving NaN produces NaN. It also has the unusual property that NaN does not equal itself: the expression NaN equals NaN is false.

In starsight, NaN can appear in input data (sensor readings with gaps, database columns with null values converted to NaN), in computed values (log of a negative number, division by zero), and in intermediate results (scale mapping of a value outside the domain).

The design principle is: NaN should never produce a panic, a garbled chart, or an infinite loop. It should produce a visible gap in the chart and optionally a warning.

For scales, NaN input should produce NaN output. The map method should check for NaN and return NaN without performing the division. This prevents NaN from contaminating the domain computation (which uses min and max, and both should skip NaN values).

For marks, NaN in the data should produce a gap. The LineMark skips NaN values and starts a new path segment at the next non-NaN value. The PointMark skips NaN values and does not draw a point. The BarMark skips NaN values and leaves a gap in the bar sequence. This behavior should be documented prominently because it differs from some tools (matplotlib raises an error, plotly draws to zero).

For the tick algorithm, NaN in the data range should cause the algorithm to return a sensible default (like ticks at 0 and 1) rather than entering an infinite loop. The Wilkinson algorithm's loop termination depends on comparing scores, and NaN comparisons always return false, which can cause the loop to run indefinitely if not guarded.

For color interpolation, NaN in the lerp parameter should return a default color (typically the first color) rather than producing a NaN color (which would have NaN in its RGB channels, which makes no sense for u8 values).

Test NaN handling explicitly in every component. A property test that feeds NaN to every public function and asserts that no panic occurs is one of the highest-value tests in the suite.

## How to handle empty data

Empty data (zero-length arrays) is a valid input that every component must handle gracefully. A user might create an empty Figure and call save, or pass an empty DataFrame column, or filter data down to zero rows.

For scales, empty data means the domain is undefined. There is no minimum or maximum. The scale should return a degenerate domain (like 0 to 1) and produce a chart with default axis labels but no data content. This is more useful than returning an error because it lets the user see the chart skeleton (title, axis labels, legend) even when no data is available.

For marks, empty data means no visual elements. The mark should return Ok from its render method without drawing anything. It should not return an error.

For the tick algorithm, empty data or a zero-range domain (min equals max) should return a pair of ticks at sensible positions. If the domain is 5 to 5, the ticks might be 4 and 6 (expanding the range to show context around the single value).

For the Figure, if all marks have empty data, the chart is a blank canvas with axes. This is valid output and should not be an error.

## How to handle very large datasets efficiently

Scientific datasets can have millions or tens of millions of data points. A line chart with ten million points generates ten million PathCommand values, each producing a line segment. Drawing ten million line segments to a 800-by-600 pixmap is wasteful because most segments are shorter than a pixel and are invisible.

The solution is data decimation: reducing the number of points to a visually lossless representation. The simplest algorithm is min-max decimation: divide the data into buckets (one per pixel column), and for each bucket, keep only the minimum and maximum values. This preserves the visual envelope of the data (peaks and valleys) while reducing the point count to at most twice the pixel width.

A more sophisticated algorithm is the Largest Triangle Three Buckets (LTTB) algorithm, which selects representative points that preserve the visual shape of the line. LTTB produces better results than min-max for sparse data but is slightly slower.

Data decimation should happen transparently: the user passes all their data, and starsight internally decimates before rendering. The original data is preserved for interaction (hover should show the exact value, not the decimated value). This means the decimation result is only used for generating path commands, not for data storage.

For scatter plots with millions of points, the equivalent optimization is spatial binning (hexbin or quad-tree aggregation). Instead of drawing a million overlapping circles, aggregate nearby points and draw one circle per aggregate with size proportional to the count.

These optimizations are not needed for 0.1.0 but should be planned for 0.4.0 or 0.5.0, when users start reporting slow rendering on large datasets.

## How to handle concurrent access patterns

Even though starsight does not use async, users may want to render charts concurrently: generating multiple PNG files from different threads, or rendering charts in a web server request handler.

The key types and their thread safety: Color, Point, Vec2, Rect, and Size are Copy plus Send plus Sync, so they can be shared freely. LinearScale is Clone plus Send plus Sync, so it can be cloned into each thread. LineMark and PointMark are Send, so they can be moved to another thread, but not Sync (they contain Vec data that is not safe to share).

The SkiaBackend is Send but not Sync (the Pixmap is mutable state). Each thread must create its own SkiaBackend. This is fine because creating a backend is cheap (just allocating a pixel buffer).

The FontSystem is neither Send nor Sync in cosmic-text's default configuration. This means you cannot share a FontSystem across threads. Each thread must create its own. This is expensive (loading system fonts takes about a second). The mitigation is to create the FontSystem once per thread (using thread-local storage or a lazy static) and reuse it.

The Figure builder is not thread-safe (it accumulates mutable state). The pattern for concurrent rendering is: clone the data into each thread, build a Figure within each thread, and render independently. Do not share Figures across threads.

## How to choose sensible defaults for every parameter

Every configurable parameter needs a default value that produces a good result without user intervention. Choosing these defaults is one of the most important design decisions in a visualization library because most users never change the defaults.

Chart dimensions default to 800 by 600 pixels. This is the same default as matplotlib. It produces a landscape-oriented chart that fits comfortably on a modern screen and has reasonable proportions for most chart types.

Line width defaults to 2 pixels. This is thick enough to be clearly visible but thin enough that multiple overlapping lines remain distinguishable.

Point radius defaults to 3 pixels. This is large enough to click on (for interactive charts) and see clearly, but small enough that moderate datasets do not produce a solid mass of overlapping circles.

Font sizes default to: 16 pixels for the title, 12 pixels for axis labels, and 10 pixels for tick labels. These are relative to the chart size and may need to scale with DPI.

Margins default to: 60 pixels left (for Y axis tick labels and label), 40 pixels bottom (for X axis tick labels and label), 30 pixels top (for the title), and 20 pixels right (minimal padding). These are adjusted dynamically based on actual text widths after text shaping, but the defaults provide a starting point.

The default color cycle (for multiple series) uses 10 distinct colors that are colorblind-safe and visually pleasant. prismatica's Tableau10 palette is a good choice: it is widely recognized, well-tested for accessibility, and produces charts that look professional.

The default background is white. The default foreground (axis lines, tick labels, axis labels) is a dark gray, not pure black. Pure black on white creates harsh contrast. A dark gray like 0x333333 is more pleasant and more consistent with professional publication standards.

Grid lines default to off. This is a deliberate choice. Grid lines add visual clutter and are often unnecessary. Users who want grid lines can enable them explicitly. The default should produce a clean, uncluttered chart.

The axis line defaults to a single pixel dark gray line. Tick marks default to 5 pixels long, pointing outward from the plot area. These conventions match the "classic" matplotlib style, which is what most scientists and engineers expect.

The legend defaults to automatic placement in the upper right corner of the plot area, with a translucent white background to ensure readability over data. If the legend would overlap with data, it should ideally move to a less obstructive position, but this optimization is for later versions.

These defaults collectively define what a starsight chart looks like. Getting them right is more important than getting any individual feature right, because every chart that any user ever creates starts from these defaults.

## How to think about backward compatibility during rapid development

During the pre-1.0 phase, the API changes frequently. New features are added, design mistakes are corrected, type signatures are improved. But users adopt the library from the first published version. Every change you make to the API requires every user to update their code.

The tension is between API quality (improving the design by changing things) and user convenience (not breaking things). The resolution is to front-load the hardest design decisions: get the primitive types right before publishing 0.1.0, get the trait interfaces right before publishing 0.2.0, and get the builder patterns right before publishing 0.3.0. Once these foundations are stable, later versions can add features (new chart types, new backends, new data sources) without changing existing interfaces.

The specific things to get right early: the fields and methods on Point, Vec2, Rect, Color (because these types appear everywhere and are copied into user code), the methods on DrawBackend (because backend implementors depend on them), the methods on Scale and Mark (because every chart type and scale type depends on them), and the signature of the plot macro (because it is the first thing in every tutorial).

The specific things that can change later without much pain: the Figure builder's method names (builders are called in one place, easy to update), the layout algorithm's behavior (affects visual output but not API), the internal module structure of each layer crate (users only see the facade re-exports), and the set of available chart types (adding new ones is never breaking).

Use the pre-1.0 period wisely. This is the time when breaking changes are socially acceptable. After 1.0, every breaking change requires a major version bump, which fractures the ecosystem. Make the hard decisions now so that 1.0 is a stable foundation for years of compatible evolution.

## How to write effective error messages that help users fix their problems

Error messages are documentation. When something goes wrong, the error message is the only thing the user sees. A good error message tells the user what happened, why it happened, and what to do about it.

Bad error messages: "rendering failed", "invalid input", "error", "something went wrong." These tell the user nothing. They force the user to read the source code to understand what happened.

Good error messages: "failed to create pixmap: dimensions 0 by 0 are invalid; both width and height must be at least 1", "scale domain is empty: the minimum value 5.0 equals the maximum value 5.0; provide data with at least two distinct values or set the domain manually", "cannot save to path /tmp/chart.xyz: unknown file extension .xyz; supported extensions are .png and .svg."

The pattern is: what happened (failed to create pixmap), why (dimensions 0 by 0 are invalid), and what to do (both width and height must be at least 1). Not every error message needs all three parts, but the what part is always required, and the what-to-do part should be included whenever the fix is obvious.

For starsight, the error messages live in the code that constructs StarsightError variants. Each call site should include context about the specific operation. Instead of returning StarsightError Render of "failed", return StarsightError Render of "failed to stroke path: the path has 0 commands; ensure at least one MoveTo and one LineTo are present."

In thiserror, the Display format string is the error message. Write it in lowercase without a trailing period. Include variable context using format placeholders. If an error wraps a source error, the source is available via the standard Error chain and should not be duplicated in the message.

A useful practice: write the error message before writing the error-producing code. If you cannot explain what went wrong and how to fix it, you do not yet understand the failure mode well enough to handle it.

## How Rust's orphan rule affects cross-crate trait implementations

The orphan rule says: you can only implement a trait for a type if either the trait or the type (or both) is defined in the current crate. This prevents two crates from independently implementing the same trait for the same type, which would create ambiguity.

For starsight, the orphan rule affects color conversions. You want to implement From of chromata Color for starsight Color. This is allowed because starsight Color is defined in the current crate (starsight-layer-1). You also want to implement From of starsight Color for tiny_skia Color. This is NOT allowed because neither starsight Color nor tiny_skia Color is the standard From trait, and the From trait is from the standard library, not from your crate.

The workaround is a method instead of a trait implementation: add a to_tiny_skia method on starsight Color that returns a tiny_skia Color. This is not a From implementation, so the orphan rule does not apply. The downside is that you cannot use the Into syntax or the question mark operator for conversion. But since these conversions happen inside backend code (not in user-facing API), the ergonomic cost is acceptable.

Similarly, you cannot implement the ratatui Widget trait for a type defined in starsight unless the Widget trait is in scope. Since Widget is defined in the ratatui crate and starsight's widget type is defined in starsight, this IS allowed: the type is local. But if you wanted a type from prismatica to implement a trait from chromata, neither is local, and the orphan rule blocks it. This is why wrapper types (newtypes) exist: wrap the foreign type in a local newtype and implement the foreign trait on the newtype.

## How to structure documentation that serves multiple audiences

starsight has three audiences: casual users who want to plot data quickly, power users who want full control over chart composition, and contributors who want to understand the architecture and extend the library.

The rustdoc documentation serves all three but with different entry points. Casual users start at the starsight crate root documentation, which should have a quick-start example showing the plot macro. Power users navigate to specific types like Figure, LineMark, LinearScale, and CartesianCoord, which have detailed examples showing compositional usage. Contributors read the architecture documentation in the dot spec directory and the internal module-level doc comments that explain design decisions.

The README serves casual users exclusively. It should have: one sentence describing what starsight is, a quick-start code block, a feature table, a list of supported chart types (ideally with thumbnails from the gallery), and links to the full documentation.

The CONTRIBUTING.md serves contributors exclusively. It should cover: setup instructions, coding standards, PR process, testing requirements, and architectural overview with links to the spec document.

The changelog serves all three audiences: casual users check it before upgrading (to see if anything broke), power users check it for new features, and contributors check it to understand recent development direction.

Do not duplicate information across these documents. The README links to docs.rs for API details. The docs.rs documentation links to the spec for architecture decisions. The spec links to the README for the public-facing description. Each document has a single authoritative role and delegates everything else.

## How to monitor and respond to upstream dependency changes

starsight depends on about 30 external crates. Each of these crates is maintained by someone else and can release breaking changes, security fixes, or performance improvements at any time. Monitoring these changes and responding appropriately is an ongoing maintenance task.

Dependabot or Renovate (GitHub's automatic dependency update tools) can create PRs when new versions of dependencies are available. For starsight, enable Dependabot with a weekly schedule. Each Dependabot PR bumps one dependency to its latest version. The CI runs automatically on the PR, and if it passes, the update is safe to merge.

For major version bumps of important dependencies (like tiny-skia going from 0.12 to 0.13, or cosmic-text going from 0.18 to 0.19), manual review is necessary. Read the dependency's changelog to understand what changed. If the dependency's API changed, update starsight's backend code accordingly. If the dependency's output changed (for example, tiny-skia's anti-aliasing algorithm improved), re-run snapshot tests and review the visual changes.

cargo-deny's advisory check catches known security vulnerabilities in dependencies. The RustSec advisory database is updated frequently. New advisories can appear at any time, so the advisory check in CI may fail suddenly on an unrelated PR. The matrix strategy with continue-on-error on the advisory job handles this gracefully: the PR is not blocked, but the advisory failure is visible.

When an upstream dependency is abandoned (no releases for over a year, no response to issues), consider forking or finding an alternative. For tiny-skia, this is unlikely (it is actively maintained by the linebender project). For more niche dependencies, abandonment is a real risk. The architecture should not make starsight's correctness depend on any single optional dependency. Backend choices should be replaceable.

When an upstream dependency introduces a regression (a new version that breaks something), pin the dependency to the previous version in Cargo.toml until the regression is fixed upstream. Document the pin with a comment explaining the issue and linking to the upstream bug report. Remove the pin when the fix is released.

## How to think about the trade-off between compile time and runtime performance

Rust is a compiled language with a famously slow compiler. Every design decision that adds code — more generics, more trait implementations, more feature flag combinations — increases compile time. For a nine-crate workspace, compile time is already significant. Adding unnecessary abstraction makes it worse.

The specific trade-offs for starsight: generic functions are monomorphized for each concrete type, which generates more machine code and takes longer to compile. Trait objects use dynamic dispatch, which compiles faster but runs slower due to vtable indirection. For starsight, the choice depends on the layer. In layer 1 (rendering), use concrete types: the DrawBackend trait is a trait object, but the types it operates on (Point, Rect, Color, Path) are concrete. In layer 5 (user API), use generics sparingly: the plot macro generates monomorphized code for each input type, but the Figure builder uses trait objects for marks.

Proc macros (like thiserror's derive Error) add compile time because they run at compile time. Each proc macro invocation is a separate compilation step. For starsight, thiserror is the only proc macro dependency in the default feature set, and it runs only on the error enum (once). If you add more proc macros later (like serde Serialize for config structs), put them behind feature flags so they do not slow down the default build.

Conditional compilation via cfg attributes has zero compile time cost for the disabled code: the compiler does not even parse it. This is why feature flags are free for users who do not enable them. A user who only enables the default features compiles only the tiny-skia and SVG backends, not wgpu, ratatui, polars, or nalgebra.

The Swatinem rust-cache action in CI caches the target directory between runs, which amortizes the initial compile cost. A clean build of the starsight workspace takes about 60 to 90 seconds. An incremental rebuild after changing one file takes about 5 to 15 seconds. The cache reduces CI time by about 50 percent on average.

Profile-guided optimization and link-time optimization (LTO) improve runtime performance at the cost of compile time. LTO is enabled in the release profile (already configured). It combines all crates into a single compilation unit and optimizes across crate boundaries. This can reduce binary size by 10 to 30 percent and improve performance by 5 to 20 percent, but it makes release builds 2 to 5 times slower. For development, LTO is disabled (the dev profile uses opt-level 1 with no LTO).

## How to balance ambition with pragmatism

starsight's scope is ambitious: 66 chart types, 5 rendering backends, GPU acceleration, terminal rendering, WASM, interactivity, streaming, 3D, animation. This scope is achievable but not in a month or even a year. The danger is losing focus on the immediate goal (0.1.0: plot save produces a PNG) by getting distracted by the exciting later features (GPU rendering! 3D surfaces! WASM deployment!).

The remedy is strict milestone discipline. Do not write code for 0.6.0 features while 0.1.0 is incomplete. Do not design APIs for chart types that will not exist for six months. Do not optimize performance before the first chart renders correctly.

It is fine to make architectural decisions that accommodate future features (the Scene graph enables GPU rendering, the DrawBackend trait enables terminal rendering, the layer separation enables interactivity). But do not implement those features until their milestone. The architecture is ready. The implementation can wait.

Similarly, do not obsess over perfection in early milestones. The 0.1.0 line chart does not need perfect text positioning, optimal margin computation, or pixel-perfect anti-aliasing. It needs to produce a recognizable line chart from user data and save it to a file. Perfection comes through iteration, not through getting everything right the first time.

The practical test: at the end of each day, can you demonstrate something new? A new test passing, a new type implemented, a new chart rendering. If the answer is yes, you are making progress. If the answer is no for several days in a row, you are either stuck on a hard problem (take a break, ask for help) or distracted by scope creep (refocus on the current milestone).

## How to read the Wilkinson Extended paper

The original paper by Talbot, Lin, and Hanrahan from 2010 is titled "An Extension of Wilkinson's Algorithm for Positioning Tick Labels on Axes." It is published in the IEEE Transactions on Visualization and Computer Graphics. The paper is dense with mathematics but the core ideas are surprisingly intuitive.

The paper starts from the observation that existing tick algorithms (including the original Wilkinson 1999 algorithm and the R default algorithm) produce suboptimal tick positions in many common cases. They propose an improved algorithm that searches over a larger space of candidates and uses a more carefully designed scoring function.

The scoring function has four components. Simplicity measures how "round" the tick values are. Ticks at 0, 20, 40, 60, 80, 100 are simpler (rounder) than ticks at 3, 23, 43, 63, 83, 103. The simplicity score depends on which step base is used (1 is simpler than 5, which is simpler than 2) and whether zero is included as a tick (bonus points if yes).

Coverage measures how well the tick range covers the data range. If the data goes from 3.7 to 97.2 and the ticks go from 0 to 100, the coverage is good (the tick range includes all data). If the ticks go from 0 to 200, the coverage is poor (the extra 100 to 200 range is wasted space). Coverage penalizes both under-coverage (ticks do not extend to the data edges) and over-coverage (ticks extend far beyond the data).

Density measures how close the number of ticks is to the target count. If the user wants 5 to 7 ticks and the algorithm produces 6, the density is perfect. If it produces 12, the density is poor (too many). If it produces 3, the density is also poor (too few). The density component has the highest weight (0.5) because the number of ticks most directly affects readability.

Legibility is a catch-all for formatting concerns. The paper simplifies it to a constant in most implementations because the other three components capture the most important factors. A more sophisticated implementation might penalize tick labels that overlap, that are too long to fit in the available space, or that use scientific notation.

The search algorithm is a set of nested loops. The outer loop iterates over skip factors (j equals 1, 2, 3, and so on). The next loop iterates over the step bases (Q equals 1, 5, 2, 2.5, 4, 3). The next loop iterates over the number of ticks (k). The innermost loop iterates over the starting position. At each level, the algorithm computes an optimistic upper bound on the achievable score. If the upper bound is less than the best score found so far, it prunes the entire subtree.

The pruning makes the algorithm fast. In the paper's analysis, the average number of candidates evaluated is about 41, regardless of the data range or the target tick count. This means the algorithm runs in effectively constant time, which is fast enough for real-time interactive use.

When implementing this in Rust, the main challenges are: getting the floating point arithmetic right (accumulated rounding errors can cause off-by-one tick positions), handling the edge cases (zero-width data range, very large or very small data values), and formatting the tick labels correctly (removing trailing zeros, using appropriate precision).

## How to approach the implementation in the first coding session

When you sit down to code for the first time, do not open nine crates and try to understand the entire workspace. Open starsight-layer-1/src/primitives.rs and nothing else. This file already has Color, Point, Rect, and Size. Your first task is to add Vec2.

Type the struct definition. Add the derives. Add the new constructor. Add the constants (ZERO, X, Y). Add the length and normalize methods. Write the first test. Run cargo test minus p starsight-layer-1. See the test pass. Commit.

Then add the arithmetic implementations. Point minus Point gives Vec2. Point plus Vec2 gives Point. Write a test for each one. Run the tests. See them pass. Commit.

Then add the From implementations for arrays and tuples. Write tests. Commit.

Then move to Rect. Add the convenience constructors and accessors. Write tests. Commit.

Then move to Color. Add the constants, from_css_hex, to_css_hex, luminance, contrast_ratio, lerp. Write tests. Commit.

Then add Transform. Write tests. Commit.

At this point, you have a complete set of primitive types with tests. You have made about seven commits over one or two sessions. The codebase has grown by maybe 400 lines of library code and 300 lines of tests. cargo check passes. cargo test passes. cargo clippy is clean.

Now open starsight-layer-1/src/backend/skia/raster/mod.rs. This is where the tiny-skia backend lives. Create the SkiaBackend struct. Implement new. Implement fill. Implement fill_rect. Write a test that creates a 200 by 100 backend, fills it white, draws a blue rect, and encodes to PNG bytes. Set up insta. Create the first snapshot. Commit.

This is the rhythm. One type or one method at a time. Tests before or alongside the implementation. Frequent small commits. Each commit leaves the workspace in a passing state.

By the third session, you should have the SkiaBackend drawing paths and text. By the fifth session, you should have LinearScale and the tick algorithm. By the seventh session, you should have LineMark rendering through a CartesianCoord. By the tenth session, you should have the Figure builder and the plot macro producing the first PNG.

The first PNG is the proof that the architecture works. Everything after that is adding chart types, backends, and features to a foundation that you know is solid.


---
---

# Part 2 — Build

Every task below has enough detail to implement it without looking anything else up. Items are ordered by dependency. Do not skip ahead. When a task says to create a struct, it tells you the fields, the derives, the trait implementations, and why.

Checked items reflect the current state of the codebase as of 2026-03-31.

---

## Pre-0.1.0 — Workspace bootstrap

These are done. Listed for audit completeness.

- [x] Create resonant-jovian/starsight GitHub repository
- [x] Add GPL-3.0-only LICENSE
- [x] Create CONTRIBUTING.md, CODE_OF_CONDUCT.md, CHANGELOG.md, SECURITY.md
- [x] Create .github/ISSUE_TEMPLATE/ (bug_report.md, feature_request.md, config.yml)
- [x] Create .github/PULL_REQUEST_TEMPLATE.md
- [x] Create .github/FUNDING.yml
- [x] Initialize workspace Cargo.toml with resolver 3, edition 2024, all workspace members
- [x] Create all 8 crate Cargo.toml files (starsight, layer-1 through layer-7) with workspace inheritance
- [x] Create xtask/Cargo.toml
- [x] Define all feature flags in starsight/Cargo.toml
- [x] Configure workspace lints: unsafe_code forbid, clippy pedantic warn
- [x] Create .rustfmt.toml and .clippy.toml with full config
- [x] Create deny.toml for cargo-deny
- [x] Configure profile.release (LTO, codegen-units 1) and profile.dev (opt-level 1)
- [x] Create .github/workflows/ci.yml (fmt, clippy, check, test matrix, deny)
- [x] Create .github/workflows/release.yml (publish, GitHub release with git-cliff)
- [x] Create .github/workflows/coverage.yml (cargo-llvm-cov, Codecov upload)
- [x] Create .github/workflows/snapshots.yml (cargo insta test, artifact upload on failure)
- [x] Create .github/workflows/gallery.yml (xtask gallery, artifact upload)
- [x] Create README.md with badges, feature table, roadmap
- [x] Create starsight-layer-1/src/error.rs with StarsightError enum (7 variants) and Result type alias
- [x] Create starsight-layer-1/src/backend/mod.rs with DrawBackend trait (partial, some methods commented)
- [x] Create starsight-layer-1/src/primitives.rs with Color (r/g/b u8), Point (x/y f32), Rect (ltrb f32), Size (wh f32)
- [x] Create From<tiny_skia::Point> for Point, From<tiny_skia::Rect> for Rect, From<tiny_skia::Size> for Size
- [x] Create all stub module files for every backend (skia/, svg/, pdf/, wgpu/, terminal/)
- [x] Create all stub lib.rs files for layers 2-7
- [x] Verify cargo check --workspace passes
- [x] Verify cargo test --workspace passes (zero tests, zero failures)

---

## 0.1.0 — Foundation

Exit criteria: plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).save("test.png") produces a correct line chart PNG.

### Layer 1: Complete the primitive types

#### Add Vec2 with semantic arithmetic

- [x] Create `Vec2` in `starsight-layer-1/src/primitives.rs`. A Vec2 is a displacement, not a position. The grocery store minus your house is a Vec2. The grocery store itself is a Point.

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct Vec2 {
        pub x: f32,
        pub y: f32,
    }

    impl Vec2 {
        pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
        pub const X: Self = Self { x: 1.0, y: 0.0 };
        pub const Y: Self = Self { x: 0.0, y: 1.0 };

        pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }

        pub fn length(self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }

        pub fn normalize(self) -> Self {
            let len = self.length();
            if len == 0.0 { Self::ZERO } else { Self { x: self.x / len, y: self.y / len } }
        }
    }
    ```

- [ ] Implement the semantic arithmetic. This is the entire point of having two types. `Point - Point = Vec2` (displacement between positions). `Point + Vec2 = Point` (shift a position). `Point + Point` does not compile (meaningless). `Vec2 + Vec2 = Vec2` (compose displacements). `Vec2 * f32 = Vec2` (scale a displacement). `Point * f32` does not compile (scaling a position is meaningless).

    ```rust
    impl std::ops::Sub for Point {
        type Output = Vec2;
        fn sub(self, rhs: Point) -> Vec2 {
            Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
        }
    }

    impl std::ops::Add<Vec2> for Point {
        type Output = Point;
        fn add(self, rhs: Vec2) -> Point {
            Point { x: self.x + rhs.x, y: self.y + rhs.y }
        }
    }

    impl std::ops::Sub<Vec2> for Point {
        type Output = Point;
        fn sub(self, rhs: Vec2) -> Point {
            Point { x: self.x - rhs.x, y: self.y - rhs.y }
        }
    }

    impl std::ops::Add for Vec2 {
        type Output = Vec2;
        fn add(self, rhs: Vec2) -> Vec2 {
            Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
        }
    }

    impl std::ops::Sub for Vec2 {
        type Output = Vec2;
        fn sub(self, rhs: Vec2) -> Vec2 {
            Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
        }
    }

    impl std::ops::Mul<f32> for Vec2 {
        type Output = Vec2;
        fn mul(self, rhs: f32) -> Vec2 {
            Vec2 { x: self.x * rhs, y: self.y * rhs }
        }
    }

    impl std::ops::Mul<Vec2> for f32 {
        type Output = Vec2;
        fn mul(self, rhs: Vec2) -> Vec2 {
            Vec2 { x: self * rhs.x, y: self * rhs.y }
        }
    }

    impl std::ops::Neg for Vec2 {
        type Output = Vec2;
        fn neg(self) -> Vec2 {
            Vec2 { x: -self.x, y: -self.y }
        }
    }
    ```

- [ ] Add `From`/`Into` conversions for interop with other libraries:

    ```rust
    impl From<[f32; 2]> for Point { fn from([x, y]: [f32; 2]) -> Self { Self { x, y } } }
    impl From<(f32, f32)> for Point { fn from((x, y): (f32, f32)) -> Self { Self { x, y } } }
    impl From<Point> for [f32; 2] { fn from(p: Point) -> Self { [p.x, p.y] } }
    impl From<Point> for (f32, f32) { fn from(p: Point) -> Self { (p.x, p.y) } }
    // Same four impls for Vec2
    ```

- [ ] Write tests:

    ```rust
    #[test]
    fn point_minus_point_is_vec2() {
        let a = Point::new(10.0, 20.0);
        let b = Point::new(3.0, 5.0);
        let v: Vec2 = a - b;
        assert_eq!(v, Vec2::new(7.0, 15.0));
    }

    #[test]
    fn point_plus_vec2_is_point() {
        let p = Point::new(1.0, 2.0);
        let v = Vec2::new(10.0, 20.0);
        let result: Point = p + v;
        assert_eq!(result, Point::new(11.0, 22.0));
    }

    #[test]
    fn vec2_scale() {
        assert_eq!(Vec2::new(3.0, 4.0) * 2.0, Vec2::new(6.0, 8.0));
        assert_eq!(2.0 * Vec2::new(3.0, 4.0), Vec2::new(6.0, 8.0));
    }

    #[test]
    fn vec2_length() {
        assert!((Vec2::new(3.0, 4.0).length() - 5.0).abs() < f32::EPSILON);
    }
    ```

#### Complete the Rect type

- [ ] Add convenience constructors and accessors:

    ```rust
    impl Rect {
        pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
            Self { left: x, top: y, right: x + width, bottom: y + height }
        }

        pub fn from_center_size(center: Point, size: Size) -> Self {
            let half_w = size.width * 0.5;
            let half_h = size.height * 0.5;
            Self { left: center.x - half_w, top: center.y - half_h,
                   right: center.x + half_w, bottom: center.y + half_h }
        }

        pub fn width(&self) -> f32 { self.right - self.left }
        pub fn height(&self) -> f32 { self.bottom - self.top }
        pub fn size(&self) -> Size { Size::new(self.width(), self.height()) }
        pub fn center(&self) -> Point {
            Point::new((self.left + self.right) * 0.5, (self.top + self.bottom) * 0.5)
        }
        pub fn top_left(&self) -> Point { Point::new(self.left, self.top) }
        pub fn bottom_right(&self) -> Point { Point::new(self.right, self.bottom) }

        pub fn contains(&self, p: Point) -> bool {
            p.x >= self.left && p.x <= self.right && p.y >= self.top && p.y <= self.bottom
        }

        pub fn intersection(&self, other: &Rect) -> Option<Rect> {
            let r = Rect {
                left: self.left.max(other.left), top: self.top.max(other.top),
                right: self.right.min(other.right), bottom: self.bottom.min(other.bottom),
            };
            if r.left < r.right && r.top < r.bottom { Some(r) } else { None }
        }

        pub fn pad(&self, amount: f32) -> Rect {
            Rect { left: self.left - amount, top: self.top - amount,
                   right: self.right + amount, bottom: self.bottom + amount }
        }

        /// Returns None if left >= right or top >= bottom.
        pub fn to_tiny_skia(&self) -> Option<tiny_skia::Rect> {
            tiny_skia::Rect::from_ltrb(self.left, self.top, self.right, self.bottom)
        }
    }
    ```

- [ ] Add derives: `#[derive(Debug, Clone, Copy, PartialEq)]` (already have some, verify all present). Add `Display`:

    ```rust
    impl std::fmt::Display for Rect {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Rect({}, {}, {}, {})", self.left, self.top, self.right, self.bottom)
        }
    }
    ```

#### Complete the Color type

- [ ] Add `ColorAlpha` and core Color methods:

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Color { pub r: u8, pub g: u8, pub b: u8 }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ColorAlpha { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

    impl Color {
        pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
        pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
        pub const RED: Self = Self { r: 255, g: 0, b: 0 };
        pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
        pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };

        pub const fn to_f32(self) -> (f32, f32, f32) {
            (self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0)
        }

        pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
            Self {
                r: (r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                g: (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                b: (b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            }
        }

        pub fn to_tiny_skia(self) -> tiny_skia::Color {
            tiny_skia::Color::from_rgba8(self.r, self.g, self.b, 255)
        }

        pub fn with_alpha(self, a: u8) -> ColorAlpha {
            ColorAlpha { r: self.r, g: self.g, b: self.b, a }
        }
    }
    ```

- [ ] Add `from_css_hex` and `to_css_hex`:

    ```rust
    impl Color {
        pub fn from_css_hex(s: &str) -> Option<Self> {
            let hex = s.strip_prefix('#').unwrap_or(s);
            match hex.len() {
                6 => {
                    let val = u32::from_str_radix(hex, 16).ok()?;
                    Some(Self::from_hex(val))
                }
                3 => {
                    let mut chars = hex.chars();
                    let r = chars.next().and_then(|c| c.to_digit(16))? as u8;
                    let g = chars.next().and_then(|c| c.to_digit(16))? as u8;
                    let b = chars.next().and_then(|c| c.to_digit(16))? as u8;
                    Some(Self { r: r << 4 | r, g: g << 4 | g, b: b << 4 | b })
                }
                _ => None,
            }
        }

        pub fn to_css_hex(self) -> String {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }
    }

    impl std::fmt::Display for Color {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }
    }
    ```

- [ ] Add `luminance`, `contrast_ratio`, `lerp`:

    ```rust
    impl Color {
        pub fn luminance(self) -> f64 {
            fn linearize(c: f64) -> f64 {
                if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
            }
            let r = linearize(self.r as f64 / 255.0);
            let g = linearize(self.g as f64 / 255.0);
            let b = linearize(self.b as f64 / 255.0);
            0.2126 * r + 0.7152 * g + 0.0722 * b
        }

        pub fn contrast_ratio(self, other: Color) -> f64 {
            let l1 = self.luminance();
            let l2 = other.luminance();
            let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
            (lighter + 0.05) / (darker + 0.05)
        }

        pub fn lerp(self, other: Color, t: f32) -> Color {
            let t = t.clamp(0.0, 1.0);
            Color {
                r: (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
                g: (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
                b: (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
            }
        }
    }
    ```

- [ ] Add sister crate conversions:

    ```rust
    impl From<chromata::Color> for Color {
        fn from(c: chromata::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    impl From<prismatica::Color> for Color {
        fn from(c: prismatica::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    ```

- [ ] Write tests: `from_hex` roundtrip, `from_css_hex` with all formats, luminance black ≈ 0, luminance white ≈ 1, contrast black/white ≈ 21, lerp at 0.0 returns self, lerp at 1.0 returns other.

#### Add the Transform type

- [ ] Create a `Transform` newtype wrapping `tiny_skia::Transform`:

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Transform(pub(crate) tiny_skia::Transform);

    impl Transform {
        pub fn identity() -> Self { Self(tiny_skia::Transform::identity()) }
        pub fn translate(dx: f32, dy: f32) -> Self { Self(tiny_skia::Transform::from_translate(dx, dy)) }
        pub fn scale(sx: f32, sy: f32) -> Self { Self(tiny_skia::Transform::from_scale(sx, sy)) }
        /// NOTE: tiny-skia takes DEGREES, not radians.
        pub fn rotate_degrees(angle: f32) -> Self { Self(tiny_skia::Transform::from_rotate(angle)) }

        pub fn then(self, other: Transform) -> Self { Self(self.0.post_concat(other.0)) }
        pub fn pre_translate(self, dx: f32, dy: f32) -> Self { Self(self.0.pre_translate(dx, dy)) }

        pub(crate) fn as_tiny_skia(self) -> tiny_skia::Transform { self.0 }
    }
    ```

### Layer 1: Implement the tiny-skia backend

#### Create the SkiaBackend struct

- [ ] Create `starsight-layer-1/src/backend/skia/raster/mod.rs`:

    ```rust
    use tiny_skia::{Pixmap, Paint, FillRule, Stroke, LineCap, LineJoin, PathBuilder};
    use crate::error::{Result, StarsightError};
    use crate::primitives::{Color, Point, Rect, Transform};
    use super::super::DrawBackend;

    pub struct SkiaBackend {
        pixmap: Pixmap,
        font_system: cosmic_text::FontSystem,
        swash_cache: cosmic_text::SwashCache,
    }

    impl SkiaBackend {
        pub fn new(width: u32, height: u32) -> Result<Self> {
            let pixmap = Pixmap::new(width, height)
                .ok_or_else(|| StarsightError::Render(
                    format!("Failed to create {width}x{height} pixmap")
                ))?;
            Ok(Self {
                pixmap,
                font_system: cosmic_text::FontSystem::new(),
                swash_cache: cosmic_text::SwashCache::new(),
            })
        }

        pub fn fill(&mut self, color: Color) {
            self.pixmap.fill(color.to_tiny_skia());
        }

        pub fn png_bytes(&self) -> Result<Vec<u8>> {
            self.pixmap.encode_png().map_err(|e| StarsightError::Export(e.to_string()))
        }
    }
    ```

- [ ] Implement `DrawBackend` for `SkiaBackend`. The key methods:

    ```rust
    impl DrawBackend for SkiaBackend {
        fn dimensions(&self) -> (u32, u32) {
            (self.pixmap.width(), self.pixmap.height())
        }

        fn save_png(&self, path: &std::path::Path) -> Result<()> {
            self.pixmap.save_png(path)
                .map_err(|e| StarsightError::Export(e.to_string()))
        }

        fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
            let sk_rect = rect.to_tiny_skia()
                .ok_or_else(|| StarsightError::Render("Invalid rect".into()))?;
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, 255);
            self.pixmap.fill_rect(sk_rect, &paint,
                tiny_skia::Transform::identity(), None);
            Ok(())
        }

        fn draw_path(&mut self, path: &crate::backend::Path,
                     style: &crate::backend::PathStyle) -> Result<()> {
            // Convert PathCommand sequence to tiny_skia::Path
            let mut pb = PathBuilder::new();
            for cmd in path.commands() {
                match cmd {
                    PathCommand::MoveTo(p) => pb.move_to(p.x, p.y),
                    PathCommand::LineTo(p) => pb.line_to(p.x, p.y),
                    PathCommand::QuadTo(c, p) => pb.quad_to(c.x, c.y, p.x, p.y),
                    PathCommand::CubicTo(c1, c2, p) =>
                        pb.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y),
                    PathCommand::Close => pb.close(),
                }
            }
            let sk_path = pb.finish()
                .ok_or_else(|| StarsightError::Render("Empty path".into()))?;

            let mut paint = Paint::default();
            paint.set_color_rgba8(style.stroke_color.r, style.stroke_color.g,
                                  style.stroke_color.b, 255);
            let stroke = Stroke {
                width: style.stroke_width,
                line_cap: style.line_cap,
                line_join: style.line_join,
                dash: style.dash_pattern.and_then(|(len, gap)|
                    tiny_skia::StrokeDash::new(vec![len, gap], 0.0)),
                ..Stroke::default()
            };
            self.pixmap.stroke_path(&sk_path, &paint, &stroke,
                tiny_skia::Transform::identity(), None);
            Ok(())
        }

        // draw_text and save_svg omitted for brevity — see Look up section
    }
    ```

- [ ] Uncomment the commented-out methods and `PathCommand` variants in `backend/mod.rs`:

    ```rust
    pub enum PathCommand {
        MoveTo(Point),
        LineTo(Point),
        QuadTo(Point, Point),
        CubicTo(Point, Point, Point),
        Close,
    }

    pub struct PathStyle {
        pub stroke_color: Color,
        pub stroke_width: f32,
        pub fill_color: Option<Color>,
        pub dash_pattern: Option<(f32, f32)>,
        pub line_cap: tiny_skia::LineCap,
        pub line_join: tiny_skia::LineJoin,
        pub opacity: f32,
    }
    ```

#### Set up snapshot testing

- [ ] Add to root `Cargo.toml`:

    ```toml
    [workspace.dependencies]
    insta = { version = "1.47.2", features = ["binary"] }
    ```

- [ ] Create `starsight-layer-1/tests/snapshot_basic.rs`:

    ```rust
    use starsight_layer_1::backend::skia::raster::SkiaBackend;
    use starsight_layer_1::primitives::{Color, Rect};

    #[test]
    fn blue_rect_on_white() {
        let mut backend = SkiaBackend::new(200, 100).unwrap();
        backend.fill(Color::WHITE);
        backend.fill_rect(Rect::from_xywh(10.0, 10.0, 180.0, 80.0), Color::BLUE).unwrap();
        let bytes = backend.png_bytes().unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```

    Run `cargo insta test`, then `cargo insta review` to accept.

### Layer 1: Implement the SVG backend

- [ ] Create starsight-layer-1/src/backend/svg/mod.rs with an SvgBackend struct. It holds an svg::Document and the dimensions (width: u32, height: u32). The constructor takes dimensions and creates a Document with the viewBox attribute set.

- [ ] Implement DrawBackend for SvgBackend. fill_rect adds a Rectangle element with x, y, width, height, and fill attributes. draw_path converts PathCommands to SVG path data using svg::node::element::path::Data. draw_text adds a Text element with x, y, font-size, text-anchor, and dominant-baseline attributes.

- [ ] Implement save_svg: call svg::save(path, &self.document) and map errors.

- [ ] Implement save_png: this is not directly supported by the SVG backend. Return StarsightError::Export("SVG backend cannot save PNG directly; use the skia backend or resvg").

- [ ] Write a snapshot test that generates SVG output for a simple chart and asserts the SVG string content with assert_snapshot!.

### Layer 2: Linear scale and Wilkinson ticks

- [ ] Create `starsight-layer-2/src/scale.rs`:

    ```rust
    pub trait Scale {
        fn map(&self, value: f64) -> f64;
        fn inverse(&self, normalized: f64) -> f64;
    }

    pub struct LinearScale {
        pub domain_min: f64,
        pub domain_max: f64,
    }

    impl Scale for LinearScale {
        fn map(&self, value: f64) -> f64 {
            if (self.domain_max - self.domain_min).abs() < f64::EPSILON { return 0.5; }
            (value - self.domain_min) / (self.domain_max - self.domain_min)
        }
        fn inverse(&self, normalized: f64) -> f64 {
            normalized * (self.domain_max - self.domain_min) + self.domain_min
        }
    }
    ```

- [ ] Create `starsight-layer-2/src/tick.rs` with the Wilkinson Extended algorithm. See Part 1 "Listen" for the full explanation. See Part 3 "Look up" for the scoring formula.

    ```rust
    pub fn extended_ticks(dmin: f64, dmax: f64, target_count: usize) -> Vec<f64> {
        let q_list = &[1.0, 5.0, 2.0, 2.5, 4.0, 3.0];
        let w = [0.2, 0.25, 0.5, 0.05];
        let mut best_score = -2.0_f64;
        let mut best = (0.0, 0.0, 0.0_f64);
        // Nested loops: j (skip), q (step base), k (tick count), z (power of 10), start
        // At each level, compute upper bound and break if no candidate can beat best_score
        // Full pseudocode in Part 3
        todo!("implement the nested loop with pruning")
    }
    ```

    Tests: `extended_ticks(0.0, 100.0, 5)` returns round numbers, always sorted, always >= 2 elements.

- [ ] Create `starsight-layer-2/src/axis.rs`:

    ```rust
    pub struct Axis {
        pub scale: LinearScale,
        pub label: Option<String>,
        pub tick_positions: Vec<f64>,
        pub tick_labels: Vec<String>,
    }

    impl Axis {
        pub fn auto_from_data(values: &[f64], target_ticks: usize) -> Self {
            let dmin = values.iter().copied().fold(f64::INFINITY, f64::min);
            let dmax = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let ticks = crate::tick::extended_ticks(dmin, dmax, target_ticks);
            let labels: Vec<String> = ticks.iter().map(|t| format!("{t}")).collect();
            Self {
                scale: LinearScale { domain_min: ticks[0], domain_max: *ticks.last().unwrap() },
                label: None, tick_positions: ticks, tick_labels: labels,
            }
        }
    }
    ```

- [ ] Create `starsight-layer-2/src/coord.rs`:

    ```rust
    pub struct CartesianCoord {
        pub x_axis: Axis,
        pub y_axis: Axis,
        pub plot_area: Rect,
    }

    impl CartesianCoord {
        pub fn data_to_pixel(&self, x: f64, y: f64) -> Point {
            let nx = self.x_axis.scale.map(x);
            let ny = self.y_axis.scale.map(y);
            Point::new(
                self.plot_area.left + nx as f32 * self.plot_area.width(),
                self.plot_area.bottom - ny as f32 * self.plot_area.height(),
            )
        }
    }
    ```

### Layer 3: Line mark and point mark

- [ ] Create `starsight-layer-3/src/mark.rs`:

    ```rust
    use starsight_layer_1::backend::DrawBackend;
    use starsight_layer_1::error::Result;
    use starsight_layer_2::coord::CartesianCoord;

    pub trait Mark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()>;
    }
    ```

- [ ] Create `starsight-layer-3/src/line.rs`. Handle NaN by starting a new `MoveTo` (breaks the line at gaps):

    ```rust
    pub struct LineMark {
        pub x_data: Vec<f64>,
        pub y_data: Vec<f64>,
        pub color: Color,
        pub width: f32,
    }

    impl Mark for LineMark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
            let mut commands = Vec::new();
            let mut need_move = true;
            for (x, y) in self.x_data.iter().zip(&self.y_data) {
                if x.is_nan() || y.is_nan() { need_move = true; continue; }
                let p = coord.data_to_pixel(*x, *y);
                if need_move { commands.push(PathCommand::MoveTo(p)); need_move = false; }
                else { commands.push(PathCommand::LineTo(p)); }
            }
            let path = Path { commands };
            let style = PathStyle {
                stroke_color: self.color,
                stroke_width: self.width,
                fill_color: None,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Default::default()
            };
            backend.draw_path(&path, &style)
        }
    }
    ```

- [ ] Create `starsight-layer-3/src/point.rs`. Batch all circles into one path for performance:

    ```rust
    pub struct PointMark {
        pub x_data: Vec<f64>,
        pub y_data: Vec<f64>,
        pub color: Color,
        pub radius: f32,
    }

    impl Mark for PointMark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
            // Batch: collect all pixel positions, draw as one filled path
            let mut commands = Vec::new();
            for (x, y) in self.x_data.iter().zip(&self.y_data) {
                if x.is_nan() || y.is_nan() { continue; }
                let p = coord.data_to_pixel(*x, *y);
                // Approximate circle with 4 cubic bezier arcs
                // Or: backend could have a draw_circles batch method
                commands.push(PathCommand::MoveTo(Point::new(p.x + self.radius, p.y)));
                // ... arc approximation commands
            }
            // Alternative: use backend-specific circle batching
            todo!("circle rendering")
        }
    }
    ```

### Layer 5: Figure builder and plot macro

- [ ] Create `starsight-layer-5/src/figure.rs`:

    ```rust
    use starsight_layer_3::mark::Mark;

    pub struct Figure {
        marks: Vec<Box<dyn Mark>>,
        pub x_label: Option<String>,
        pub y_label: Option<String>,
        pub title: Option<String>,
        pub width: u32,
        pub height: u32,
    }

    impl Figure {
        pub fn new() -> Self {
            Self { marks: Vec::new(), x_label: None, y_label: None,
                   title: None, width: 800, height: 600 }
        }
        pub fn title(&mut self, s: impl Into<String>) -> &mut Self { self.title = Some(s.into()); self }
        pub fn x_label(&mut self, s: impl Into<String>) -> &mut Self { self.x_label = Some(s.into()); self }
        pub fn y_label(&mut self, s: impl Into<String>) -> &mut Self { self.y_label = Some(s.into()); self }
        pub fn size(&mut self, w: u32, h: u32) -> &mut Self { self.width = w; self.height = h; self }
        pub fn add(&mut self, mark: impl Mark + 'static) -> &mut Self {
            self.marks.push(Box::new(mark)); self
        }

        pub fn save(&self, path: impl AsRef<std::path::Path>) -> starsight_layer_1::error::Result<()> {
            let mut backend = starsight_layer_1::backend::skia::raster::SkiaBackend::new(self.width, self.height)?;
            backend.fill(Color::WHITE);
            // Compute plot area, create CartesianCoord, render axes, render marks
            todo!("full render pipeline")
        }
    }
    ```

- [ ] Create the `plot!` macro in `starsight-layer-5/src/macros.rs`:

    ```rust
    #[macro_export]
    macro_rules! plot {
        ($x:expr, $y:expr $(,)?) => {{
            let mut fig = $crate::figure::Figure::new();
            fig.add($crate::line::LineMark {
                x_data: $x.into_iter().map(|v| v as f64).collect(),
                y_data: $y.into_iter().map(|v| v as f64).collect(),
                color: starsight_layer_1::primitives::Color::BLUE,
                width: 2.0,
            });
            fig
        }};
    }
    ```

- [ ] Wire the facade. In `starsight/src/lib.rs`:

    ```rust
    pub use starsight_layer_1 as layer1;
    pub use starsight_layer_2 as layer2;
    pub use starsight_layer_3 as layer3;
    pub use starsight_layer_4 as layer4;
    pub use starsight_layer_5 as layer5;
    pub use starsight_layer_6 as layer6;
    pub use starsight_layer_7 as layer7;
    pub mod prelude;
    ```

    In `starsight/src/prelude.rs`:

    ```rust
    pub use starsight_layer_1::primitives::{Color, Point, Vec2, Rect, Size};
    pub use starsight_layer_1::error::{StarsightError, Result};
    pub use starsight_layer_5::figure::Figure;
    pub use starsight_layer_5::plot;
    ```

- [ ] Write the integration test in `starsight/tests/integration.rs`:

    ```rust
    use starsight::prelude::*;

    #[test]
    fn quickstart_produces_png() {
        let fig = plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let tmp = std::env::temp_dir().join("starsight_test.png");
        fig.save(&tmp).unwrap();
        assert!(tmp.exists());
        assert!(std::fs::metadata(&tmp).unwrap().len() > 0);
        std::fs::remove_file(&tmp).ok();
    }
    ```


## 0.2.0 through 1.0.0 — Remaining milestones

These are abbreviated. Expand each when the previous milestone is complete.

### 0.2.0 — Core chart types part 1

- [ ] BarMark (vertical and horizontal bars, grouped and stacked)
- [ ] AreaMark (filled area between line and baseline)
- [ ] Histogram stat transform (bin data into counts)
- [ ] HeatmapMark (2D color grid from matrix data)
- [ ] Snapshot tests for all four

### 0.3.0 — Core chart types part 2

- [ ] BoxPlotMark (compute quartiles, whiskers, outliers)
- [ ] ViolinMark (KDE mirrored vertically)
- [ ] KDE stat transform (kernel density estimation)
- [ ] PieMark and DonutMark (arc geometry)
- [ ] ContourMark (isolines from 2D scalar field)
- [ ] CandlestickMark (OHLC financial chart)
- [ ] Polars DataFrame integration in layer 5 (accept &DataFrame, reference columns by name)
- [ ] Snapshot tests for all

### 0.4.0 — Layout and composition

- [ ] GridLayout in layer 4 (arrange multiple figures in rows/columns)
- [ ] FacetWrap (one subplot per category value, wrapping to multiple rows)
- [ ] FacetGrid (row and column faceting variables)
- [ ] Legend (map visual encodings back to data labels)
- [ ] Colorbar (continuous color scale display)
- [ ] PairPlot shorthand (scatter matrix)
- [ ] JointPlot shorthand (scatter with marginal distributions)

### 0.5.0 — Scale infrastructure

- [ ] LogScale, SymlogScale (symmetric log for data spanning zero)
- [ ] DateTimeScale (auto tick granularity: year/month/day/hour/minute/second)
- [ ] BandScale, CategoricalScale (discrete axis positions)
- [ ] ColorScale backed by prismatica (Sequential, Diverging, Qualitative)
- [ ] TickLocator and TickFormatter traits for custom tick logic

### 0.6.0 — GPU and interactivity

- [ ] wgpu DrawBackend in starsight-layer-1/src/backend/wgpu/
- [ ] Native window via winit in layer 6
- [ ] Hover tooltips, box zoom, wheel zoom, pan
- [ ] Legend click-to-toggle visibility
- [ ] Streaming data append with rolling window

### 0.7.0 — 3D visualization

- [ ] Scatter3D, Surface3D, Wireframe3D, Line3D
- [ ] Camera orbit/pan with nalgebra transforms
- [ ] Isosurface, VolumeRender

### 0.8.0 — Terminal backend

- [ ] Kitty graphics protocol output
- [ ] Sixel output
- [ ] iTerm2 inline images
- [ ] Half-block and Braille character rendering
- [ ] StarsightWidget implementing ratatui::Widget
- [ ] Automatic protocol detection

### 0.9.0 — All chart types

- [ ] Complete the remaining 40+ mark types from the taxonomy
- [ ] Snapshot test for every one

### 0.10.0 — Export and WASM

- [ ] PDF export via krilla
- [ ] Self-contained interactive HTML export
- [ ] GIF animation export
- [ ] WASM + WebGPU browser target

### 0.11.0 — Polish

- [ ] Recipe proc macro (#[starsight::recipe])
- [ ] ndarray and Arrow RecordBatch data acceptance
- [ ] API audit against Rust API Guidelines checklist

### 0.12.0 — Documentation

- [ ] Rustdoc for every public item
- [ ] 12 example programs
- [ ] Gallery generation via xtask
- [ ] docs.rs configuration

### 1.0.0 — Stable release

- [ ] cargo-semver-checks pass
- [ ] Full CI green on all platforms
- [ ] Announcement


---
---

# Part 3 — Look up

Quick-reference for type signatures, API details, conversion formulas, and dependency specifics. Come here mid-implementation when you need to check something.

---

## tiny-skia 0.12 API reference

### Color types

| Type | Fields | Alpha | Constructor | Returns |
|------|--------|-------|-------------|---------|
| `Color` | f32 × 4 | Straight | `from_rgba(r,g,b,a)` | `Option<Self>` (None if out of 0.0-1.0) |
| `Color` | f32 × 4 | Straight | `from_rgba8(r,g,b,a)` | `Self` (infallible) |
| `ColorU8` | u8 × 4 | Straight | `from_rgba(r,g,b,a)` | `Self` (const, infallible) |
| `PremultipliedColorU8` | u8 × 4 | Premultiplied | `from_rgba(r,g,b,a)` | `Option<Self>` (None if channel > alpha) |

### Drawing methods (all take `Option<&Mask>` as final param)

```rust
pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
pixmap.fill_rect(rect, &paint, Transform::identity(), None);
pixmap.draw_pixmap(x: i32, y: i32, pixmap_ref, &pixmap_paint, Transform::identity(), None);
```

### PathBuilder

```rust
let mut pb = PathBuilder::new();
pb.move_to(x, y);
pb.line_to(x, y);
pb.quad_to(x1, y1, x, y);
pb.cubic_to(x1, y1, x2, y2, x, y);
pb.close();
pb.push_circle(cx, cy, r);          // add circle to existing builder
let path: Option<Path> = pb.finish(); // None if empty
```

Static constructors: `PathBuilder::from_rect(rect) -> Path`, `PathBuilder::from_circle(cx, cy, r) -> Option<Path>`.

### Stroke

```rust
Stroke {
    width: 2.0,
    miter_limit: 4.0,
    line_cap: LineCap::Round,    // Butt | Round | Square
    line_join: LineJoin::Round,  // Miter | MiterClip | Round | Bevel
    dash: StrokeDash::new(vec![10.0, 5.0], 0.0), // returns Option
}
```

### Transform — DEGREES not radians

```rust
Transform::identity()
Transform::from_translate(tx, ty)
Transform::from_scale(sx, sy)
Transform::from_rotate(degrees)              // NOT radians
Transform::from_rotate_at(degrees, cx, cy)
t.pre_translate(tx, ty)
t.post_concat(other)
```

### PNG export

```rust
pixmap.save_png("file.png")?;                          // to file
let bytes: Vec<u8> = pixmap.encode_png()?;             // to memory
// DPI: 300 DPI = 11811 pixels/meter
```

---

## cosmic-text 0.18 API reference

```rust
let mut font_system = FontSystem::new();           // loads system fonts (~1s)
let mut swash_cache = SwashCache::new();           // no params
let metrics = Metrics::new(14.0, 20.0);            // font_size, line_height (f32 px)
let mut buffer = Buffer::new(&mut font_system, metrics);
buffer.set_text(&mut font_system, "text", &Attrs::new(), Shaping::Advanced, None);
buffer.set_size(&mut font_system, Some(width), Some(height));
buffer.shape_until_scroll(&mut font_system, true);
```

### Measure text dimensions

```rust
let (mut w, mut h) = (0.0f32, 0.0f32);
for run in buffer.layout_runs() {
    w = w.max(run.line_w);
    h = run.line_top + run.line_height;
}
```

### Draw onto tiny-skia (NO channel swap for file output)

```rust
buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    if let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
});
```

### Embed custom font

```rust
font_system.db_mut().load_font_data(include_bytes!("fonts/Inter.ttf").to_vec());
```

---

## prismatica API reference

```rust
// Continuous colormap — sample at t in [0,1]
let color: Color = prismatica::crameri::BATLOW.eval(0.5);
let color: Color = prismatica::crameri::BATLOW.eval_rational(5, 10);

// Reversed (zero allocation)
let rev = prismatica::crameri::BATLOW.reversed();

// Discrete palette — categorical data
let color: Color = prismatica::colorbrewer::SET2_PALETTE.get(0); // wraps around

// Metadata
prismatica::crameri::BATLOW.name()       // "batlow"
prismatica::crameri::BATLOW.kind()       // ColormapKind::Sequential
prismatica::crameri::BATLOW.meta.perceptually_uniform  // true
prismatica::crameri::BATLOW.meta.cvd_friendly          // true

// Runtime lookup
let cm = prismatica::find_by_name("batlow");
let diverging = prismatica::filter_by_kind(ColormapKind::Diverging);
```

### Colormap selection guide

| Data type | Use | Examples |
|-----------|-----|---------|
| Sequential (temperature, elevation) | Sequential | `BATLOW`, `VIRIDIS`, `OSLO` |
| Diverging (anomalies, residuals) | Diverging | `BERLIN`, `VIK`, `SMOOTH_COOL_WARM` |
| Cyclic (phase, direction) | Cyclic | `ROMA_O`, `PHASE` |
| Categorical (labels, classes) | Discrete palette | `SET2_PALETTE`, `TABLEAU10` |

---

## chromata API reference

```rust
// Access theme
let theme: &Theme = &chromata::popular::gruvbox::DARK_HARD;
theme.bg           // Color { r, g, b } — always present
theme.fg           // Color — always present
theme.keyword      // Option<Color>
theme.accent()     // Color — first available (blue > purple > cyan > green > orange > red > fg)
theme.is_dark()    // bool
theme.colors()     // Vec<(&str, Color)> — all defined fields

// Query
chromata::find_by_name("Catppuccin Mocha")       // Option<&'static Theme>
chromata::filter_by_variant(Variant::Dark)        // Vec<&'static Theme>
chromata::collect_all_themes()                     // Vec<&'static Theme>
```

### Theme fields

Always: `name`, `author`, `variant`, `contrast`, `bg`, `fg`.
Optional UI: `cursor`, `selection`, `line_highlight`, `gutter`, `statusbar_bg`, `statusbar_fg`.
Optional syntax: `comment`, `keyword`, `string`, `function`, `variable`, `r#type`, `constant`, `operator`, `tag`.
Optional diagnostics: `error`, `warning`, `info`, `success`.
Optional accents: `red`, `orange`, `yellow`, `green`, `cyan`, `blue`, `purple`, `magenta`.

---

## Wilkinson Extended tick algorithm

```
Score = 0.2 * simplicity + 0.25 * coverage + 0.5 * density + 0.05 * legibility
Q = [1, 5, 2, 2.5, 4, 3]
Step = j * q * 10^z

simplicity = 1 - (i-1)/(|Q|-1) - j + v   (v=1 if zero included)
coverage = 1 - 0.5 * ((dmax-lmax)^2 + (dmin-lmin)^2) / (0.1*(dmax-dmin))^2
density = 2 - max(rho/rho_t, rho_t/rho)
```

Paper: https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf
R reference: https://rdrr.io/cran/labeling/src/R/labeling.R

---

## SVG text positioning

```xml
<!-- Centered text -->
<text x="100" y="50" text-anchor="middle" dominant-baseline="central">Label</text>

<!-- Rotated Y-axis label -->
<text transform="translate(15, 200) rotate(-90)" text-anchor="middle" dominant-baseline="central">Y Label</text>
```

Text width estimation: digits ≈ 0.55 × font_size, average char ≈ 0.6 × font_size.

---

## Data-to-pixel conversion

```rust
fn to_px_x(val: f64, dmin: f64, dmax: f64, px_left: f64, px_right: f64) -> f64 {
    (val - dmin) / (dmax - dmin) * (px_right - px_left) + px_left
}
fn to_px_y(val: f64, dmin: f64, dmax: f64, px_top: f64, px_bottom: f64) -> f64 {
    px_bottom - (val - dmin) / (dmax - dmin) * (px_bottom - px_top)  // Y inverted
}
```

---

## Chart layout margins

```
left_margin   = pad + y_label_height + label_pad + max_ytick_width + tick_pad
bottom_margin = pad + x_label_height + label_pad + xtick_height + tick_pad
plot_width    = figure_width  - left_margin - right_margin
plot_height   = figure_height - top_margin  - bottom_margin
max_ytick_width = max(len(format(tick))) * font_size * 0.6
```

---

## Error handling pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StarsightError {
    #[error("Rendering: {0}")]  Render(String),
    #[error("Data: {0}")]       Data(String),
    #[error("I/O: {0}")]        Io(#[from] std::io::Error),
    #[error("Scale: {0}")]      Scale(String),
    #[error("Export: {0}")]     Export(String),
    #[error("Config: {0}")]     Config(String),
    #[error("Unknown: {0}")]    Unknown(String),
}
pub type Result<T> = std::result::Result<T, StarsightError>;
```

---

## plot! macro pattern

```rust
#[macro_export]
macro_rules! plot {
    ($df:expr, x = $x:expr, y = $y:expr $(, $key:ident = $val:expr)* $(,)?) => {{
        let mut cfg = $crate::DataFramePlotConfig::new($df, $x, $y);
        $( cfg = cfg.$key($val); )*
        cfg.build()
    }};
    ($x:expr, $y:expr $(,)?) => { $crate::PlotBuilder::from_arrays($x, $y).build() };
    ($data:expr $(,)?) => { $crate::PlotBuilder::from_single($data).build() };
}
```

---

## Links

| Crate | docs.rs | GitHub |
|-------|---------|--------|
| tiny-skia | https://docs.rs/tiny-skia | https://github.com/linebender/tiny-skia |
| cosmic-text | https://docs.rs/cosmic-text | https://github.com/pop-os/cosmic-text |
| svg | https://docs.rs/svg | https://github.com/bodoni/svg |
| palette | https://docs.rs/palette | https://github.com/Ogeon/palette |
| image | https://docs.rs/image | https://github.com/image-rs/image |
| thiserror | https://docs.rs/thiserror | https://github.com/dtolnay/thiserror |
| insta | https://docs.rs/insta | https://github.com/mitsuhiko/insta |
| prismatica | https://docs.rs/prismatica | https://github.com/resonant-jovian/prismatica |
| chromata | https://docs.rs/chromata | https://github.com/resonant-jovian/chromata |
| wgpu | https://docs.rs/wgpu | https://github.com/gfx-rs/wgpu |
| ratatui | https://docs.rs/ratatui | https://github.com/ratatui/ratatui |
| polars | https://docs.rs/polars | https://github.com/pola-rs/polars |
| krilla | https://docs.rs/krilla | https://github.com/LaurenzV/krilla |
| winit | https://docs.rs/winit | https://github.com/rust-windowing/winit |
| egui | https://docs.rs/egui | https://github.com/emilk/egui |

### Theory and standards

- Wilkinson Extended ticks: https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/checklist.html
- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Edition 2024: https://doc.rust-lang.org/edition-guide/rust-2024/index.html
- Kitty protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- Sixel: https://vt100.net/docs/vt3xx-gp/chapter14.html
- Crameri colormaps: https://www.fabiocrameri.ch/colourmaps/


---
---

# Part 4 — Navigate

Tree structures and maps. Come here when you need to know which file to create, which crate a type belongs in, or how the pieces connect.

---

## Crate dependency graph

```
starsight (facade — re-exports everything, the only crate users depend on)
├── starsight-layer-1  (rendering, primitives, error, backends)
├── starsight-layer-2  (scales, axes, coordinates)
│   └── starsight-layer-1
├── starsight-layer-3  (marks, stats, aesthetics)
│   ├── starsight-layer-1
│   └── starsight-layer-2
├── starsight-layer-4  (layout, faceting, legends)
│   ├── starsight-layer-1
│   ├── starsight-layer-2
│   └── starsight-layer-3
├── starsight-layer-5  (Figure, plot!(), data acceptance)
│   ├── starsight-layer-1
│   ├── starsight-layer-2
│   ├── starsight-layer-3
│   └── starsight-layer-4
├── starsight-layer-6  (interactivity, streaming)
│   ├── starsight-layer-1 through 5
├── starsight-layer-7  (animation, export)
│   ├── starsight-layer-1 through 6
└── xtask              (build automation, not published)
```

The rule: each layer can depend on any layer below it, never on a layer above. This is enforced by Cargo.toml, not convention.

---

## File tree — current state and target

Exists means the file is in the repo right now. Target means it needs to be created for 0.1.0.

```
starsight/
├── Cargo.toml                          [exists]  workspace root
├── .spec/STARSIGHT.md                  [exists]  this document
├── LICENSE                             [exists]
├── README.md                           [exists]
├── CONTRIBUTING.md                     [exists]
├── CHANGELOG.md                        [exists]
├── CODE_OF_CONDUCT.md                  [exists]
├── SECURITY.md                         [exists]
├── .clippy.toml                        [exists]
├── .rustfmt.toml                       [exists]
├── deny.toml                           [exists]
│
├── .github/
│   ├── FUNDING.yml                     [exists]
│   ├── PULL_REQUEST_TEMPLATE.md        [exists]
│   ├── ISSUE_TEMPLATE/                 [exists]  bug_report.md, feature_request.md, config.yml
│   └── workflows/
│       ├── ci.yml                      [exists]  fmt, clippy, check, test matrix, deny
│       ├── release.yml                 [exists]  publish, github-release with git-cliff
│       ├── coverage.yml                [exists]  cargo-llvm-cov, codecov
│       ├── snapshots.yml               [exists]  cargo insta test, artifact upload
│       └── gallery.yml                 [exists]  xtask gallery, artifact upload
│
├── starsight/                          FACADE CRATE
│   ├── Cargo.toml                      [exists]  depends on all layers
│   └── src/
│       ├── lib.rs                      [exists]  re-exports
│       └── prelude.rs                  [exists]  pub use of primary types
│
├── starsight-layer-1/                  RENDERING + PRIMITIVES + ERROR
│   ├── Cargo.toml                      [exists]  deps: tiny-skia, thiserror
│   └── src/
│       ├── lib.rs                      [exists]  pub mod backend, error, primitives
│       ├── error.rs                    [exists]  StarsightError enum, Result type
│       ├── primitives.rs               [exists]  Color, Point, Rect, Size + From impls
│       │                                [target] add Vec2, Transform, ColorAlpha
│       │                                [target] add all arithmetic, accessors, conversions
│       ├── scene.rs                    [target]  SceneNode enum, Scene struct
│       └── backend/
│           ├── mod.rs                  [exists]  DrawBackend trait (partial)
│           │                            [target] uncomment all methods
│           ├── skia/
│           │   ├── mod.rs              [exists]  sub-module declarations
│           │   ├── raster/mod.rs       [exists]  empty — [target] SkiaBackend struct + DrawBackend impl
│           │   ├── headless/mod.rs     [exists]  empty — headless rendering (later)
│           │   └── png/mod.rs          [exists]  empty — PNG-specific logic (later)
│           ├── svg/
│           │   └── mod.rs              [exists]  empty — [target] SvgBackend struct + DrawBackend impl
│           ├── pdf/
│           │   └── mod.rs              [exists]  empty — PDF backend (0.10.0)
│           ├── wgpu/
│           │   ├── mod.rs              [exists]  sub-module declarations
│           │   ├── native/mod.rs       [exists]  empty — native GPU window (0.6.0)
│           │   └── web/mod.rs          [exists]  empty — WASM WebGPU (0.10.0)
│           └── terminal/
│               ├── mod.rs              [exists]  sub-module declarations
│               ├── kitty/mod.rs        [exists]  empty — Kitty protocol (0.8.0)
│               ├── sixel/mod.rs        [exists]  empty — Sixel protocol (0.8.0)
│               ├── iterm2/mod.rs       [exists]  empty — iTerm2 protocol (0.8.0)
│               ├── half_block/mod.rs   [exists]  empty — half-block chars (0.8.0)
│               └── braille/mod.rs      [exists]  empty — Braille dots (0.8.0)
│
├── starsight-layer-2/                  SCALES + AXES + COORDINATES
│   ├── Cargo.toml                      [exists]  deps: starsight-layer-1
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod scale, tick, axis, coord
│       ├── scale.rs                    [target]  Scale trait, LinearScale, LogScale, etc.
│       ├── tick.rs                     [target]  Wilkinson Extended algorithm
│       ├── axis.rs                     [target]  Axis struct (scale + ticks + labels)
│       └── coord.rs                    [target]  CartesianCoord (data-to-pixel mapping)
│
├── starsight-layer-3/                  MARKS + STATS + AESTHETICS
│   ├── Cargo.toml                      [exists]  deps: layer-1, layer-2
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod mark, line, point, bar, ...
│       ├── mark.rs                     [target]  Mark trait
│       ├── line.rs                     [target]  LineMark
│       ├── point.rs                    [target]  PointMark
│       ├── bar.rs                      [target]  BarMark (0.2.0)
│       ├── area.rs                     [target]  AreaMark (0.2.0)
│       ├── aes.rs                      [target]  Aesthetic mapping types
│       ├── position.rs                 [target]  Dodge, Stack, Jitter adjustments
│       └── stat/
│           ├── mod.rs                  [target]  stat module
│           ├── bin.rs                  [target]  Histogram binning (0.2.0)
│           └── kde.rs                  [target]  Kernel density estimation (0.3.0)
│
├── starsight-layer-4/                  LAYOUT + FACETING + LEGENDS
│   ├── Cargo.toml                      [exists]  deps: layer-1, layer-2, layer-3
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod grid, facet, legend, colorbar
│       ├── grid.rs                     [target]  GridLayout (0.4.0)
│       ├── facet.rs                    [target]  FacetWrap, FacetGrid (0.4.0)
│       ├── legend.rs                   [target]  Legend (0.4.0)
│       └── colorbar.rs                [target]  Colorbar (0.4.0)
│
├── starsight-layer-5/                  HIGH-LEVEL API
│   ├── Cargo.toml                      [exists]  deps: layer-1 through layer-4
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod figure, macro, data
│       ├── figure.rs                   [target]  Figure struct + builder
│       ├── macro.rs                    [target]  plot!() macro
│       ├── auto.rs                     [target]  chart type auto-inference
│       └── data/
│           ├── mod.rs                  [target]  DataSource trait
│           ├── raw.rs                  [target]  Vec/slice acceptance
│           ├── polars.rs               [target]  DataFrame acceptance (0.3.0)
│           ├── ndarray.rs              [target]  ndarray acceptance (0.11.0)
│           └── arrow.rs               [target]  Arrow acceptance (0.11.0)
│
├── starsight-layer-6/                  INTERACTIVITY
│   ├── Cargo.toml                      [exists]  deps: layer-1 through layer-5
│   └── src/
│       ├── lib.rs                      [exists]  empty — all 0.6.0+
│       ├── hover.rs                    [target]  tooltips (0.6.0)
│       ├── zoom.rs                     [target]  box/wheel zoom (0.6.0)
│       ├── pan.rs                      [target]  drag pan (0.6.0)
│       ├── select.rs                   [target]  box/lasso selection (0.6.0)
│       └── stream.rs                   [target]  streaming data (0.6.0)
│
├── starsight-layer-7/                  ANIMATION + EXPORT
│   ├── Cargo.toml                      [exists]  deps: layer-1 through layer-6
│   └── src/
│       ├── lib.rs                      [exists]  empty — all 0.7.0+
│       ├── animation.rs               [target]  frame recording (0.10.0)
│       ├── pdf.rs                      [target]  PDF export (0.10.0)
│       ├── html.rs                     [target]  interactive HTML (0.10.0)
│       └── terminal.rs                [target]  terminal inline output (0.8.0)
│
├── examples/
│   ├── quickstart.rs                   [exists]  empty — [target] plot!(x,y).save(...)
│   ├── scatter.rs                      [exists]  empty
│   ├── statistical.rs                  [exists]  empty
│   ├── surface3d.rs                    [exists]  empty
│   ├── terminal.rs                     [exists]  empty
│   ├── interactive.rs                  [exists]  empty
│   ├── polars_integration.rs           [exists]  empty
│   ├── streaming.rs                    [exists]  empty
│   ├── faceting.rs                     [exists]  empty
│   ├── custom_theme.rs                 [exists]  empty
│   ├── recipe.rs                       [exists]  empty
│   └── gallery.rs                      [exists]  empty
│
└── xtask/
    ├── Cargo.toml                      [exists]
    └── src/main.rs                     [exists]  empty main
```

---

## What belongs where — type ownership

| Type | Lives in | Why |
|------|----------|-----|
| `Point`, `Vec2`, `Rect`, `Size` | `starsight-layer-1::primitives` | Geometry primitives are the foundation everything else builds on |
| `Color`, `ColorAlpha` | `starsight-layer-1::primitives` | Every layer needs colors; layer 1 owns conversion to backend types |
| `Transform` | `starsight-layer-1::primitives` | Wraps tiny_skia::Transform, needed by Scene and backends |
| `StarsightError`, `Result` | `starsight-layer-1::error` | Error types must be in the lowest layer so all layers can return them |
| `DrawBackend` trait | `starsight-layer-1::backend` | Trait that backends implement |
| `SkiaBackend` | `starsight-layer-1::backend::skia::raster` | CPU rendering via tiny-skia |
| `SvgBackend` | `starsight-layer-1::backend::svg` | SVG document generation |
| `Scene`, `SceneNode` | `starsight-layer-1::scene` | Scene is data that backends consume |
| `PathStyle`, `PathCommand` | `starsight-layer-1::backend` | Drawing primitives consumed by DrawBackend |
| `Scale` trait, `LinearScale` | `starsight-layer-2::scale` | Maps data values to normalized positions |
| `extended_ticks()` | `starsight-layer-2::tick` | Tick generation algorithm |
| `Axis` | `starsight-layer-2::axis` | Scale + ticks + labels bundled together |
| `CartesianCoord` | `starsight-layer-2::coord` | Data-to-pixel coordinate mapping |
| `Mark` trait | `starsight-layer-3::mark` | Interface all visual marks implement |
| `LineMark`, `PointMark`, etc. | `starsight-layer-3::line`, etc. | Concrete mark implementations |
| `GridLayout`, `FacetWrap` | `starsight-layer-4` | Multi-chart arrangement |
| `Figure` | `starsight-layer-5::figure` | The main builder users interact with |
| `plot!()` macro | `starsight-layer-5::macro` | Zero-config entry point |
| Interactivity types | `starsight-layer-6` | Hover, zoom, pan, selection |
| Export/animation types | `starsight-layer-7` | PNG/SVG/PDF/HTML/GIF/terminal output |

---

## Hard rules

1. No JavaScript runtime dependencies
2. No C/C++ system library dependencies in default feature set
3. No unsafe in layers 3-7
4. No runtime file I/O for core functionality (colormaps, themes, fonts are compile-time)
5. No println or eprintln in library code (use log crate)
6. No panics except in .show() when no display backend is available
7. No nightly-only features required
8. No async in the public API

---

## MSRV

1.85 (edition 2024). Tracks latest stable minus two.

---

## License

GPL-3.0-only.

