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

This part is designed for text-to-speech. No code blocks, no backticks, no tables.


---

## 1.1 Overview


### What starsight is

starsight is a scientific visualization library for Rust. It exists because Rust has no equivalent of Python's matplotlib. The current options are plotters (powerful but verbose and stagnating), plotly-rs and charming (which secretly bundle JavaScript engines), egui_plot (locked to the egui framework), and textplots (terminal only). Researchers working in Rust end up exporting CSV and plotting in Python. starsight fixes this.

The library provides one import, sixty chart types, and five rendering backends. A user writes "plot x y dot save chart dot png" and gets a chart. A power user writes a grammar-of-graphics figure with layered marks, custom scales, faceting, and publication-quality PDF export. Both use the same library.

starsight belongs to the resonant-jovian ecosystem. Its sister crates are prismatica, which provides 308 scientific colormaps as compile-time lookup tables, and chromata, which provides 1104 editor color themes as compile-time constants. These are not optional integrations. They are the actual color and theme systems starsight uses. When starsight needs a viridis colormap, it calls prismatica dot crameri dot BATLOW dot eval of 0.5 and gets an RGB color back. When starsight needs a dark theme background color, it reads chromata dot popular dot gruvbox dot DARK HARD dot bg and gets three bytes.


### The layer architecture

The library is organized into seven layers, each a separate crate. Layer one is the foundation. Layer seven is the roof. Each layer depends only on layers below it. This is enforced by Cargo dependencies, not by convention. starsight-layer-3 literally cannot import anything from starsight-layer-5 because it is not in its dependency list.

Layer one is the rendering abstraction. It contains geometry primitives like Point, Rect, Size, and Color. It contains the DrawBackend trait that all rendering backends implement. It contains the Scene type that accumulates drawing commands. It contains the error types. It contains the backend implementations for tiny-skia (CPU), SVG, PDF, wgpu (GPU), and terminal (Kitty, Sixel, iTerm2, half-block, Braille). Everything in starsight ultimately bottoms out at layer one.

Layer two is the scale, axis, and coordinate system. A scale maps data values to pixel positions. A linear scale maps the range zero to one hundred onto the range zero to eight hundred pixels. A log scale does the same but logarithmically. Layer two also contains the tick generation algorithm, which decides where to place axis labels. starsight uses the Wilkinson Extended algorithm, which optimizes a scoring function over simplicity, coverage, density, and legibility. No Rust crate implements this algorithm. starsight will be the first. Layer two also contains coordinate systems. Cartesian is the default. Polar wraps angles. Geographic projects latitude and longitude.

Layer three is the mark and stat system. This is the grammar of graphics layer. A mark is a visual element: a point, a line, a bar, an area, a rect, an arc. A stat is a data transform: binning, kernel density estimation, regression, boxplot summary. An aesthetic mapping connects data columns to visual properties: x position, y position, color, size, shape. Position adjustments handle overlapping marks: dodge, stack, jitter. This layer does not render anything. It describes what should be rendered.

Layer four is layout and composition. Grid layouts arrange multiple charts in rows and columns. Faceting splits data by a categorical variable and creates one chart per value. Legends map visual encodings back to data values. Colorbars show the continuous color scale. Inset axes place a small chart inside a bigger one. This layer arranges charts but does not render them.

Layer five is the high-level API. The plot macro lives here. The Figure builder lives here. Data acceptance for Polars DataFrames, ndarray arrays, and Arrow RecordBatches lives here. Auto-inference of chart types from data shape lives here. This is the layer most users interact with.

Layer six is interactivity. Hover tooltips, box zoom, wheel zoom, pan, lasso selection, linked views between multiple charts, streaming data with rolling windows. This layer requires a windowing system (winit for native, web-sys for browser) and is entirely optional.

Layer seven is animation and export. Frame recording for GIF and MP4. Transition animations between chart states. Static export to PNG, SVG, PDF. Interactive HTML export. Terminal inline output with automatic protocol detection.


### The resonant-jovian ecosystem

The resonant-jovian organization on GitHub hosts four published crates and two in development. Understanding how they connect helps you make architectural decisions in starsight.

prismatica provides colormaps. It is a compile-time dependency of starsight for color scales. When a user maps data values to colors using Scale sequential with a prismatica colormap, starsight calls colormap dot eval with a normalized value and gets back a prismatica Color. That Color has the same three-byte structure as starsight's Color, so conversion is zero-cost.

chromata provides themes. It is a compile-time dependency of starsight for the theming system. When a user applies a chromata theme to a chart, starsight reads the theme's bg, fg, and accent colors and derives a chart theme. The mapping is: theme bg becomes chart background, theme fg becomes axis and text color, theme accent colors become the data series color cycle.

caustic is a Vlasov-Poisson solver for astrophysical simulation. It is not a dependency of starsight, but it is a consumer. caustic will use starsight to visualize simulation results: phase-space density plots, potential field contours, particle distributions. This consumer relationship informs starsight's API design: the API should work well for large scientific datasets with millions of data points.

phasma is a terminal UI for caustic, built with ratatui. It will use starsight's terminal backend to render inline charts within the TUI. This consumer relationship informs the terminal backend design: the charts must render correctly within ratatui's layout system and respond to terminal resize events.

The licensing chain matters. prismatica and chromata are GPL-3.0. starsight is GPL-3.0. caustic and phasma are also GPL-3.0. This means the entire ecosystem is GPL-consistent. A user who depends on starsight already accepts the GPL, so depending on prismatica and chromata creates no additional licensing constraint.


---

## 1.2 Type system and primitives


### Point, Vec2, and semantic arithmetic

This is a pattern from egui and from game engine math libraries. A Point is a position in space. The pixel at x equals 100, y equals 200. A Vec2 is a displacement. Fifty pixels to the right, thirty pixels down.

They are both two floats. But the valid operations are different. Subtracting one point from another gives a displacement, a Vec2. The distance from your house to the grocery store is a displacement, not a location. Adding a displacement to a point gives a new point. Your house plus the displacement to the grocery store gives the grocery store's location. But adding two points together is meaningless. Your house plus the grocery store is not a place.

The type system enforces this. Point minus Point returns Vec2. Point plus Vec2 returns Point. Point plus Point does not compile. This catches real bugs. In chart layout code, you deal with positions (where does this axis label go) and offsets (how much margin do I add). If they are both just float tuples, nothing stops you from accidentally adding two positions together and getting garbage coordinates. With separate types, the compiler catches this.

Vec2 also supports scalar multiplication. A displacement times two is twice as far in the same direction. A position times two is nonsensical. So Vec2 implements multiplication by f32, and Point does not.


### Color, alpha, and the conversion pipeline

Colors in starsight flow through multiple representations on the way from user intent to rendered pixel. Understanding the pipeline prevents an entire category of subtle bugs where colors look slightly wrong.

A user specifies a color in one of several ways. They might use a named constant like Color RED. They might use a hex literal like Color from hex 0xFF8000. They might sample a prismatica colormap like BATLOW dot eval of 0.5. They might read a theme field like gruvbox DARK HARD dot keyword. All of these produce a starsight Color: three u8 values in sRGB space, no alpha.

When the SkiaBackend needs to draw with this color, it converts to tiny skia Color using from rgba8 with alpha 255. This is a straight-alpha f32 Color. Internally, tiny-skia premultiplies it: each RGB channel is multiplied by the alpha value. For fully opaque colors (alpha 255), premultiplication is a no-op because multiplying by 255/255 equals 1. But for semi-transparent colors (like scatter point alpha of 0.5), premultiplication means the stored RGB values are half what you specified. This matters when reading back pixel data for testing or compositing.

When encoding a pixmap to PNG, tiny-skia demultiplies the pixels back to straight alpha. The encode png method handles this automatically. If you ever need to read raw pixel data from the pixmap, remember that the bytes are premultiplied. The formula to recover straight alpha is: straight r equals premultiplied r times 255 divided by alpha, except when alpha is zero (transparent), where all channels are zero.

For the SVG backend, none of this matters. SVG uses CSS color strings like fill equals hash ff8000. The SVG backend converts starsight Color to a hex string and writes it directly. No alpha premultiplication, no pixel format concerns.


### The Scene graph

Scene is a struct that holds a vector of SceneNode values. A SceneNode is an enum with variants for Path, Text, Group (with children and a transform), and Clip (with a rect and a child). The Scene does not know how to render itself. It is pure data. You build a Scene by pushing nodes into it, and then you hand the Scene to a backend which reads the nodes and renders them.

This is the pattern used by Vello (flat encoding), egui (clipped shapes list), and every modern Rust graphics library. The alternative, used by Plotters, is to make charts call backend methods directly during construction. That approach tangles chart logic with rendering logic, makes testing harder (you cannot inspect the scene without rendering it), and prevents optimizations like batching or reordering draw calls.

With a data-based scene, you can serialize it for debugging, compare two scenes for equality in tests, render the same scene to multiple backends without re-running the chart logic, and build the scene on one thread while rendering it on another.


### Builder patterns

The Figure builder uses mutable reference returns: each setter takes and mut self and returns and mut Self. This lets you chain calls or use them separately. The chained style looks like figure dot title of "Chart" dot x label of "Time" dot size of 800 comma 600. The separate style looks like: let mut fig equals Figure new, then on the next line fig dot title "Chart", then fig dot size 800 600.

This pattern was chosen over consuming self (where each method takes self by value and returns Self) because consuming self is awkward with conditional configuration. With mutable references, you can write: if show legend then fig dot legend of true. With consuming self, you would have to write: let fig equals if show legend then fig dot legend of true else fig. The consuming style also prevents partially configuring a builder, storing it, and configuring more later.

The exception is the build or save method, which does consume self (or borrows immutably and clones what it needs). This prevents accidentally modifying a figure after it has been rendered.

For mark types like LineMark and PointMark, the types are plain structs with public fields. No builder needed. You construct them with struct literal syntax. This is simpler and appropriate for types with a small number of fields where most fields are always specified.


### Error handling

Error messages are documentation. When something goes wrong, the error message is the only thing the user sees. A good error message tells the user what happened, why it happened, and what to do about it.

Bad error messages: "rendering failed", "invalid input", "error", "something went wrong." These tell the user nothing. They force the user to read the source code to understand what happened.

Good error messages: "failed to create pixmap: dimensions 0 by 0 are invalid; both width and height must be at least 1", "scale domain is empty: the minimum value 5.0 equals the maximum value 5.0; provide data with at least two distinct values or set the domain manually", "cannot save to path /tmp/chart.xyz: unknown file extension .xyz; supported extensions are .png and .svg."

The pattern is: what happened (failed to create pixmap), why (dimensions 0 by 0 are invalid), and what to do (both width and height must be at least 1). Not every error message needs all three parts, but the what part is always required, and the what-to-do part should be included whenever the fix is obvious.

For starsight, the error messages live in the code that constructs StarsightError variants. Each call site should include context about the specific operation. Instead of returning StarsightError Render of "failed", return StarsightError Render of "failed to stroke path: the path has 0 commands; ensure at least one MoveTo and one LineTo are present."

In thiserror, the Display format string is the error message. Write it in lowercase without a trailing period. Include variable context using format placeholders. If an error wraps a source error, the source is available via the standard Error chain and should not be duplicated in the message.

A useful practice: write the error message before writing the error-producing code. If you cannot explain what went wrong and how to fix it, you do not yet understand the failure mode well enough to handle it.


### Thread safety

Send means a value can be transferred between threads. Sync means a value can be shared (by reference) between threads. Most Rust types are automatically Send and Sync if all their fields are Send and Sync.

For starsight, Send and Sync matter for two reasons. First, users might want to render charts on a background thread to avoid blocking the UI. Second, the wgpu backend requires Send plus Sync for GPU resources.

The tiny-skia Pixmap type is Send but not Sync (it contains mutable state). This means you can move a SkiaBackend to another thread, but you cannot share it between threads without a mutex. This is fine for starsight's architecture because the rendering pipeline is sequential: build the scene, then render it. There is no need for concurrent access to the backend.

The Scene type should be Send and Sync because it is immutable data. Once built, it can be shared between threads. This enables a pattern where the scene is built on one thread and rendered on another.

The Figure type should be Send but does not need to be Sync because it is a builder that accumulates mutable state. You build a Figure on one thread and render it on the same thread or move it to another.

Make sure all public types are Send by default. Check with a compile-time assertion: const _ colon fn of unit where Figure colon Send equals open close curly braces. This is a zero-cost way to verify Send bounds.


---

## 1.3 Rendering


### How tiny-skia works

tiny-skia is a CPU rasterizer. You create a Pixmap (a pixel buffer), you draw paths and shapes onto it, you encode it as PNG. The Pixmap stores premultiplied RGBA pixels. Every pixel is four bytes: red, green, blue, alpha, where each RGB byte has already been multiplied by the alpha value divided by 255.

To draw a line, you build a Path. You call PathBuilder new, then move to the start point, then line to the end point, then finish. The finish method returns Option of Path. It returns None if the path is empty, which happens if you called finish without adding any segments.

To actually paint the path onto the Pixmap, you need a Paint struct and a Stroke struct. The Paint holds the color (via a Shader, which defaults to solid color) and the blend mode (default SourceOver). The Stroke holds the line width, line cap (Butt, Round, or Square), line join (Miter, Round, or Bevel), and optional dash pattern.

Then you call pixmap dot stroke path, passing the path, the paint, the stroke, a Transform (use identity for no transformation), and an optional Mask (pass None for no clipping, or pass Some of a Mask to restrict drawing to a region).

The critical thing about Transform is that its rotation method takes degrees, not radians. This is unlike virtually every other math library. If you pass pi divided by two expecting a 90-degree rotation, you will get a 1.57-degree rotation instead.

For text, starsight uses cosmic-text. You create a FontSystem (which loads system fonts and takes about one second in release mode), a SwashCache (no arguments), and a Buffer (with a Metrics struct specifying font size and line height in pixels). You set the text, call shape until scroll to lay it out, then call draw with a callback that receives individual glyph rectangles. Each callback invocation gives you an x, y, width, height, and color. You paint each rectangle onto the Pixmap using fill rect.

There is a persistent myth that you need to swap the red and blue channels between cosmic-text and tiny-skia. You do not. That swap exists in the cosmic-text example code because the example renders to softbuffer, which uses a different byte order. For PNG and SVG output, pass the channels straight through.


### The DrawBackend trait

The DrawBackend trait is the contract between charts and rendering engines. Any type that implements DrawBackend can turn path commands, text, and rectangles into visible output. The tiny-skia backend implements it by rasterizing to a pixel buffer. The SVG backend implements it by building an XML document. A hypothetical cairo backend would implement it by calling cairo C functions.

The trait is object-safe, meaning you can write dyn DrawBackend and use it as a trait object. This is critical because the Figure does not know at compile time which backend it will render to. The user calls save with a file path, the Figure checks the extension, and picks the backend at runtime. If the extension is png, it creates a SkiaBackend. If the extension is svg, it creates an SvgBackend. This requires dynamic dispatch, which requires the trait to be object-safe.

A trait is object-safe if none of its methods use Self as a return type, none of its methods use generic type parameters, and the trait does not require Sized. Plotters made the mistake of adding a Sized bound to their DrawingBackend trait, which prevents dynamic dispatch entirely. Every plotters function that accepts a backend must be generic over the backend type, which infects all downstream code with generic parameters. This is why extracting a plotters chart-drawing function into a helper is famously difficult, and it is one of the most common complaints in their issue tracker.

starsight avoids this by keeping DrawBackend object-safe from day one. The render method on Scene takes a mutable reference to dyn DrawBackend. No generics, no Sized bound, no monomorphization overhead.


### Anti-aliasing

Anti-aliasing smooths the jagged edges of diagonal lines and curves by blending edge pixels with the background. Without it, a line at a slight angle shows visible staircase steps. In tiny-skia, anti-aliasing is controlled by the anti alias field on Paint, which defaults to true.

For chart rendering, anti-aliasing should be on for all geometric elements: lines, areas, bars with rounded corners, circles. It should be off for axis lines and tick marks that are exactly horizontal or exactly vertical, because anti-aliasing a perfectly aligned line makes it appear blurry (it bleeds into adjacent pixels instead of being a crisp single-pixel line). It should also be off for glyph rectangles from cosmic-text, because the text rasterizer already handles its own anti-aliasing.

The practical rule: set paint dot anti alias to true by default, set it to false when drawing horizontal or vertical lines at integer coordinates, and set it to false when compositing text glyphs.


### Clipping

When you draw a line chart, the line should not extend beyond the plot area. If a data point maps to a pixel position outside the chart rectangle (because the scale extends beyond the data range, or because of padding), the line segment to that point should be cropped at the boundary.

In tiny-skia, clipping uses a Mask. A Mask is a grayscale image the same size as the pixmap. White areas allow drawing, black areas block it. You create a Mask, fill a rectangle into it (the plot area), and then pass it to every draw call as the last parameter. Any pixels that fall outside the mask region are silently discarded.

This is much simpler than the alternative, which is computing the intersection of every line segment with the clipping rectangle (Cohen-Sutherland algorithm). For SVG output, clipping uses the clip-path element. You define a rectangle in a defs block, reference it via clip-path attribute on a group, and the browser handles the rest.

The key insight is that clipping is a backend concern, not a mark concern. The LineMark does not need to know about clipping. It produces path commands for all data points including those outside the plot area. The backend applies the mask. This keeps mark code simple and makes the clipping behavior consistent across all mark types.


### Text rendering

Text is the hardest part of chart rendering. The reason is that text involves four separate systems that must cooperate: a font database, a shaping engine, a layout engine, and a rasterizer.

The font database (managed by cosmic-text's FontSystem) knows which fonts are available on the system. When you request "14 pixel sans-serif", the database resolves this to a specific font file, like DejaVu Sans Regular at 14 pixels. On Linux, it reads the fontconfig database. On macOS, it queries Core Text. On Windows, it reads the registry. This resolution takes about one second in release mode and up to ten seconds in debug mode, which is why FontSystem must be created once and reused, not created per draw call.

The shaping engine (harfbuzz, via cosmic-text's internal harfrust port) converts a string of Unicode characters into a sequence of positioned glyphs. Shaping handles ligatures (f plus i becoming the fi ligature), kerning (adjusting the space between specific letter pairs like AV), mark attachment (combining diacritics with base characters), and complex scripts (Arabic, Devanagari, Thai). For chart labels with Latin digits and letters, shaping mostly just assigns glyph indices and advance widths. But it still must run, because even Latin text has kerning.

The layout engine arranges the shaped glyphs into lines. For chart tick labels, this is trivial because each label is a single line. For multi-line titles or wrapped annotations, the layout engine decides where to break lines. cosmic-text's Buffer manages this. You set the text, set the maximum width, call shape until scroll, and the buffer computes line breaks and glyph positions.

The rasterizer converts glyph outlines into pixel coverage values. cosmic-text uses swash for this, accessed through SwashCache. Each glyph is rasterized once at a given size and cached. The draw callback delivers rectangular regions with alpha values representing how much of each pixel is covered by the glyph outline.

For measuring text (needed for margin calculation), you iterate layout runs after shaping. Each run has a line w field (width in pixels) and line height field. The total width is the maximum line w across all runs. The total height is the last run's line top plus its line height.

The critical integration detail: cosmic-text and tiny-skia use different color types. cosmic-text Color has r, g, b, a as u8 values. tiny-skia Color has r, g, b, a as f32 values. The conversion goes through set color rgba8 on the Paint struct, which does the division by 255 internally.


### Memory allocation

Understanding allocation patterns helps you avoid unnecessary copies and heap pressure during rendering.

The Scene is allocated on the heap. The Vec of SceneNode grows as marks emit drawing commands. Each SceneNode variant contains owned data: paths own their command vectors, text nodes own their strings. This means scene construction allocates memory proportional to the complexity of the chart.

The Pixmap in the tiny-skia backend is a large contiguous allocation: width times height times four bytes (for RGBA). For an 800 by 600 chart at 300 DPI (2400 by 1800 pixels), this is about 17 megabytes. The allocation happens once when the backend is created and the memory is reused for the entire rendering pass.

The cosmic-text FontSystem allocates when loading fonts. On a typical system, loading all system fonts takes 10 to 50 megabytes of memory. This is why FontSystem must be a long-lived object: creating it for every text draw call would allocate and deallocate this memory repeatedly.

Path construction allocates for the Vec of PathCommand. For a line chart with N points, this is proportional to N. For a scatter plot with N circles, this is proportional to N times the number of segments per circle (typically 4 cubic bezier arcs, so 16 commands per circle).

The PNG encoding allocates a buffer for the compressed output. The size depends on the image content: highly compressible charts (solid backgrounds, few colors) compress well, while detailed charts with gradients compress less.

For 0.1.0, do not optimize memory allocation. Use straightforward Vec and String allocations. Profile first (with cargo-flamegraph), optimize only the hotspots. The most likely optimization targets after profiling are: reusing Path buffers between draw calls (instead of allocating new Vecs for each path), pre-allocating the Scene vector to the expected number of nodes, and caching shaped text (so the same tick label is not shaped repeatedly if it appears on multiple axes).


### DPI handling

Charts need to render at different resolutions depending on the output target. A screen display might be 96 DPI. A retina display is 192 DPI. A print PDF is 300 DPI. A poster is 600 DPI.

starsight separates logical size from physical size. The user specifies the chart size in logical pixels (800 by 600). The rendering pipeline multiplies by a scale factor to get physical pixels. A scale factor of 1.0 gives 800 by 600 physical pixels (for screen). A scale factor of 3.75 gives 3000 by 2250 physical pixels (for 300 DPI print at the same logical size).

Font sizes, line widths, and point radii are all specified in logical units and scaled by the same factor. A 12-pixel font at scale factor 1.0 is 12 physical pixels. At scale factor 3.75, it is 45 physical pixels. This ensures charts look the same at all resolutions, just sharper at higher DPI.

The tiny-skia backend creates the Pixmap at the physical size and applies a Transform that scales all drawing operations by the scale factor. This is transparent to the marks and layout system, which always work in logical coordinates.

For SVG output, DPI does not apply because SVG is resolution-independent. The viewBox is set to the logical size, and the SVG renderer handles scaling to the display resolution.


---

## 1.4 Color and themes


### Prismatica colormaps

A Colormap in prismatica is a lookup table. It stores 256 RGB triplets as a static array of u8 three-element arrays compiled into the binary. When you call eval with a float between zero and one, it scales the float to the array index, interpolates linearly between the two nearest entries, and returns a Color.

The interpolation is in sRGB space, not linear space. This matches matplotlib, ParaView, and most scientific tools. Perceptual uniformity comes from how the lookup table was constructed (by Crameri, or the CET group, or matplotlib's team), not from the interpolation method.

eval rational takes two integers, i and n, and returns the i-th of n evenly spaced samples. This is useful when you have categorical data with n categories and want n distinct colors from a sequential map.

reversed returns a ReversedColormap, which is a zero-allocation wrapper that internally calls eval with one minus t. It does not copy or reverse the lookup table.

A DiscretePalette is different from a Colormap. It stores a fixed set of distinct colors for categorical data. It has get which takes an index and wraps around if the index exceeds the palette size. It has iter which returns an iterator over all colors without allocation.


### Chromata themes

A Theme in chromata has 29 color fields plus metadata. The bg and fg fields are always present. Everything else is Option of Color because not every source theme defines every semantic role. The accent method returns the first available accent color, checking blue, then purple, then cyan, then green, then orange, then red, falling back to fg if none are defined.

The Theme struct is non-exhaustive, meaning you cannot construct it with struct literal syntax outside the crate. Use the builder: Theme builder of name, author, bg color, fg color, then chain optional setters, then call build.

starsight uses chromata for the theming system. When a user applies a theme (for example, Catppuccin Mocha or Gruvbox Dark), starsight reads the theme's background, foreground, and accent colors and derives a chart color scheme. The background becomes the chart background. The foreground becomes the axis and text color. The accent colors become the data series color cycle. The is-dark method determines whether to use light or dark grid lines and whether text should be white-on-dark or dark-on-light.

There are 1104 themes across the popular, base16, base24, vim, and emacs modules. The popular module contains widely used themes (Dracula, Gruvbox, Catppuccin, Nord, Solarized, One Dark, Tokyo Night). The base16 module contains hundreds of themes following the Base16 color scheme specification. For starsight, the popular module provides the primary theme options. The full collection is available for users who want precise matching with their editor environment. The build method auto-detects variant (dark if background luminance is 0.5 or below) and contrast level (from the WCAG contrast ratio between bg and fg).


### Palette crate integration

The palette crate provides a comprehensive color management library with support for dozens of color spaces. For starsight, the most relevant capabilities are: sRGB to linear RGB conversion (needed for correct blending), Oklab/Oklch (perceptually uniform color spaces for generating pleasant gradients), and color mixing (blending colors in perceptually uniform space).

Currently, starsight's Color type is a simple three-byte sRGB struct. The palette crate is listed in the workspace dependencies but is not used in the 0.1.0 implementation. It becomes important when starsight implements:

Color gradient fills for area charts and heatmaps, where blending should happen in linear RGB or Oklab space for perceptual uniformity. Creating custom colormaps where the user specifies control points and starsight interpolates between them. Color accessibility tools that convert a chart's color scheme to simulate how it appears to colorblind users.

The integration path is: starsight Color converts to palette's Srgb type via From, palette performs the color space operations, and the result converts back to starsight Color. All conversions go through the f32 representation. The conversion chain is: starsight Color (u8 sRGB) to palette Srgb of f32 (f32 sRGB) to palette LinSrgb of f32 (linear RGB) to palette Oklab of f32 (perceptually uniform) and back.

For 0.1.0, the Color lerp method does linear interpolation in sRGB space. This is slightly incorrect perceptually (the midpoint between two colors appears too dark in sRGB) but matches what matplotlib and most other tools do. Correct perceptual interpolation via Oklab can be added as an option in a later version without breaking the existing API.


### Default theme and sensible defaults

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


---

## 1.5 Visualization theory

The previous subparts covered what starsight is made of: types, rendering, and color. This subpart covers what it does: translating data into visual form.


### The grammar of graphics

The grammar of graphics is a theory from Leland Wilkinson's 1999 book. The core idea is that every chart is a composition of independent components: data, aesthetic mappings, geometric marks, statistical transforms, position adjustments, scales, coordinate systems, and facets. Instead of having a "scatter plot function" and a "bar chart function" and a "box plot function" as separate things, you have a small set of composable pieces that combine to produce any chart.

An aesthetic mapping connects a column of data to a visual property. The mapping x equals sepal length means the sepal length column controls horizontal position. The mapping color equals species means the species column controls the color of each point. The mapping size equals population means the population column controls the radius of each circle. Aesthetics are declarations. They do not compute anything. They say "this data dimension should drive this visual dimension."

A geometric mark is the visual shape drawn for each data point. A point mark draws a circle. A line mark connects points with a line segment. A bar mark draws a rectangle from a baseline to the data value. An area mark fills the region between a line and the baseline. Marks read the aesthetic mappings to determine their visual properties: where to draw, what color, what size.

A statistical transform preprocesses data before the mark renders. A bin transform groups continuous data into histogram buckets and counts the number of points in each bucket. A KDE transform estimates a smooth probability density curve from discrete data points. A regression transform fits a line or curve to the data. A boxplot transform computes the five-number summary (minimum, first quartile, median, third quartile, maximum) from the data. The stat runs before the mark. A histogram is not a special chart type. It is a bar mark applied to data that has been bin-transformed.

A position adjustment handles overlapping marks. If you have a bar chart with two categories at the same x position, dodge places them side by side. Stack places them on top of each other. Jitter adds random noise to prevent overplotting in scatter plots where many points share the same coordinates.

This decomposition is why starsight has a marks layer (layer three) separate from a high-level API layer (layer five). Layer three provides the composable pieces. Layer five provides convenient shortcuts that assemble the pieces for common chart types. When a user writes plot of data frame comma x equals petal length comma kind equals Histogram, layer five internally creates a bin stat transform piped into a bar mark with a linear x scale and a count y scale.


### Scales

A scale is a function that converts a data value to a visual value. The simplest is the linear scale: given a data domain of 0 to 100 and a pixel range of 0 to 800, the value 50 maps to pixel 400. The formula is: output equals (input minus domain minimum) divided by (domain maximum minus domain minimum) times the range extent plus the range minimum.

The inverse mapping (pixel back to data value) is used for interactive features: hover tooltips convert the cursor's pixel position back to a data value.

A logarithmic scale computes the logarithm of the data value before mapping to pixel space. This compresses large values and expands small values, making it possible to visualize data that spans several orders of magnitude on a single axis. Stock prices, earthquake magnitudes, and sound levels are commonly plotted on log scales.

The edge cases are: zero (the logarithm of zero is negative infinity), negative values (the logarithm of a negative number is undefined for real numbers), and values very close to zero (the logarithm produces extremely large negative numbers that dominate the axis).

starsight's LogScale should reject data ranges that include zero or negative values by returning a StarsightError::Scale error. This is different from silently clamping or filtering, which can produce misleading charts. If the user's data contains zeros, they should use a symlog scale instead.

A symmetric log (symlog) scale handles data that crosses zero. It applies a logarithm to the absolute value and preserves the sign. Near zero, it transitions to a linear region to avoid the log singularity. The transition point is controlled by a threshold parameter C: values with absolute value less than C are mapped linearly, and values with absolute value greater than C are mapped logarithmically.

The formula is: if the absolute value of x is less than or equal to C, then the result is x divided by C. If the absolute value of x is greater than C, then the result is sign of x times (1 plus the logarithm base 10 of the absolute value of x divided by C). The factor of 1 ensures continuity at the transition point.

For ticks on a log scale, the standard positions are powers of 10: 1, 10, 100, 1000, and so on. Within each decade, minor ticks can appear at 2, 3, 4, 5, 6, 7, 8, 9 times the power of 10. For a symlog scale, the tick positions mirror around zero: minus 1000, minus 100, minus 10, minus 1, 0, 1, 10, 100, 1000.

A categorical scale maps discrete labels to evenly spaced positions. The categories "apple", "banana", "cherry" might map to pixel positions 100, 300, 500. The spacing between categories is determined by the band width. A band scale adds the concept of a bar width within each category position, which is how grouped and stacked bar charts know how wide to make each bar.

A color scale maps data values to colors using a prismatica colormap. A sequential color scale maps the range 0 to 100 onto a gradient from dark blue to bright yellow. A diverging color scale maps negative values to one color, positive values to another, with a neutral center. A qualitative color scale assigns distinct colors from a discrete palette to each category.


### The Wilkinson tick algorithm

The original paper by Talbot, Lin, and Hanrahan from 2010 is titled "An Extension of Wilkinson's Algorithm for Positioning Tick Labels on Axes." It is published in the IEEE Transactions on Visualization and Computer Graphics. The paper is dense with mathematics but the core ideas are surprisingly intuitive.

The paper starts from the observation that existing tick algorithms (including the original Wilkinson 1999 algorithm and the R default algorithm) produce suboptimal tick positions in many common cases. They propose an improved algorithm that searches over a larger space of candidates and uses a more carefully designed scoring function.

The scoring function has four components. Simplicity measures how "round" the tick values are. Ticks at 0, 20, 40, 60, 80, 100 are simpler (rounder) than ticks at 3, 23, 43, 63, 83, 103. The simplicity score depends on which step base is used (1 is simpler than 5, which is simpler than 2) and whether zero is included as a tick (bonus points if yes).

Coverage measures how well the tick range covers the data range. If the data goes from 3.7 to 97.2 and the ticks go from 0 to 100, the coverage is good (the tick range includes all data). If the ticks go from 0 to 200, the coverage is poor (the extra 100 to 200 range is wasted space). Coverage penalizes both under-coverage (ticks do not extend to the data edges) and over-coverage (ticks extend far beyond the data).

Density measures how close the number of ticks is to the target count. If the user wants 5 to 7 ticks and the algorithm produces 6, the density is perfect. If it produces 12, the density is poor (too many). If it produces 3, the density is also poor (too few). The density component has the highest weight (0.5) because the number of ticks most directly affects readability.

Legibility is a catch-all for formatting concerns. The paper simplifies it to a constant in most implementations because the other three components capture the most important factors. A more sophisticated implementation might penalize tick labels that overlap, that are too long to fit in the available space, or that use scientific notation.

The search algorithm is a set of nested loops. The outer loop iterates over skip factors (j equals 1, 2, 3, and so on). The next loop iterates over the step bases (Q equals 1, 5, 2, 2.5, 4, 3). The next loop iterates over the number of ticks (k). The innermost loop iterates over the starting position. At each level, the algorithm computes an optimistic upper bound on the achievable score. If the upper bound is less than the best score found so far, it prunes the entire subtree.

The pruning makes the algorithm fast. In the paper's analysis, the average number of candidates evaluated is about 41, regardless of the data range or the target tick count. This means the algorithm runs in effectively constant time, which is fast enough for real-time interactive use.

When implementing this in Rust, the main challenges are: getting the floating point arithmetic right (accumulated rounding errors can cause off-by-one tick positions), handling the edge cases (zero-width data range, very large or very small data values), and formatting the tick labels correctly (removing trailing zeros, using appropriate precision).


### Chart types and the mark-stat system

Understanding how common chart types decompose into marks and stats is the key to implementing them without special-casing. Each chart type is a combination of one or more marks, zero or more stat transforms, and a coordinate system.

#### Point and line marks

A scatter plot is a PointMark with x and y aesthetic mappings. No stat transform. Cartesian coordinates. Optionally, a color aesthetic maps a third variable to point color, and a size aesthetic maps a fourth variable to point radius. This is the simplest chart type and the one you should implement after LineMark.

A line chart is a LineMark with x and y aesthetic mappings. No stat transform. Cartesian coordinates. Multiple lines (series) are distinguished by the color aesthetic. The x data is typically ordered (monotonically increasing), and the LineMark connects points in order. If the data is not ordered, the line may cross itself, which is usually a user error but not something the library should prevent.

#### Bar and area marks

A histogram is a BarMark with a Bin stat transform. The Bin transform takes a single series of continuous values, divides them into bins (using Sturges' rule, the Freedman-Diaconis rule, or a user-specified bin count), and produces two arrays: bin center positions and bin counts. The BarMark renders each bin as a vertical bar from the baseline to the count value. The x scale is continuous (bin positions), and the y scale is continuous (counts).

A bar chart (as opposed to a histogram) is a BarMark with categorical x data. No stat transform is needed because the data already represents categorical values and their associated bar heights. The x scale is a BandScale (categorical positions with band widths). The y scale is continuous.

A stacked bar chart is a BarMark with a Stack position adjustment. The Stack transform computes cumulative sums within each category. Each series' bar starts where the previous series' bar ended. The rendering uses multiple BarMarks, each with a different color, layered from bottom to top.

An area chart is an AreaMark with x and y aesthetic mappings. The AreaMark is a closed path: the line goes from left to right, then drops to the baseline, runs back along the baseline, and closes. The fill color is set with some transparency to see overlapping areas.

A stacked area chart is multiple AreaMarks with a Stack position adjustment. Each area's baseline is the top of the previous area. The areas are rendered from bottom to top so that earlier series are behind later ones.

#### Grid and matrix marks

A heatmap is a RectMark (or a special HeatmapMark) with x, y, and color aesthetic mappings. The x and y are typically both categorical or both discrete grid positions. The color maps a continuous value to a prismatica colormap. Each cell is a filled rectangle.

#### Statistical marks

A box plot is a BoxPlotMark with a Boxplot stat transform. The Boxplot transform computes Q1, median, Q3, whisker endpoints, and outlier positions from each group's data. The mark renders the box, whiskers, median line, and outlier dots. The x scale is categorical (one box per group) and the y scale is continuous.

A violin plot is a ViolinMark with a KDE stat transform. The KDE transform computes a density curve for each group. The mark renders the density curve mirrored around a center line. The x scale is categorical and the y scale is continuous.

#### Specialized marks

A pie chart is an ArcMark on a polar coordinate system. The data values are normalized to sum to one, and each value maps to an angular extent. The ArcMark renders each slice as a filled arc. A donut chart is the same but with a hole in the center (inner radius greater than zero).

A contour plot is a ContourMark with a Contour stat transform. The Contour transform runs the marching squares algorithm on a 2D grid and produces a set of polylines at specified threshold values. The mark renders each polyline, colored according to the threshold value via a colormap.

A surface plot is a Surface3DMark on a 3D coordinate system. The data is a 2D grid of z-values. The mark renders a mesh of colored quadrilaterals, where the color at each vertex is determined by the z-value via a colormap. The 3D coordinate system handles the perspective projection to 2D screen coordinates.

A candlestick chart is a CandlestickMark with open, high, low, and close aesthetic mappings. Each data point represents one time period. The mark renders a vertical line from low to high (the wick) and a filled rectangle from open to close (the body). The body is green if close is greater than open (price went up) and red if close is less than open (price went down).

#### Hierarchical and flow marks

A dendrogram is a tree layout with lines connecting parent and child nodes. This is a specialized chart type that requires a hierarchical data structure, not just tabular data. The layout algorithm positions nodes in layers and draws orthogonal connector lines. This is one of the more complex chart types and is planned for the later milestones.

A Sankey diagram shows flows between nodes. The nodes are positioned in columns, and curved bands connect them, with the band width proportional to the flow magnitude. The layout algorithm positions nodes to minimize crossing, which is an optimization problem.

A treemap shows hierarchical data as nested rectangles. The squarified treemap algorithm divides a rectangle into sub-rectangles proportional to the data values, optimizing for aspect ratios close to 1 (squares are easier to compare than long thin rectangles).

#### The composition principle

Each of these chart types is a composition of marks, stats, scales, and coordinates. The grammar of graphics framework means you do not need 66 separate chart type implementations. You need about 15 marks, 10 stats, 8 scales, 3 coordinate systems, and a composition system that lets the user combine them.

#### Kernel density estimation

Kernel density estimation (KDE) is a statistical technique that estimates a smooth probability density curve from discrete data points. Given a set of values (like the ages of survey respondents), KDE produces a smooth curve showing the probability density at each value.

The algorithm places a kernel function (typically a Gaussian bell curve) at each data point, then sums all the kernels. The result is a smooth curve that peaks where data points are dense and approaches zero where data points are sparse.

The bandwidth parameter controls the smoothness. A small bandwidth produces a spiky curve that closely follows the individual data points. A large bandwidth produces a smooth curve that may obscure important features of the distribution. The default bandwidth is computed using Silverman's rule of thumb: bandwidth equals 0.9 times the minimum of (standard deviation, interquartile range divided by 1.34) times n to the power of minus 0.2, where n is the number of data points.

For starsight, KDE is implemented as a stat transform in layer 3. The input is a series of values. The output is a pair of arrays: x positions (evenly spaced across the data range) and y values (the estimated density at each position). The output feeds into a LineMark or AreaMark for rendering.

The ViolinMark is a mirrored KDE: the density curve is reflected vertically around a center line, creating a shape that resembles a violin. The width of the violin at each y-position represents the density at that value. Violin plots are used for comparing distributions across categories, replacing or supplementing box plots.

#### Box plot statistics

A box plot displays the five-number summary of a dataset: the minimum, the first quartile (Q1, the 25th percentile), the median (Q2, the 50th percentile), the third quartile (Q3, the 75th percentile), and the maximum. It also identifies outliers: points that fall more than 1.5 times the interquartile range (IQR equals Q3 minus Q1) below Q1 or above Q3.

The computation is a stat transform. The input is a series of values. The output is a struct containing: the median line position, the box edges (Q1 and Q3), the whisker endpoints (the most extreme non-outlier values, not the theoretical 1.5 IQR limits), and a list of outlier positions.

The whisker computation is subtle. The whiskers do not extend to Q1 minus 1.5 IQR and Q3 plus 1.5 IQR. They extend to the most extreme data values within those limits. If all data points are within the limits, the whiskers extend to the actual minimum and maximum. If some data points are beyond the limits, those points become outliers (drawn as individual dots) and the whiskers stop at the last non-outlier value.

The box is rendered as a filled rectangle from Q1 to Q3. The median is rendered as a line across the box. The whiskers are rendered as thin lines extending from the box edges to the whisker endpoints, with a short horizontal cap at each end. Outliers are rendered as individual points.

For grouped box plots (comparing distributions across categories), the band scale positions each group's box within a category band, similar to grouped bar charts.


### Coordinate systems

Cartesian coordinates (x right, y up) are the default for most charts. But several chart types need different coordinate systems, and understanding how they work helps you implement them correctly.

Polar coordinates use an angle (theta, measured from the right horizontal axis, increasing counterclockwise) and a radius (r, measured from the center). To convert polar to Cartesian for rendering: x equals r times cosine of theta, y equals r times sine of theta. Polar coordinates are used for radar charts, wind rose charts, and polar scatter plots. The axis is circular: tick marks appear along the circumference and along radii.

Geographic coordinates use longitude (degrees east or west of the prime meridian) and latitude (degrees north or south of the equator). Rendering geographic data requires a map projection: a mathematical function that converts the curved surface of the earth onto a flat plane. The simplest projection is equirectangular (longitude maps directly to x, latitude maps directly to y), which distorts areas far from the equator. The Mercator projection preserves angles but grossly distorts areas (Greenland appears as large as Africa). The Robinson and Natural Earth projections are compromises used in most atlases.

For starsight, the geo feature flag enables geographic chart types. The proj crate provides the actual projection functions. The coordinate system in layer 2 becomes polymorphic: CartesianCoord, PolarCoord, and GeoCoord all implement a common interface that maps data coordinates to pixel positions. The mark types do not need to know which coordinate system they are in; they call data_to_pixel and get a Point back.


### Faceting and legends

Faceting takes a single dataset and creates a grid of small charts, one per value of a categorical variable. If your data has a species column with values setosa, versicolor, and virginica, facet wrap on species creates three charts side by side, each showing only the data for one species. The axes are shared so the charts are directly comparable.

Facet wrap lays out the panels in a single row that wraps to multiple rows when it exceeds the available width. You specify the number of columns. Facet grid uses two variables: one for rows and one for columns, creating a matrix of panels.

The layout system in layer four handles faceting. It divides the available space into cells, computes shared axis ranges (unless free scales are requested), and creates a CartesianCoord for each cell. Each mark renders once per cell, filtered to the subset of data belonging to that cell.

Free scales versus fixed scales is a key user choice. Fixed scales mean all panels share the same axis range, making cross-panel comparison easy. Free scales allow each panel to zoom in on its own data range, which is useful when the scales are very different across groups. Free x and free y can be set independently.


### The data-to-pixel pipeline

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


---

## 1.6 Output formats

starsight targets seven output formats. Not all are needed for 0.1.0, but the architecture must accommodate them from the start.


### SVG

When starsight generates an SVG file, that file will be rendered by a variety of SVG implementations: web browsers (Chrome, Firefox, Safari), image viewers (Eye of GNOME, Preview.app), vector editors (Inkscape, Illustrator), and programmatic rasterizers (resvg, librsvg).

Each implementation has slightly different behavior. The most common differences are in text rendering (different fonts, different kerning, different text-anchor interpretation), gradient rendering (different interpolation methods at the edges), and filter effects (different blur implementations). For chart SVGs, which use simple shapes, solid colors, and basic text, these differences are usually invisible.

The biggest practical issue is text width. When starsight creates an SVG, it positions text elements based on estimated character widths (because it cannot measure actual rendered width without a rendering engine). If the user opens the SVG in a browser that uses a different font than starsight estimated for, the text might be slightly too wide or too narrow, causing overlaps or excessive whitespace.

The mitigation for production use is to embed the font in the SVG. SVG supports the font-face element and the url() CSS function for embedding fonts. starsight can optionally embed the font data as a base64-encoded data URI within a style block. This ensures that every SVG renderer uses the same font and produces the same layout.

For resvg-based rasterization (converting SVG to PNG without a browser), starsight can use the resvg crate (behind the resvg feature flag). resvg is a Rust SVG renderer that handles most of the SVG specification. The advantage over browser rendering is determinism: resvg always produces the same output for the same input, making it suitable for snapshot testing of SVG output.


### PDF

PDF is a page description language, not a pixel format and not an XML format. It uses a stack-based drawing model similar to PostScript. You push a graphics state, set a color, draw a path, fill or stroke it, and pop the state. Text in PDF is positioned with exact glyph coordinates and references embedded font programs.

starsight uses the krilla crate for PDF export. krilla provides a high-level API for creating PDF documents with correct text handling, color management, and page geometry. The critical advantage of PDF over SVG for publication use is that PDF embeds the actual font glyphs used in the document. The recipient does not need the font installed. SVG references font names by string, and the rendering depends on whatever font the viewer's system substitutes.

PDF also supports precise color management via ICC profiles. A PDF can declare that its colors are in sRGB, Adobe RGB, or a custom profile. This matters for print workflows where color accuracy is critical. SVG has limited color management support that varies by viewer.

The PDF backend in starsight works differently from the tiny-skia and SVG backends. It does not implement DrawBackend directly (though it could). Instead, it renders the same Scene using krilla's drawing API, which outputs PDF content streams. The conversion from SceneNode to PDF operations is similar to the SVG conversion: paths become PDF path operators, text becomes PDF text operators, groups with transforms become PDF save/restore blocks.

PDF export is gated behind the pdf feature flag and is planned for version 0.10.0.


### Terminal

starsight supports rendering charts directly in the terminal. Different terminals support different graphics protocols, and they vary wildly in capability. Kitty protocol can display full color images at pixel resolution. Sixel protocol (supported by mlterm, WezTerm, and some xterm builds) can display images at a reduced color depth. iTerm2 has its own inline image protocol. Terminals without any image protocol fall back to character-based rendering: half-block characters (the upper half and lower half block Unicode characters, giving twice the vertical resolution of regular characters) or Braille dot patterns (giving eight dots per character cell for line drawing).

The detection cascade queries terminal capabilities in order: try Kitty first (by sending a query escape sequence and checking the response), then Sixel (by checking the TERM and COLORTERM environment variables and sending a device attributes query), then iTerm2 (by checking the TERM_PROGRAM variable), then fall back to half-block, and finally Braille as the lowest-fidelity option.

For the Kitty and Sixel protocols, the chart is rendered to a PNG via the tiny-skia backend at a resolution appropriate for the terminal's cell size, then encoded in the protocol's format and written to stdout as escape sequences. The terminal interprets the escape sequences and displays the image inline. For half-block and Braille, the chart is rendered at low resolution and the pixel values are mapped to Unicode characters with foreground and background colors.

The terminal backend lives in layer one because it is a rendering backend, not an export format. It implements DrawBackend just like the tiny-skia and SVG backends. From the perspective of the marks and the figure builder, terminal rendering is indistinguishable from PNG rendering. The difference is only in how the final pixels reach the user's eyes.


### GPU

The tiny-skia CPU backend rasterizes one primitive at a time, sequentially, on a single core. It processes each pixel of each path, applying paint, blend mode, and anti-aliasing. This is simple, deterministic, and correct, but it scales linearly with the number of primitives and pixels. A scatter plot with a million points takes proportionally longer than one with a thousand.

The wgpu GPU backend works fundamentally differently. Instead of rasterizing paths, it tessellates them into triangles (using lyon or a similar library), uploads the triangle vertices to GPU memory as a vertex buffer, sets up shader programs that compute color and blending on the GPU, and issues a single draw call. The GPU processes all triangles in parallel across hundreds or thousands of cores.

For charts with many primitives (large scatter plots, dense heatmaps, real-time streaming data), the GPU backend is orders of magnitude faster. For simple charts with a few dozen elements, the CPU backend is actually faster because it avoids the overhead of GPU initialization, buffer uploads, and shader compilation.

The critical design consequence is that the DrawBackend trait must accommodate both models. The CPU backend can handle draw path calls one at a time. The GPU backend needs to batch all draw calls and submit them together. The solution is the Scene graph. Instead of calling backend methods directly during chart construction, marks emit SceneNode data into a Scene. The Scene is then handed to the backend, which can process all nodes in whatever order and batching strategy it prefers. The CPU backend iterates the nodes sequentially. The GPU backend sorts them by shader, batches compatible draw calls, and submits minimal draw calls.

The wgpu backend is entirely optional, behind the gpu feature flag. It is not needed for 0.1.0 and should not distract from getting the CPU rendering pipeline correct first. But the Scene-based architecture exists specifically to make adding the GPU backend later a matter of implementing a new backend, not restructuring the entire library.


### Animation

Animated charts (version 0.10.0) produce GIF or MP4 files showing a chart changing over time. The simplest animation is a line chart that draws from left to right. The most complex is a scatter plot where points appear, move, change color, and disappear over a time range.

The animation system works by rendering the chart at multiple time steps and encoding the frames. Each frame is a full chart render to a tiny-skia Pixmap. The frames are then encoded to GIF using the image crate's GIF encoder, or to MP4 using an external encoder like ffmpeg (invoked as a subprocess).

The animation API uses a builder pattern: Figure animate with a frame count, a frame rate, and a callback that receives the current time and modifies the chart state. The callback might update the data visible range (to animate a sliding window), modify the color scale (to animate a heatmap), or adjust the camera position (to rotate a 3D chart).

For GIF encoding, the main challenge is palette quantization. GIF supports only 256 colors per frame. The image crate handles this by finding the best 256-color palette for each frame using a median-cut algorithm. For charts with many colors (like heatmaps with continuous color scales), the quantization can produce visible banding. The mitigation is to use dithering, which distributes the quantization error across neighboring pixels.

For MP4 encoding, starsight writes raw RGBA frames to a pipe connected to ffmpeg's stdin. This requires ffmpeg to be installed on the system, which is a system dependency that violates the "no C dependencies" rule. Therefore, MP4export might use a pure-Rust encoder like rav1e (for AV1) or x264 bindings (if GPL-compatible alternatives exist). This is a version 0.10.0 concern and does not affect the current architecture.


### WASM

When starsight compiles to WebAssembly for browser deployment, it uses wasm-bindgen to bridge between Rust and JavaScript. wasm-bindgen generates JavaScript glue code that converts between Rust types and JavaScript types. web-sys provides bindings to browser Web APIs like the Canvas API and WebGPU.

The WASM target is feature-gated behind the web flag. When enabled, the wgpu backend uses WebGPU (the browser's native GPU API) instead of Vulkan or Metal. The rendering code is the same; only the GPU backend initialization differs.

For the WASM target, starsight needs to handle several differences from native: there is no filesystem (save functions write to an in-memory buffer and trigger a browser download), there is no windowing system (the chart renders into an HTML Canvas element), and font loading works differently (system fonts are not available; fonts must be bundled or loaded from URLs).

This is all planned for version 0.10.0 and should not distract from the native rendering pipeline. But the architecture accommodates it: the DrawBackend trait is generic enough that a WebGPU backend can implement it without changes to the marks, scales, or figure layers.


---

## 1.7 Data integration

A visualization library is only as useful as the data it can consume. starsight accepts raw arrays, Polars DataFrames, Arrow RecordBatches, and ndarray arrays.


### Polars and Arrow

Polars is a DataFrame library for Rust. A DataFrame is a table of named columns, where each column is a Series of typed values. When starsight accepts a DataFrame, it needs to extract columns by name and convert them to the internal data representation.

The plot macro has a special form for DataFrames: plot of ampersand df comma x equals "column name" comma y equals "column name". The string literals are column names. At runtime, the macro generates code that calls df dot column of "column name" and extracts the values as a slice of f64. If the column does not exist, or if it contains non-numeric data, the operation returns a StarsightError Data with a message explaining which column was missing or had the wrong type.

The DataFrame integration lives in layer five behind the polars feature flag. Layer five depends on polars only when that feature is enabled. Layers one through four have no knowledge of DataFrames. The data acceptance module in layer five converts DataFrame columns to plain Vec of f64 before passing them to marks in layer three.

This design means adding support for a new data source (like an Arrow RecordBatch or a CSV reader) is purely a layer five concern. You write a new data acceptance module that converts the external format to Vec of f64, and every mark and scale in the lower layers works automatically.


### Edge case data

NaN (Not a Number) is a floating point value that represents undefined or missing data. It propagates through arithmetic: any operation involving NaN produces NaN. It also has the unusual property that NaN does not equal itself: the expression NaN equals NaN is false.

In starsight, NaN can appear in input data (sensor readings with gaps, database columns with null values converted to NaN), in computed values (log of a negative number, division by zero), and in intermediate results (scale mapping of a value outside the domain).

The design principle is: NaN should never produce a panic, a garbled chart, or an infinite loop. It should produce a visible gap in the chart and optionally a warning.

For scales, NaN input should produce NaN output. The map method should check for NaN and return NaN without performing the division. This prevents NaN from contaminating the domain computation (which uses min and max, and both should skip NaN values).

For marks, NaN in the data should produce a gap. The LineMark skips NaN values and starts a new path segment at the next non-NaN value. The PointMark skips NaN values and does not draw a point. The BarMark skips NaN values and leaves a gap in the bar sequence. This behavior should be documented prominently because it differs from some tools (matplotlib raises an error, plotly draws to zero).

For the tick algorithm, NaN in the data range should cause the algorithm to return a sensible default (like ticks at 0 and 1) rather than entering an infinite loop. The Wilkinson algorithm's loop termination depends on comparing scores, and NaN comparisons always return false, which can cause the loop to run indefinitely if not guarded.

For color interpolation, NaN in the lerp parameter should return a default color (typically the first color) rather than producing a NaN color (which would have NaN in its RGB channels, which makes no sense for u8 values).

Test NaN handling explicitly in every component. A property test that feeds NaN to every public function and asserts that no panic occurs is one of the highest-value tests in the suite.


### Streaming and scale

Scientific datasets can have millions or tens of millions of data points. A line chart with ten million points generates ten million PathCommand values, each producing a line segment. Drawing ten million line segments to a 800-by-600 pixmap is wasteful because most segments are shorter than a pixel and are invisible.

The solution is data decimation: reducing the number of points to a visually lossless representation. The simplest algorithm is min-max decimation: divide the data into buckets (one per pixel column), and for each bucket, keep only the minimum and maximum values. This preserves the visual envelope of the data (peaks and valleys) while reducing the point count to at most twice the pixel width.

A more sophisticated algorithm is the Largest Triangle Three Buckets (LTTB) algorithm, which selects representative points that preserve the visual shape of the line. LTTB produces better results than min-max for sparse data but is slightly slower.

Data decimation should happen transparently: the user passes all their data, and starsight internally decimates before rendering. The original data is preserved for interaction (hover should show the exact value, not the decimated value). This means the decimation result is only used for generating path commands, not for data storage.

For scatter plots with millions of points, the equivalent optimization is spatial binning (hexbin or quad-tree aggregation). Instead of drawing a million overlapping circles, aggregate nearby points and draw one circle per aggregate with size proportional to the count.

These optimizations are not needed for 0.1.0 but should be planned for 0.4.0 or 0.5.0, when users start reporting slow rendering on large datasets.


---

## 1.8 Interactivity

Static charts cover most use cases. Interactive charts cover the rest: exploration, live dashboards, and presentation.


### Interactive charts

An interactive chart runs inside an event loop. On native platforms, this is a winit event loop that receives keyboard, mouse, and window events from the operating system. On the web, this is a requestAnimationFrame loop that receives events from the browser DOM.

When the user moves the mouse over the chart, the event loop receives a cursor moved event with the pixel coordinates. The chart needs to determine what the cursor is hovering over: which data point, which axis, which legend entry. This is called hit testing. Hit testing converts the pixel coordinates back to data coordinates using the inverse of the CartesianCoord mapping, then checks which data points are within a threshold distance of the cursor.

When the user scrolls the mouse wheel, the chart zooms. Zooming modifies the scale domain: scrolling in narrows the domain (zooming in), scrolling out widens it (zooming out). The zoom center is the cursor position, so the point under the cursor stays fixed while everything else scales around it. This requires converting the cursor pixel position to data coordinates, adjusting the domain bounds, and re-rendering.

When the user clicks and drags, the chart pans. Panning shifts the scale domain by the amount the cursor moved, converted from pixels to data units. Click and drag can also mean box selection or lasso selection, depending on the active tool.

All interactive state (current zoom level, pan offset, selected points, hover target) lives in layer six. The marks in layer three and the figure in layer five are stateless descriptions. The interactive layer wraps a figure and maintains the mutable state. Each frame, it applies the current interactive state to the figure's scales, re-renders the scene, and presents it to the window.


### Custom chart types

The recipe proc macro (planned for version 0.11.0) lets users define new chart types as compositions of existing marks, stats, and scales. A recipe is a function that takes data and configuration parameters and returns a Figure with the appropriate marks and scales already configured.

Without the recipe system, creating a custom chart type requires manually constructing a Figure, adding marks, configuring scales, and applying statistical transforms. This is verbose and error-prone. With the recipe system, the user annotates a function with starsight recipe, and the macro generates the boilerplate: it creates the Figure, adds the marks in the right order, configures the scales based on the data, and applies the statistical transforms.

For example, a volcano plot (used in genomics to display differentially expressed genes) combines a scatter mark with specific axis scales (log2 fold change on x, minus log10 p-value on y), threshold lines (significance cutoff as a horizontal line, fold change cutoff as two vertical lines), and color mapping (up-regulated genes in red, down-regulated in blue, non-significant in gray). Without a recipe, setting this up requires about 30 lines of code. With a recipe, it is a single function call.

The recipe system is an API convenience, not a fundamental architectural component. starsight works perfectly well without it. But it significantly reduces the barrier to creating and sharing custom chart types, which is important for adoption in specialized scientific communities.


### 3D visualization

nalgebra is a linear algebra library that provides vectors, matrices, and transforms for arbitrary dimensions. For starsight's 3D chart types (Surface3D, Scatter3D, Wireframe3D, Isosurface), nalgebra provides the camera model, projection matrices, and transform operations.

The camera model determines how 3D data points map to 2D screen coordinates. The two common projections are perspective (distant objects appear smaller, giving a sense of depth) and orthographic (all objects appear the same size regardless of distance, useful for technical drawings).

The camera state includes the position (where the camera is in 3D space), the target (what point the camera looks at), the up direction (which way is up), and the field of view (how wide the camera's view angle is). Orbit controls let the user rotate the camera around the target by dragging the mouse, zoom by scrolling, and pan by right-dragging.

For 3D marks, each data point has three coordinates (x, y, z). The mark converts these to screen coordinates by applying the model-view-projection matrix. The model matrix positions the chart in world space. The view matrix positions the camera. The projection matrix converts 3D to 2D. The result is a 2D point in normalized device coordinates, which is then mapped to pixel coordinates using the viewport transform.

nalgebra is behind the 3d feature flag and is not needed until version 0.7.0. The layer-1 architecture does not need to know about 3D: the 3D mark types in layer 3 perform the projection and emit 2D PathCommand sequences to the DrawBackend. From the backend's perspective, a 3D scatter plot is just a collection of 2D circles at projected positions.


### Gallery generation

The gallery is a collection of rendered chart images, one per chart type, that serves as both a visual reference and a test suite. The gallery is generated by the xtask crate and published as part of the documentation.

Each gallery entry is an example program that renders a specific chart type to PNG. The xtask gallery subcommand runs each example, collects the output files, generates a thumbnail for each (by rendering at a smaller size), and writes an HTML index page with all thumbnails linking to full-size images.

The gallery serves as a visual regression suite: if any chart type changes appearance, the gallery images change, and the gallery workflow in CI uploads the new images as artifacts. Reviewers can compare the old and new galleries to assess whether the visual changes are acceptable.

The gallery examples should use synthetic but realistic data. For a line chart, use a sine wave with noise. For a scatter plot, use clustered Gaussian data with clear separation. For a bar chart, use a small set of labeled values. For a heatmap, use a 2D Gaussian or a Mandelbrot set. For a 3D surface, use a mathematical surface like z equals sine of x times cosine of y. The data should be visually interesting and should exercise the chart type's key features (axis labels, legends, color scales, etc.).

The gallery HTML should be deployable to GitHub Pages. The gallery workflow generates the images and the HTML, then a deployment step copies them to the gh-pages branch. This makes the gallery available at resonant-jovian.github.io/starsight/gallery/.


---

## 1.9 Development tooling

This subpart covers every tool in the development workflow. Each tool is explained with what it does, why it matters for starsight, and how it fits into CI.


### Formatting and linting

Clippy has four lint groups: correctness (on by default, catches bugs), style (on by default, enforces conventions), complexity (on by default, simplifies code), and pedantic (off by default, enforces stricter conventions). There are also restriction (off by default, potentially controversial) and nursery (off by default, may have false positives) groups.

starsight enables pedantic at the warn level, which is more aggressive than most Rust projects. This catches things like missing documentation, overly complex boolean expressions, and unnecessary closures. It also produces many warnings that are correct but noisy: the must_use_candidate lint fires on every function that returns a value, suggesting you add the must_use attribute. For builder methods that return Self (where the return value is always used), this is just noise.

The strategy is: enable pedantic globally, then selectively allow specific lints either in the workspace configuration (for truly noisy lints) or with per-function allow attributes (for specific exceptions). Document why each allow is necessary.

The restriction group contains lints that are too opinionated for most projects but useful for specific cases. For starsight, the most useful restriction lints are: print_stdout (catches accidental println), unwrap_used (catches accidental unwrap), and dbg_macro (catches leftover debug macros).


### Testing

Testing a visualization library is different from testing a data processing library or a web framework. The output is visual, which means many bugs are invisible to standard assertions. You cannot assert that a chart "looks correct" — you can only assert that it matches a known-good reference, or that its numerical properties are correct.

The testing pyramid for starsight has four levels. At the base: unit tests for pure functions (scale mapping, tick generation, color conversion, coordinate math). These are fast, deterministic, and catch logic errors in the mathematical foundations. They do not catch rendering bugs.

The second level: snapshot tests for rendered output. These catch visual regressions: changes to how charts look. They are deterministic when using the CPU backend (tiny-skia) and should cover every chart type at a fixed size. The weakness is that they cannot distinguish between an intentional change (you improved the layout algorithm) and an unintentional change (you broke the layout algorithm). Human review is required when snapshots change.

The third level: property tests for mathematical invariants. These catch edge cases that unit tests miss: what happens when the data range is zero, when all values are NaN, when the array has one million elements, when the font size is 0.001, when the chart dimensions are 1 by 1 pixel. Property tests generate random inputs and check that invariants hold.

The fourth level: reference tests that compare output to other visualization libraries. Render the same data in starsight and matplotlib, and compare the results visually. This catches systematic errors where starsight's implementation diverges from established conventions (for example, if the Y axis is accidentally not inverted, all charts will be upside down). Reference tests are manual and run infrequently, but they validate that starsight produces charts that match user expectations formed by experience with other tools.

Do not aim for 100 percent code coverage. Some code paths (error handling for unlikely conditions, fallback rendering for missing fonts, format detection for file extensions) are difficult to test and unlikely to contain complex bugs. Aim for 80 percent coverage on library code and 100 percent coverage on the mathematical core (scales, ticks, coordinates, color conversion).

insta is a snapshot testing library. You render something to a string or bytes, call an assertion macro, and insta stores the output as a reference file. On subsequent runs, it compares against the stored reference and fails if anything changed.

For starsight, snapshot testing is the primary mechanism for catching visual regressions. You render a chart to PNG bytes using the tiny-skia backend, pass the bytes to the binary snapshot assertion macro, and insta stores the PNG file. If a code change makes the chart look different (even one pixel), the test fails.

The workflow has three commands. cargo insta test runs all tests and creates pending files (with a dot snap dot new extension) for any mismatches. cargo insta review opens an interactive terminal interface where you see the old and new snapshots side by side and can accept or reject each change. cargo insta accept bulk-accepts all pending changes without review.

In CI, you run cargo insta test with the check flag, which fails immediately on any mismatch instead of creating pending files. You also pass the unreferenced reject flag, which fails if there are orphaned snapshot files from deleted tests. These two flags together ensure that CI catches both regressions and stale snapshots.

The snapshot files live in a snapshots directory next to the test file. For binary snapshots like PNG images, the actual binary file is stored alongside a metadata dot snap file. Both must be committed to version control.

A practical detail: snapshot names should be descriptive. If your test renders a blue rect on white background, name the snapshot blue rect on white, not test1. When a snapshot fails in CI, the name tells you exactly what regressed.

For SVG output, use the string snapshot macro instead of binary. SVG is text, so insta can show a readable diff. This is often more useful for debugging than a PNG diff because you can see exactly which element changed.


### Coverage and profiling

cargo-flamegraph generates flamegraph visualizations that show where your program spends its CPU time. It works by sampling the call stack at high frequency (typically 1997 times per second) and aggregating the results into a hierarchical chart where wider boxes indicate more time.

Install it with cargo install flamegraph. On Linux, it requires the perf tool (install with sudo apt install linux-tools-generic). The critical setup step: enable debug symbols in release builds by adding debug equals true to the release profile in Cargo.toml. Without debug symbols, the flamegraph shows memory addresses instead of function names.

Create a dedicated profiling example that generates a realistic workload. For starsight, this might render a scatter plot with 100000 points to a PNG, or render a complex multi-panel chart with faceting and legends. The workload should run for at least a few seconds to collect enough samples.

Run it with cargo flamegraph minus minus example followed by the example name. The output is an SVG file that you open in a web browser. Click on boxes to zoom in on specific call stacks. Look for wide boxes near the top of the flame (these are the functions that directly consume the most CPU time) and narrow but tall stacks (these indicate deep call chains that might benefit from inlining).

For tiny-skia rendering, the typical hotspots are fill_path (filling shapes with color), stroke_path (drawing outlines), and the alpha blending pipeline. If fill_path dominates, consider reducing path complexity by pre-culling off-screen geometry. If alpha blending dominates, consider reducing the number of overlapping semi-transparent elements.

An alternative tool is samply, which opens the Firefox Profiler web UI with an interactive timeline, call tree, and flame chart. It is more powerful for exploratory profiling than static SVG flamegraphs.


### Dependencies

cargo-deny is a dependency governance tool that checks four things about your dependencies: their licenses, their security advisories, whether banned crates are present, and whether their source registries are trusted. You install it with cargo install cargo-deny and run it with cargo deny check.

For starsight, license checking is critical because the project is GPL-3.0. The GPL is a viral license: it requires that any program linking to starsight also be GPL-compatible. This means every dependency starsight pulls in must itself be GPL-compatible. Most Rust crates use MIT or Apache-2.0, both of which are compatible with GPL-3.0. But if a dependency uses a proprietary license, SSPL, or a GPL-incompatible copyleft license, you cannot use it.

The configuration lives in deny.toml at the workspace root. The licenses section has an allow list where you enumerate every acceptable SPDX license identifier. For a GPL project, this list includes MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, 0BSD, CC0-1.0, Unlicense, MPL-2.0, LGPL variants, and GPL variants. A common gotcha: nearly every Rust project depends on the unicode-ident crate (pulled in by proc-macro2), which uses the Unicode-DFS-2016 license. If you forget to add this to your allow list, every cargo deny check will fail with a cryptic license error on a crate you never directly depend on.

Another gotcha: the ring cryptography crate uses a mix of ISC, MIT, and OpenSSL licenses that cargo-deny cannot parse automatically. You need a clarify block in deny.toml that tells cargo-deny how to interpret ring's LICENSE file.

The advisory check queries the RustSec advisory database for known vulnerabilities in your dependency tree. This is the same database that cargo-audit uses. The difference is that cargo-deny combines it with the other three checks in a single tool. In CI, you should separate the advisory check from the license and ban checks using a matrix strategy. Advisory checks can fail unexpectedly when a new vulnerability is published for a transitive dependency, and you do not want that to block unrelated PRs. Set continue-on-error to true for the advisory job.

The ban check prevents specific crates from entering your dependency tree. For starsight, you might ban openssl in favor of rustls, or ban any crate that pulls in a C dependency you want to avoid.

The source check ensures all dependencies come from trusted registries. By default, only crates.io is trusted. If you ever need a git dependency, you must explicitly allow that git URL.

In CI, use the EmbarkStudios cargo-deny-action at version 2. It runs cargo deny check with the specified arguments. The recommended pattern is a matrix with two entries: one for advisories with continue-on-error true, and one for bans licenses sources without continue-on-error.


### API stability

cargo-semver-checks compares your current public API against the last published version on crates.io and reports any breaking changes. It works by analyzing rustdoc JSON output, which means it can detect over 120 categories of API breakage: removed public items, changed function signatures, removed trait implementations, visibility reductions, non-exhaustive additions to previously exhaustive types, and even Cargo.toml changes like removing feature flags.

The things it catches are exactly the things that are hardest to notice during code review. You rename a public method and every downstream user's code breaks. You add a new required trait method and every implementor breaks. You change a function parameter from u32 to u64 and — actually, cargo-semver-checks does not catch that one. It also misses behavioral changes, lifetime changes, and generic parameter changes. These limitations mean cargo-semver-checks is a safety net, not a guarantee.

Run it with cargo semver-checks check-release for a single crate, or cargo semver-checks for the workspace. In CI, use the obi1kenobi cargo-semver-checks-action. Run it on every PR, not just before publishing. The earlier you catch an accidental break, the easier it is to fix.

For starsight's pre-1.0 phase, every minor version bump (0.1.0 to 0.2.0) is implicitly a breaking change under semver. But running cargo-semver-checks anyway catches unintentional breaks within a minor version: if you are at 0.1.3 and accidentally remove a public method that existed in 0.1.0, it catches that.


### Publishing

git-cliff reads your git history, parses commit messages that follow the Conventional Commits format, and generates a changelog. If your commits look like feat colon add linear scale support and fix colon correct Y axis inversion, git-cliff groups them under Features and Bug Fixes headings.

Install with cargo install git-cliff and initialize with git cliff minus minus init, which creates a cliff.toml configuration file. The configuration defines how commit types map to changelog sections, what to include or exclude, and the output template.

For starsight, the template should produce a Keep a Changelog format: each release has a date and groups for Added, Changed, Deprecated, Removed, Fixed, and Security. The Conventional Commits mapping is: feat maps to Added, fix maps to Fixed, perf maps to Changed, refactor maps to Changed, and chore commits are skipped.

In the release workflow, git-cliff generates release notes for the GitHub Release page. The command git cliff minus minus latest minus minus strip header extracts just the changes since the last tag. Always use fetch-depth zero in the checkout step of the GitHub Action, because shallow clones omit the git history that git-cliff needs.

For workspace changelogs, you can generate per-crate changelogs using the include-path flag. This filters commits to only those touching files within a specific crate's directory. Whether you want a single monorepo changelog or per-crate changelogs is a project decision. For starsight, a single changelog is simpler because releases are coordinated across all layers.


### Debugging

cargo-expand shows the output of macro expansion. When you write the plot macro and something does not work, cargo expand tells you exactly what code the macro generated. Install with cargo install cargo-expand (requires nightly to be installed as a toolchain, but not as the default).

Run cargo expand with the minus p flag to target a specific crate and optionally a module path. For example, cargo expand minus p starsight-layer-5 followed by the macro module name shows the expanded output of that module. You can also expand a specific test file or example.

The expanded output is a debugging aid, not compilable code. Macro hygiene may produce identifiers that look odd. But it tells you whether the macro generated the correct structure, whether it captured the right expressions, and whether the type annotations are correct.

For the plot macro, the most common bug is incorrect expression capture. If the DataFrame form of the macro (with x equals and y equals literal tokens) does not match properly, cargo expand shows you which arm of the macro actually matched and what code it generated. This is much faster than trying to reason about macro matching rules in your head.


### Feature and MSRV testing

cargo-msrv finds and verifies the minimum supported Rust version for your project. The find subcommand does a binary search across Rust versions, compiling your project with each one until it finds the oldest version that succeeds. The verify subcommand checks that the project compiles with the declared rust-version in Cargo.toml.

For starsight, the MSRV is 1.85 because that is when edition 2024 became available. cargo-msrv find would discover this automatically, but since you already know it, the useful command is cargo msrv verify, which confirms that the declared version still works.

In practice, the simplest MSRV verification in CI is to just include the MSRV in the test matrix. If your CI tests against stable and 1.85, and 1.85 passes, the MSRV is verified. This avoids installing cargo-msrv as a separate tool.

The MSRV policy for starsight is to track the latest stable minus two releases, consistent with wgpu and ratatui. This means when Rust 1.90 ships, the MSRV may advance to 1.88. Each MSRV advance is a breaking change in the strictest reading of semver, so it should be noted in the changelog.


---

## 1.10 Workspace

A nine-crate workspace has configuration that single-crate projects never need. This subpart covers the Cargo-level plumbing.


### Workspace inheritance

Workspace inheritance is the mechanism that lets you define common metadata once in the root Cargo.toml and reference it from member crates. When you write version.workspace equals true in a member crate's Cargo.toml, Cargo reads the version field from the workspace.package section of the root.

The fields that can be inherited are: version, edition, description, license, authors, repository, documentation, readme, keywords, categories, publish, and rust-version. Each member crate chooses which fields to inherit and which to override. For starsight, all crates inherit version and edition (so they stay in sync), but each has its own description (because "Layer 1: Rendering abstraction" is more useful than the project-level description in a search result).

Dependencies can also be inherited. The workspace.dependencies section in the root Cargo.toml defines dependency versions once. Member crates reference them with dependency.workspace equals true. This ensures all crates use the same version of tiny-skia, thiserror, and every other shared dependency.

The key constraint: workspace dependency inheritance works for dependencies in the dependencies, dev-dependencies, and build-dependencies sections, but the member crate can override features. If the workspace declares tiny-skia at version 0.12.0, a member crate can reference it with workspace equals true and add features equals png to enable additional features for just that crate.

Lints can also be inherited. The workspace.lints section defines clippy and rustc lint levels that all crates share. Each crate opts in with lints.workspace equals true. This is how starsight enforces unsafe_code equals forbid and clippy pedantic equals warn across all nine crates.

Profile settings (release and dev) are always workspace-level. You cannot have different optimization settings for different member crates within the same profile. The opt-level of 1 in the dev profile applies to all starsight crates.


### Feature gating

Resolver version 3 (implied by edition 2024) changes how Cargo handles features in a workspace. When you build or test the workspace, Cargo creates a single dependency graph for all member crates and unifies their feature requirements. If starsight-layer-7 enables the terminal feature and starsight-layer-2 does not, the terminal feature is still enabled in the unified graph because some crate needs it.

This feature unification is a source of subtle bugs. If you test starsight-layer-2 in the workspace context, it compiles with the terminal feature enabled even though it does not depend on terminal functionality. This means a test might pass in the workspace but fail when starsight-layer-2 is compiled standalone. The mitigation is to test individual crates with cargo test minus p starsight-layer-2 during development, and to use cargo-hack with the each-feature flag in CI to verify that each feature compiles independently.

Resolver 3 adds MSRV-aware resolution on top of resolver 2's feature behavior. If a dependency's latest version requires Rust 1.90 but your declared rust-version is 1.85, Cargo will select an older version of the dependency that is compatible with 1.85. This only applies to user-initiated resolution (cargo update, adding new dependencies) and does not retroactively change existing Cargo.lock entries.


### Lints and CI

At 0.1.0, CI runs: cargo fmt check, cargo clippy, cargo test on three platforms (Linux, macOS, Windows) and two Rust versions (stable and 1.85), cargo insta test with check, cargo deny check, cargo doc with no-deps. This is what the current ci.yml already does.

At 0.3.0, add: cargo-semver-checks on PRs (once the crate is published to crates.io), coverage reporting with cargo-llvm-cov.

At 0.5.0, add: cargo-hack each-feature check to verify all feature combinations compile, WASM target check (cargo build with target wasm32-unknown-unknown and the web feature).

At 0.8.0, add: terminal rendering smoke tests (if possible in CI; terminal protocol tests may need a pseudo-terminal).

At 1.0.0, add: cargo-release dry-run in PRs to verify the release process works, full feature-powerset testing.

Keep CI fast. The total CI time should be under 15 minutes. If it exceeds that, split slow jobs (coverage, mutation testing, feature-powerset) into a separate workflow that runs on a schedule rather than on every PR.


### Publishing order

When you publish starsight to crates.io, you must publish the crates in dependency order. starsight-layer-1 first (it has no internal dependencies), then layer-2 (which depends on layer-1), then layer-3, and so on up to starsight (the facade which depends on everything).

Each crate's Cargo.toml must specify both path and version for workspace dependencies. During development, Cargo uses the path. During publication, Cargo strips the path and uses the version. If starsight-layer-2 depends on starsight-layer-1 at version 0.1.0, and you have not published 0.1.0 of layer-1 yet, publishing layer-2 will fail.

Cargo 1.90 and later support cargo publish minus minus workspace, which automates the topological sort and publishes all crates in the correct order. For earlier Cargo versions, use the cargo-release tool. The release workflow in GitHub Actions already has the correct order listed (commented out, ready to uncomment when the first publish happens).

crates.io also supports Trusted Publishing via OIDC tokens, which means the GitHub Action can publish without a stored API token. The workflow generates a short-lived token via the crates-io-auth-action. This is more secure than storing a long-lived CARGO_REGISTRY_TOKEN secret.

One important detail: each published crate must have a unique, meaningful description in its Cargo.toml. The current descriptions ("Layer 1: Rendering abstraction", "Layer 2: Scale, axis, coordinate") are fine for internal use but should be expanded before the first publish. crates.io search weights the description heavily.


### Compile times

Rust is a compiled language with a famously slow compiler. Every design decision that adds code — more generics, more trait implementations, more feature flag combinations — increases compile time. For a nine-crate workspace, compile time is already significant. Adding unnecessary abstraction makes it worse.

The specific trade-offs for starsight: generic functions are monomorphized for each concrete type, which generates more machine code and takes longer to compile. Trait objects use dynamic dispatch, which compiles faster but runs slower due to vtable indirection. For starsight, the choice depends on the layer. In layer 1 (rendering), use concrete types: the DrawBackend trait is a trait object, but the types it operates on (Point, Rect, Color, Path) are concrete. In layer 5 (user API), use generics sparingly: the plot macro generates monomorphized code for each input type, but the Figure builder uses trait objects for marks.

Proc macros (like thiserror's derive Error) add compile time because they run at compile time. Each proc macro invocation is a separate compilation step. For starsight, thiserror is the only proc macro dependency in the default feature set, and it runs only on the error enum (once). If you add more proc macros later (like serde Serialize for config structs), put them behind feature flags so they do not slow down the default build.

Conditional compilation via cfg attributes has zero compile time cost for the disabled code: the compiler does not even parse it. This is why feature flags are free for users who do not enable them. A user who only enables the default features compiles only the tiny-skia and SVG backends, not wgpu, ratatui, polars, or nalgebra.

The Swatinem rust-cache action in CI caches the target directory between runs, which amortizes the initial compile cost. A clean build of the starsight workspace takes about 60 to 90 seconds. An incremental rebuild after changing one file takes about 5 to 15 seconds. The cache reduces CI time by about 50 percent on average.

Profile-guided optimization and link-time optimization (LTO) improve runtime performance at the cost of compile time. LTO is enabled in the release profile (already configured). It combines all crates into a single compilation unit and optimizes across crate boundaries. This can reduce binary size by 10 to 30 percent and improve performance by 5 to 20 percent, but it makes release builds 2 to 5 times slower. For development, LTO is disabled (the dev profile uses opt-level 1 with no LTO).


### The xtask pattern

The xtask crate is a binary that lives in the workspace but is never published. It automates development tasks that are too complex for shell scripts but do not belong in the library code. The standard Rust convention is to run these tasks with cargo xtask followed by a subcommand.

For starsight, xtask will eventually handle: generating the gallery (running all examples and collecting their PNG output into a directory), running benchmarks (rendering a standard set of charts and measuring time and memory), checking that all example files compile, and preparing release artifacts. For now, its main dot rs is an empty function. Fill it in as the need arises.

The xtask pattern works because Cargo allows running binaries from workspace members without installing them. You add a .cargo/config.toml file at the workspace root with the alias: xtask equals run minus minus manifest-path xtask/Cargo.toml minus minus. Then cargo xtask gallery runs the xtask binary with the gallery argument.


---

## 1.11 API design

The public API is the contract with users. Changing it after 1.0 requires a major version bump. Getting it right before 1.0 is worth the effort.


### Rust API Guidelines

The Rust API Guidelines checklist has about 70 items. Here are the ones most relevant to starsight, with specific guidance on how to apply them.

Types eagerly implement common traits. For starsight: every public struct should implement Debug (for print debugging and error messages), Clone (for users who want to modify a copy of a configuration), and Display where meaningful (for colors, points, errors). Send and Sync should be implemented or verified for types that users might want to move between threads.

Conversions use the standard From, Into, TryFrom, and TryInto traits. For starsight: Color implements From for chromata Color and prismatica Color. Point implements From for two-element arrays and tuples. Rect implements TryFrom for tiny_skia Rect (TryFrom because the conversion can fail if bounds are invalid). Use Into in function signatures for ergonomic callers.

Error types implement std Error. For starsight: StarsightError implements Error through thiserror's derive macro. The Display implementation provides human-readable messages. The source method links to underlying errors.

Builder methods are named well. For starsight: methods that create a new modified copy use the with prefix (with_alpha on Color). Methods that mutate in place use the set prefix or take and mut self. Methods that convert use the to or into prefix.

Public dependencies are re-exported. If starsight's public API exposes a type from tiny-skia (it should not, but if it ever does), the type must be re-exported so users do not need to add tiny-skia as a separate dependency. This is one more reason to wrap external types in your own types: it avoids the re-export requirement entirely.

Sealed traits prevent external implementations. If the DrawBackend trait should only be implemented by starsight's own backends (not by external crates), use the sealed trait pattern: add a method that returns a private type. External crates cannot implement the private method, so they cannot implement the trait. However, for starsight, DrawBackend should probably be implementable externally (a user might want to implement a custom backend for their own use), so do not seal it.


### The orphan rule

The orphan rule says: you can only implement a trait for a type if either the trait or the type (or both) is defined in the current crate. This prevents two crates from independently implementing the same trait for the same type, which would create ambiguity.

For starsight, the orphan rule affects color conversions. You want to implement From of chromata Color for starsight Color. This is allowed because starsight Color is defined in the current crate (starsight-layer-1). You also want to implement From of starsight Color for tiny_skia Color. This is NOT allowed because neither starsight Color nor tiny_skia Color is the standard From trait, and the From trait is from the standard library, not from your crate.

The workaround is a method instead of a trait implementation: add a to_tiny_skia method on starsight Color that returns a tiny_skia Color. This is not a From implementation, so the orphan rule does not apply. The downside is that you cannot use the Into syntax or the question mark operator for conversion. But since these conversions happen inside backend code (not in user-facing API), the ergonomic cost is acceptable.

Similarly, you cannot implement the ratatui Widget trait for a type defined in starsight unless the Widget trait is in scope. Since Widget is defined in the ratatui crate and starsight's widget type is defined in starsight, this IS allowed: the type is local. But if you wanted a type from prismatica to implement a trait from chromata, neither is local, and the orphan rule blocks it. This is why wrapper types (newtypes) exist: wrap the foreign type in a local newtype and implement the foreign trait on the newtype.


### non_exhaustive

Adding non_exhaustive to a type that is already published is a breaking change. This is because downstream code that exhaustively matches on the enum or constructs the struct with literal syntax will no longer compile. Removing non_exhaustive is also a breaking change for structs (because it changes the struct's constructibility). For enums, removing non_exhaustive is technically not breaking (it only makes matches easier), but cargo-semver-checks may still flag it.

The practical rule for starsight: add non_exhaustive to every public enum and every public struct that might gain fields, before the first publish. Once it is on the type, adding new variants or fields is a non-breaking change in any future version.

The exception is types whose fields are their complete mathematical definition. Point (x, y), Vec2 (x, y), Color (r, g, b), and Size (width, height) have fields that are fundamental to what the type is. Adding a third field to Point would change it from a 2D point to something else entirely, which would be a redesign, not an incremental change. These types should not have non_exhaustive.

For configuration structs like RenderOptions or ThemeConfig, non_exhaustive is essential. You will definitely want to add fields like dpi, background_color, or font_family in future versions. With non_exhaustive, these additions are non-breaking.

For error enums like StarsightError, non_exhaustive is essential. You will discover new error conditions as you implement more backends and chart types. Adding a new variant like Gpu(String) or Font(String) should not break downstream match statements.

The tradeoff: non_exhaustive makes the API slightly less ergonomic. Users cannot construct the struct with literal syntax, so they need a constructor function. Users cannot exhaustively match, so they need a wildcard arm. But this tradeoff is overwhelmingly worthwhile for a pre-1.0 library that will evolve rapidly.


### Generic versus concrete types

A common question when designing a Rust API is whether to make a function generic. For starsight, the answer depends on the layer.

In layer 1 (rendering), use concrete types. The DrawBackend trait methods take specific types: Path, PathStyle, Color, Rect, Point. Making them generic would add complexity without benefit. There is one Path type, one Color type, one Rect type. The backend implementations need to know exactly what they are receiving.

In layer 3 (marks), use trait objects where needed. The Mark trait is object-safe and marks are stored as Box dyn Mark in the Figure. This allows different mark types (LineMark, PointMark, BarMark) to coexist in the same marks vector without generics.

In layer 5 (high-level API), use generics on entry points. The data acceptance functions should accept impl Into DataSource, which enables passing a Polars DataFrame, a pair of slices, or an ndarray without the user explicitly converting. The builder methods should accept impl Into String for labels and titles.

The general rule: concrete types at the bottom (where implementation details matter), generic types at the top (where user ergonomics matter), trait objects in the middle (where heterogeneous collections are needed).


### Prelude design

The prelude module re-exports the types that every user needs in every program. It should contain the types that appear in the most common usage pattern: use starsight prelude star, then call plot and save.

The prelude should export: Figure (the builder everyone uses), the plot macro (the one-liner everyone starts with), Color (needed to customize colors), Point (needed for manual positioning), StarsightError and Result (needed for error handling), and whatever trait is needed for save and show to work.

The prelude should not export: backend types (SkiaBackend, SvgBackend), internal types (PathCommand, PathStyle, SceneNode), mark types (LineMark, PointMark), scale types (LinearScale), or any type that is only needed for advanced compositional use. These live in the crate's module tree and users import them explicitly when needed.

The principle is: if a type appears in the getting started example, it belongs in the prelude. If it appears only in the advanced composition example, it does not. Overstuffing the prelude pollutes the user's namespace and causes name collisions. Understuffing it forces the user to write long import lists for basic operations.


### Documentation

starsight has three audiences: casual users who want to plot data quickly, power users who want full control over chart composition, and contributors who want to understand the architecture and extend the library.

The rustdoc documentation serves all three but with different entry points. Casual users start at the starsight crate root documentation, which should have a quick-start example showing the plot macro. Power users navigate to specific types like Figure, LineMark, LinearScale, and CartesianCoord, which have detailed examples showing compositional usage. Contributors read the architecture documentation in the dot spec directory and the internal module-level doc comments that explain design decisions.

The README serves casual users exclusively. It should have: one sentence describing what starsight is, a quick-start code block, a feature table, a list of supported chart types (ideally with thumbnails from the gallery), and links to the full documentation.

The CONTRIBUTING.md serves contributors exclusively. It should cover: setup instructions, coding standards, PR process, testing requirements, and architectural overview with links to the spec document.

The changelog serves all three audiences: casual users check it before upgrading (to see if anything broke), power users check it for new features, and contributors check it to understand recent development direction.

Do not duplicate information across these documents. The README links to docs.rs for API details. The docs.rs documentation links to the spec for architecture decisions. The spec links to the README for the public-facing description. Each document has a single authoritative role and delegates everything else.


---

## 1.12 Code standards

These rules apply to every line of code in the workspace. They are enforced by clippy, CI, and code review.


### Standard trait derives

Every public struct and enum should derive Debug, Clone, and PartialEq at minimum. Debug is required for readable test failure messages and for users to println their chart configurations. Clone is required because users will want to create a chart configuration, modify it slightly, and render both versions. PartialEq is required for assertions in tests.

For types that represent values (colors, points, sizes), also derive Copy, Eq, and Hash. Copy is appropriate because these types are small (under 32 bytes) and there is no ownership semantic. Eq is appropriate because bitwise equality is meaningful for u8 color channels and f32 coordinates (with the caveat that NaN does not equal itself, but we handle that separately). Hash is needed for using colors as HashMap keys when batching draw calls by color.

For types that hold heap data (Figure, LineMark with Vec data), derive Debug and Clone but not Copy. Implement PartialEq if meaningful comparison exists.

Do not derive Default on types where the default is not useful. A default Point at zero zero is sensible. A default Figure with no data and no marks is not: it produces an empty chart with no axes and no content, which is never what anyone wants. If Default does not produce something useful, force the user to call a constructor.


### No panicking in library code

The clippy configuration forbids unwrap_used and expect_used. These are panicking operations. A library should never crash the caller's program because a color string was malformed or a path was empty.

Use the question mark operator to propagate errors. Use ok_or_else to convert Option to Result. Use map_err to convert external error types to StarsightError. If an operation truly cannot fail (because you have already validated the inputs), use a comment explaining why and use the match or if-let pattern instead of unwrap.

The only permitted exception is in tests. Test code may use unwrap because a panic in a test is an expected failure mode. But even in tests, prefer the question mark operator with a test function that returns Result, because the error message from a propagated error is more informative than the generic "called unwrap on a None value" message.


### Dependency isolation

If your DrawBackend trait has a method that takes a tiny_skia Point, you have coupled your public API to tiny-skia's versioning. When tiny-skia releases a breaking change, your API breaks too, even if your code is unchanged. This forces a major version bump for something you did not control.

Wrap external types in your own types. starsight has its own Point, Rect, Color, and Transform types specifically for this reason. The DrawBackend trait takes starsight types. The backend implementation internally converts to tiny-skia types. This insulates the public API from dependency churn.

The same principle applies to error types. StarsightError variants contain Strings, not tiny_skia::png::EncodingError or cosmic_text::SomeError. When a backend encounters a dependency-specific error, it wraps it in a StarsightError with a descriptive message. The dependency error type never leaks through the public API.


### Feature flags and string params

The user who writes cargo add starsight should get a working library with CPU rendering, SVG output, and PNG export. They should not be forced to compile wgpu, polars, ratatui, nalgebra, or any other heavyweight dependency they do not need.

Every optional dependency goes behind a feature flag. The feature flag is defined in the starsight facade crate's Cargo.toml and forwarded to the appropriate layer crate. When the user enables the gpu feature, the facade crate enables the gpu feature on starsight-layer-1, which activates the wgpu dependency and compiles the wgpu backend code.

Feature flags must be additive. Enabling a feature must never remove functionality. A crate compiled with all features enabled must work exactly the same as one compiled with the default features, plus additional capabilities. This means feature flags should never be used for exclusive choices (either wgpu or tiny-skia, but not both). Both backends are always available; the user chooses at runtime which to use.


### Documentation

This is enforced by the warn missing_docs lint. Every public function, method, struct, enum, trait, type alias, and constant needs a doc comment. The doc comment should explain what the item does, not how it is implemented. It should include an example for anything in the prelude.

Doc comments are also tests. Rust compiles and runs code blocks in doc comments as part of cargo test minus minus doc. This means your examples must compile, your imports must be correct, and your error handling must work. Use the question mark operator in doc examples and end with a hidden line containing Ok of unit type so the example compiles as a function returning Result.

A common mistake is writing doc comments that say "creates a new Point" on a function called new on the Point struct. This adds no information. Instead, describe the semantics: "creates a point at the given screen coordinates, where x increases rightward and y increases downward."

For trait methods, the doc comment should describe the contract: what the implementor must guarantee, what the caller can assume. For the DrawBackend trait, each method should document whether it is safe to call concurrently, whether it may block, and what errors it may return.


### Don'ts

Do not return mutable references from builder methods if the builder will be consumed later. If the Figure builder returns and mut Self from title() but then save() takes self by value, the user has to call save on a temporary, which is syntactically awkward. Either make all methods take and mut self and have save take and self, or make all methods take self by value and have save also take self by value.

Do not use type aliases to hide complexity. If a function returns Result of Vec of Box dyn Mark plus Send plus Sync, StarsightError, do not create a type alias MarkList that hides the Box dyn part. Users need to see the boxed trait object to understand the ownership and dynamic dispatch implications. Type aliases are appropriate for Result T StarsightError (because every function in the crate uses this pattern) but not for application-specific composed types.

Do not add a method to a trait when a free function or a blanket impl would work. Every method on the DrawBackend trait requires every backend to implement it. If a method can be implemented in terms of other trait methods (like drawing a dashed rect by drawing four dashed lines), provide a default implementation so backends get it for free.


### Commits

starsight uses Conventional Commits. Every commit message starts with a type, an optional scope, and a description. The type determines how git-cliff categorizes the commit and how cargo-release determines the version bump.

The type feat indicates a new feature. It maps to a minor version bump under semver. The type fix indicates a bug fix. It maps to a patch version bump. The type perf, refactor, docs, test, and chore are informational and do not trigger version bumps. The type feat with an exclamation mark (feat bang) or a BREAKING CHANGE footer indicates a breaking change, which maps to a major version bump (or minor in pre-1.0).

The scope is the area of the codebase affected. For starsight, useful scopes are layer-1, layer-2, primitives, scale, backend, skia, svg, tick, and ci. The scope appears in parentheses after the type: feat layer-2 colon implement log scale.

The description is imperative mood, lowercase, no period. "add linear scale support" not "added linear scale support" and not "adds linear scale support." The description should complete the sentence "this commit will" followed by the description.

Bad commit messages: "fix stuff", "wip", "updates", "more changes." These tell you nothing about what changed or why. Good commit messages: "fix layer-2 colon correct Y axis inversion in CartesianCoord," "feat layer-1 colon implement SVG backend fill_rect," "test layer-1 colon add snapshot for blue rect on white."


---

## 1.13 Versioning

For a visualization library, breaking changes include both API changes and visual output changes.


### Breaking changes

During the pre-1.0 phase, the API changes frequently. New features are added, design mistakes are corrected, type signatures are improved. But users adopt the library from the first published version. Every change you make to the API requires every user to update their code.

The tension is between API quality (improving the design by changing things) and user convenience (not breaking things). The resolution is to front-load the hardest design decisions: get the primitive types right before publishing 0.1.0, get the trait interfaces right before publishing 0.2.0, and get the builder patterns right before publishing 0.3.0. Once these foundations are stable, later versions can add features (new chart types, new backends, new data sources) without changing existing interfaces.

The specific things to get right early: the fields and methods on Point, Vec2, Rect, Color (because these types appear everywhere and are copied into user code), the methods on DrawBackend (because backend implementors depend on them), the methods on Scale and Mark (because every chart type and scale type depends on them), and the signature of the plot macro (because it is the first thing in every tutorial).

The specific things that can change later without much pain: the Figure builder's method names (builders are called in one place, easy to update), the layout algorithm's behavior (affects visual output but not API), the internal module structure of each layer crate (users only see the facade re-exports), and the set of available chart types (adding new ones is never breaking).

Use the pre-1.0 period wisely. This is the time when breaking changes are socially acceptable. After 1.0, every breaking change requires a major version bump, which fractures the ecosystem. Make the hard decisions now so that 1.0 is a stable foundation for years of compatible evolution.


### Deprecation

Rust has built-in deprecation support via the deprecated attribute. When you mark a function as deprecated, any code that calls it produces a compiler warning. The warning includes the deprecation message, which should tell the user what to use instead.

For starsight, the deprecation cycle works like this. In version 0.2.0, you realize that the draw_path method on DrawBackend should take a reference to a PathStyle, not an owned PathStyle. You cannot just change the signature because that breaks all existing backend implementations. Instead: in 0.2.0, add a new method draw_path_ref that takes a reference. Mark the old draw_path as deprecated with a note saying "use draw_path_ref instead; draw_path will be removed in 0.4.0." Provide a default implementation of draw_path that calls draw_path_ref. In 0.4.0, remove draw_path.

This gives users two full releases to migrate. The deprecation warning is visible but not blocking (it is a warning, not an error, unless the user has turned warnings into errors). The migration path is clear: find all calls to draw_path, change them to draw_path_ref.

In the changelog, deprecations appear under the Deprecated heading. Removals appear under the Removed heading in the version where the deprecated item is finally removed. Each removal entry should reference the version where the item was deprecated and the replacement.


---

## 1.14 Language choices

Three high-level decisions that affect everything downstream.


### Edition 2024

Rust edition 2024 (shipped with Rust 1.85) changed several things relevant to starsight. The gen keyword is now reserved for future generators, so any identifier named gen must become r#gen. The unsafe_op_in_unsafe_fn lint is now warn by default, meaning unsafe operations inside unsafe functions need explicit unsafe blocks. RPIT (return position impl trait) lifetime capture rules changed: functions returning impl Trait now capture all in-scope lifetimes by default, which can affect public API signatures.

Resolver 3 (implied by edition 2024) adds MSRV-aware dependency resolution. If a dependency's latest version requires a newer Rust than your declared rust-version, Cargo falls back to an older compatible version. Feature unification behavior is unchanged from resolver 2.


### GPL-3.0

starsight is GPL-3.0-only. Not MIT, not Apache-2.0, not dual-licensed. This is an intentional choice. The sister crates chromata and prismatica are also GPL-3.0. The license is viral: any program that links starsight into its binary must also be distributed under GPL-3.0 or a compatible license. This means proprietary applications cannot use starsight without releasing their source code.

For the codebase, this means every dependency must be GPL-3.0 compatible. MIT, Apache-2.0, BSD, ISC, Zlib, and similar permissive licenses are all compatible. LGPL is compatible. Proprietary licenses and SSPL are not. The deny.toml file configures cargo-deny to check this: any dependency with an incompatible license will fail CI.

The practical impact during development: before adding a new dependency, check its license. All current workspace dependencies (tiny-skia is BSD-3, cosmic-text is MIT/Apache-2.0, thiserror is MIT/Apache-2.0, image is MIT/Apache-2.0, svg is MIT/Apache-2.0) are permissive and therefore GPL-compatible.


### No async

Rust's async ecosystem is powerful but adds significant complexity: every async function requires an executor runtime (tokio, async-std, smol), error types must be Send and Sync, and the colored function problem means async infects every call site above it.

starsight is a visualization library, not a network service. Its operations are CPU-bound (rasterization, layout computation, text shaping), not I/O-bound (waiting for network responses, reading files). CPU-bound work does not benefit from async. An async rasterizer is just a synchronous rasterizer with extra overhead.

The one place where async might seem natural is streaming data: receiving sensor readings from an async channel and updating a chart. starsight handles this with a push-based synchronous API instead. The user calls append from their own async context (or synchronous context, or signal handler, or whatever). The figure does not know or care whether the caller is async.

This design means starsight has zero dependency on any async runtime. It works equally well in a tokio application, a bare metal embedded system, a WASM browser environment, and a simple synchronous command-line tool. Adding a tokio dependency to a visualization library would be an architectural mistake that constrains every downstream user.


---

## 1.15 Sustainability

A multi-year open-source project needs more than code. It needs positioning, maintenance habits, and community awareness.


### Ecosystem positioning

The Rust ecosystem for data science and visualization is growing but fragmented. starsight positions itself as the comprehensive solution that bridges the gap between quick-and-dirty plotting (textplots, plotters) and full-featured interactive dashboards (plotly-rs, which bundles JavaScript).

The closest competitors are plotters (the most mature Rust plotting library, with good API documentation but limited chart types, stagnating development, and the Sized bound issue described earlier), plotly-rs (which generates Plotly.js charts and requires a JavaScript runtime or opens a browser tab), charming (which generates ECharts configurations and has the same JavaScript dependency), and egui_plot (which is excellent but locked to the egui framework).

starsight's differentiator is: no JavaScript runtime, no C dependencies in the default build, 66 chart types from a single library, both static export and interactive native windows, terminal rendering, GPU acceleration, and deep integration with the Rust data science stack (Polars, ndarray, Arrow). No existing Rust library offers all of these.

The risk is scope. Building a library this comprehensive takes years. Many Rust visualization projects have been abandoned after the initial enthusiasm. starsight mitigates this risk with a narrow initial scope (0.1.0 is just line charts and scatter plots), a sustainable development pace, and an architecture that allows incremental expansion without restructuring.

The opportunity is timing. The Rust data science ecosystem is maturing rapidly. Polars is approaching feature parity with pandas. ndarray is stable and widely used. Arrow support is standardized. The missing piece is visualization. The first Rust visualization library that reaches maturity will become the default choice for the ecosystem, just as matplotlib became the default for Python. starsight aims to be that library.


### Accessibility

Accessibility in data visualization means ensuring that charts communicate information to people with visual impairments, including color blindness, low vision, and blindness.

For color blindness (affecting about 8 percent of men and 0.5 percent of women), the primary mitigation is using color palettes that are distinguishable by people with all common forms of color vision deficiency. The most common form, deuteranopia, makes red and green appear similar. Protanopia has a similar effect but with different wavelengths. Tritanopia makes blue and yellow appear similar.

prismatica's perceptually uniform colormaps are designed with color vision deficiency in mind. The viridis colormap (and its variants inferno, magma, plasma) were specifically created to be distinguishable by people with all common forms of color blindness. starsight should default to these colormaps rather than rainbow colormaps (like jet) which are notoriously bad for accessibility.

Beyond colormaps, starsight should support redundant encoding: using both color and shape, or both color and pattern, to distinguish data series. A scatter plot where series A is blue circles and series B is orange squares is accessible to color-blind users because the shape distinction is sufficient even if the colors look similar.

For low vision, starsight should support configurable font sizes, line widths, and point sizes. The default values should be large enough to read at typical viewing distances (14 pixel font minimum for screen, 10 point minimum for print). High-contrast modes (black on white, white on black) should be available.

For blindness, the most accessible approach is to provide the underlying data table alongside the chart. This is straightforward for HTML export (include a hidden table element that screen readers can access) but not possible for static image formats. An alternative is to generate a text description of the chart: "Line chart showing temperature from January to December. The minimum is 5 degrees in January, the maximum is 32 degrees in July."

These accessibility features are not planned for 0.1.0 but should inform design decisions from the start: do not hardcode colors that are only distinguishable by people with full color vision, do not hardcode font sizes that are too small, and design the API so accessibility options can be added later without breaking changes.


### Long-term maintenance

starsight depends on about 30 external crates. Each of these crates is maintained by someone else and can release breaking changes, security fixes, or performance improvements at any time. Monitoring these changes and responding appropriately is an ongoing maintenance task.

Dependabot or Renovate (GitHub's automatic dependency update tools) can create PRs when new versions of dependencies are available. For starsight, enable Dependabot with a weekly schedule. Each Dependabot PR bumps one dependency to its latest version. The CI runs automatically on the PR, and if it passes, the update is safe to merge.

For major version bumps of important dependencies (like tiny-skia going from 0.12 to 0.13, or cosmic-text going from 0.18 to 0.19), manual review is necessary. Read the dependency's changelog to understand what changed. If the dependency's API changed, update starsight's backend code accordingly. If the dependency's output changed (for example, tiny-skia's anti-aliasing algorithm improved), re-run snapshot tests and review the visual changes.

cargo-deny's advisory check catches known security vulnerabilities in dependencies. The RustSec advisory database is updated frequently. New advisories can appear at any time, so the advisory check in CI may fail suddenly on an unrelated PR. The matrix strategy with continue-on-error on the advisory job handles this gracefully: the PR is not blocked, but the advisory failure is visible.

When an upstream dependency is abandoned (no releases for over a year, no response to issues), consider forking or finding an alternative. For tiny-skia, this is unlikely (it is actively maintained by the linebender project). For more niche dependencies, abandonment is a real risk. The architecture should not make starsight's correctness depend on any single optional dependency. Backend choices should be replaceable.

When an upstream dependency introduces a regression (a new version that breaks something), pin the dependency to the previous version in Cargo.toml until the regression is fixed upstream. Document the pin with a comment explaining the issue and linking to the upstream bug report. Remove the pin when the fix is released.


---

## 1.16 Getting started

Everything before this was context. This subpart is about action: how to sit down and start writing code.


### First coding session

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


### Managing complexity and motivation

A nine-crate workspace with a 66-chart-type goal can feel overwhelming. The file tree has hundreds of entries. The dependency graph has dozens of edges. The roadmap has hundreds of checkboxes. Managing this complexity without becoming paralyzed is a skill.

The first technique is to ignore most of the codebase most of the time. When you are implementing the LinearScale, the only files that matter are scale.rs and its tests. Close every file except the one you are working on.

The second technique is to work in vertical slices, not horizontal layers. Do not implement all of layer 1 before starting layer 2. Instead, implement the minimum of layer 1 needed for layer 2, then the minimum of layer 2 needed for layer 3, and so on up to the first working chart. This gives you a working system at every step.

The third technique is to defer decisions. If you are not sure whether a parameter should be f32 or f64, pick one and move on. You can refactor later. The cost of a wrong decision that is refactored in 0.2.0 is much less than the cost of agonizing for a week.

The fourth technique is to keep a running list of things you noticed but will not fix right now. Write them down and continue with your current task. This prevents scope creep.

The fifth technique is to celebrate milestones. When plot save produces the first PNG, that is a milestone worth celebrating.

Building something ambitious alone over years is closer to farming than to sprinting. You prepare soil for months before anything is visible. You tend things daily without drama. Some seasons produce abundantly and others feel barren despite your effort being the same. Motivation is not a fuel tank you fill up and drive until it runs out. It is more like weather. Some days you feel unstoppable and other days the sky is flat and grey and you cannot remember why you started. Both days are normal. The project survives not because you are always motivated but because you have built habits that carry you through the grey days.

The single most effective habit is shrinking your daily definition of progress. Do not sit down each day thinking you need to finish a module or close an issue. Sit down thinking you need to write one function, fix one test, or improve one doc string. When your threshold for a successful day is low, you win almost every day. Winning almost every day compounds into enormous momentum over months. A clean cargo check on a small change is a real win. Let it feel like one.

Publish early and publish often, even if the crate is not ready for production use. Put a zero dot one version on crates dot io with an honest README. Having something published changes your psychology. It is no longer a folder on your laptop. It is a real thing in the world. That shift from private to public creates a gentle healthy pressure to keep going.

Document before you implement, or at least alongside. Writing specs, architecture documents, and roadmaps is not busywork. It is motivation infrastructure. On days when you cannot write code, you can write docs. On days when you feel lost, your own specification tells you exactly what to build next. A detailed roadmap with phases and exit criteria is like leaving breadcrumbs for your future self.

Build in public when you can. Post progress updates to the Rust users forum, Mastodon, or Discord servers. Even three or four people who occasionally say nice work can carry you through a difficult month.

Take breaks deliberately, not guiltily. If you step away for two weeks, or even two months, that is fine. The crate will be there when you come back. The danger is not taking breaks. The danger is taking breaks while feeling guilty about them, because guilt poisons the well. When you rest, rest fully. Tell yourself the project is on pause and that pausing is part of the plan.

Keep a changelog or development journal. Even a simple text file where you write one line per day about what you did. When motivation dips, read the last fifty entries. You will be surprised how much you have done. Your brain discounts past effort by default. A written record corrects that distortion.

Do not compare your project's progress to teams or funded organizations. You are one person. Your timeline is your own. Comparison against teams is not just unfair, it is logically meaningless.

Do not chase feature completeness before shipping. If you have sixty chart types planned and you have built twelve well, that is a library worth publishing. The Rust ecosystem rewards crates that do a few things reliably over crates that promise everything.

Do not rewrite from scratch unless the architecture is genuinely unsalvageable. The urge to start over is one of the most common project killers. Refactor incrementally instead.

Avoid treating every design decision as permanent. Make a reasonable choice, document your reasoning, and move on. Rust's strong type system makes large refactors surprisingly tractable.

Do not ignore burnout signals. If you dread opening your editor, if you are staying up late out of obligation rather than excitement, step back. Better to pause for a month than to abandon the project entirely because you ground yourself down.

A solo multi-year project that actually ships is extraordinarily rare. Most ambitious open-source efforts are abandoned within the first year. If you make it past year one with a working crate and a clear direction, you are already in a small minority. The Rust ecosystem rewards persistence. Crates maintained steadily for years earn trust in a way that flashy newcomers cannot. Your project does not need to be finished to be valuable. Value accumulates with every release, not just at the finish line.
### The 0.1.0 MVP

The exit criteria for 0.1.0 is: plot exclamation of array 1.0, 2.0, 3.0 comma array 4.0, 5.0, 6.0 dot save "test.png" produces a correct line chart. This is not a full visualization library. It is the minimum vertical slice that proves the architecture works.

To get there, you need: the primitive types (Point, Vec2, Rect, Color, Transform), the tiny-skia backend (creating a pixmap, drawing paths, filling rects, rendering text, saving PNG), the SVG backend (at least fill_rect and save_svg), a linear scale, the Wilkinson tick algorithm, a Cartesian coordinate system, axis rendering (tick lines, tick labels, axis labels), a line mark, the Figure builder, the plot macro, and snapshot tests proving it all works.

You do not need: log scales, categorical scales, bar charts, histograms, box plots, faceting, legends, GPU rendering, terminal rendering, interactivity, streaming data, PDF export, WASM, Polars integration, ndarray, Arrow, or any of the 60 chart types beyond basic lines and points.

Resist the temptation to add features before the vertical slice is complete. A library that renders one chart type correctly and has tests is more valuable than a library that has stubs for 60 chart types and renders nothing.


### Debugging charts

When a chart renders incorrectly, the bug is at one of the pipeline boundaries. Here is how to isolate it.

First, check the data. Print the raw values. Are they what you expect? Are there NaN values or infinities? Are the x and y arrays the same length?

Second, check the scales. Print the domain min and max. Are they reasonable? Did the Wilkinson tick algorithm produce sensible tick positions? If the ticks look wrong, the scale domain is wrong, which means the data range computation is wrong.

Third, check the coordinate mapping. Pick a known data point and manually compute its expected pixel position using the formula: pixel x equals plot area left plus normalized x times plot area width, pixel y equals plot area bottom minus normalized y times plot area height. Does the actual pixel position match?

Fourth, check the path commands. Before sending them to the backend, print the PathCommand sequence. Are the move to and line to positions correct? Are there unexpected NaN values producing gaps?

Fifth, check the rendering. Render to SVG instead of PNG. Open the SVG in a browser and inspect the elements. SVG is human-readable. You can see the exact coordinates, colors, and transforms applied to each element. If the SVG looks correct but the PNG does not, the bug is in the tiny-skia backend translation.

Sixth, check clipping. Temporarily disable the mask (pass None instead of the plot area mask). If elements appear that were missing, the clipping rect is wrong, which means the margin or plot area computation is wrong.

The snapshot test approach helps here too. When you fix a visual bug, the snapshot test captures the corrected output. If the bug regresses, the snapshot comparison fails immediately.


### Performance

Performance optimization should not happen before the code works correctly. But performance-aware architecture decisions should happen from the start, because they are expensive to retrofit.

The key architecture decision for starsight's performance is the Scene graph. By accumulating SceneNode data instead of calling backend methods directly, the architecture enables batching (grouping similar draw calls), reordering (drawing all fills before all strokes to reduce state changes), and culling (skipping nodes that are entirely outside the visible area). None of these optimizations are implemented in 0.1.0, but the architecture supports them without changes to the mark or figure layers.

The second key decision is object reuse. The FontSystem and SwashCache must be created once and reused across all text rendering operations. The Pixmap should be created once per render call, not per mark. The PathBuilder should be reused (via clear and rebuild) rather than allocated fresh for each path.

The third key decision is avoiding unnecessary allocation. Use slices (and f64) instead of Vec of f64 for read-only data access. Use Cow for strings that are usually static but occasionally owned. Preallocate Vecs with with_capacity when the size is known.

After correctness is established, use criterion benchmarks to measure baseline performance and cargo-flamegraph to identify hotspots. Optimize only the hotspots. A 10x speedup on a function that takes 1 percent of the total time saves 0.9 percent. A 2x speedup on a function that takes 50 percent of the total time saves 25 percent.


### After 1.0

You now have the complete mental model for building starsight. The architecture is seven layers, each a separate crate, with strict dependency direction. The rendering pipeline goes from data to marks to scales to coordinates to path commands to backend to pixels. The color pipeline goes from user specification to sRGB Color to tiny-skia premultiplied pixels. The text pipeline goes from string to cosmic-text shaped glyphs to per-pixel callback to pixmap fill_rect. The testing strategy is snapshot tests for visual output, property tests for mathematical invariants, and unit tests for everything else.

The tools are: rustfmt for formatting, clippy for linting, cargo-deny for dependency governance, cargo-semver-checks for API compatibility, cargo-insta for snapshot testing, cargo-llvm-cov for coverage, cargo-nextest for fast test execution, cargo-hack for feature flag verification, git-cliff for changelogs, taplo for TOML formatting, criterion for benchmarks, and cargo-flamegraph for profiling.

The rules are: no unsafe in layers 3 through 7, no panics in library code, no println or eprintln, no async, no JavaScript dependencies, no C dependencies in the default feature set, no nightly-only features. Every public type derives Debug and Clone. Every public item has a doc comment. Every error is a StarsightError. Every feature-gated module is behind a cfg attribute at the module level.

Start with the vertical slice. Get plot save to produce a PNG. Everything else follows from there.


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

The primitive types are the foundation. Every other layer depends on them. Get these right and the rest of the codebase inherits their correctness.

#### Add Vec2 with semantic arithmetic

- [x] Create `Vec2` struct in `starsight-layer-1/src/primitives.rs` with `x: f32, y: f32` fields
- [x] Add derives: `Debug, Clone, Copy, PartialEq, Default`
- [x] Add constants: `ZERO`, `X`, `Y`
- [x] Add `new(x, y)` constructor
- [x] Add `length()` and `normalize()` methods

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

- [x] Implement `Point - Point = Vec2` (Sub trait)
- [x] Implement `Point + Vec2 = Point` (Add trait)
- [x] Implement `Point - Vec2 = Point` (Sub trait)
- [x] Implement `Vec2 + Vec2 = Vec2` (Add trait)
- [x] Implement `Vec2 * f32 = Vec2` (Mul trait)
- [x] Verify `Point + Point` does not compile (no Add<Point> for Point)
- [x] Verify `Point * f32` does not compile (no Mul<f32> for Point)
- [x] Write unit tests for all arithmetic operations

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

- [x] Add `From`/`Into` conversions for interop with other libraries:

    ```rust
    impl From<[f32; 2]> for Point { fn from([x, y]: [f32; 2]) -> Self { Self { x, y } } }
    impl From<(f32, f32)> for Point { fn from((x, y): (f32, f32)) -> Self { Self { x, y } } }
    impl From<Point> for [f32; 2] { fn from(p: Point) -> Self { [p.x, p.y] } }
    impl From<Point> for (f32, f32) { fn from(p: Point) -> Self { (p.x, p.y) } }
    // Same four impls for Vec2
    ```

- [x] Write tests:

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

- [x] Add convenience constructors and accessors:

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

- [x] Verify derives on Transform: `Debug, Clone, Copy, PartialEq`
- [ ] Add `Display` implementation for Transform:

    ```rust
    impl std::fmt::Display for Rect {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Rect({}, {}, {}, {})", self.left, self.top, self.right, self.bottom)
        }
    }
    ```

#### Complete the Color type

- [x] Add `ColorAlpha` and core Color methods:

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

- [x] Add `from_css_hex` and `to_css_hex`:

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

- [x] Add `luminance`, `contrast_ratio`, `lerp`:

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

- [x] Add sister crate conversions:

    ```rust
    impl From<chromata::Color> for Color {
        fn from(c: chromata::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    impl From<prismatica::Color> for Color {
        fn from(c: prismatica::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    ```

- [x] Write tests: `from_hex` roundtrip, `from_css_hex` with all formats, luminance black ≈ 0, luminance white ≈ 1, contrast black/white ≈ 21, lerp at 0.0 returns self, lerp at 1.0 returns other.

#### Add the Transform type

- [x] Create a `Transform` newtype wrapping `tiny_skia::Transform`:

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

- [x] Create `starsight-layer-1/src/backend/skia/raster/mod.rs`:

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

- [x] Implement `DrawBackend::dimensions()` for SkiaBackend
- [x] Implement `DrawBackend::save_png()` for SkiaBackend
- [x] Implement `DrawBackend::fill_rect()` for SkiaBackend
- [x] Implement `DrawBackend::draw_path()` for SkiaBackend
- [ ] Implement `DrawBackend::draw_text()` for SkiaBackend
- [ ] Implement `DrawBackend::set_clip()` for SkiaBackend
- [ ] Key methods reference:

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

- [x] Add to root `Cargo.toml`:

    ```toml
    [workspace.dependencies]
    insta = { version = "1.47.2", features = ["binary"] }
    ```

- [x] Create `starsight-layer-1/tests/snapshot_basic.rs`:

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

- [ ] Create `SvgBackend` struct in `starsight-layer-1/src/backend/svg/mod.rs`
- [ ] Add fields: `svg::Document`, `elements: Vec`, `width: u32`, `height: u32`
- [ ] Implement `new(width, height)`: create Document with viewBox attribute

- [ ] Implement `DrawBackend::fill_rect()` for SvgBackend — Rectangle element with CSS hex fill
- [ ] Implement `DrawBackend::draw_path()` for SvgBackend — convert PathCommands to SVG path data
- [ ] Implement `DrawBackend::draw_text()` for SvgBackend — Text element with positioning attributes

- [ ] Implement save_svg: call svg::save(path, &self.document) and map errors.

- [ ] Implement `save_svg()`: serialize document to file
- [ ] Implement `save_png()`: return `StarsightError::Export` (not supported by SVG backend)

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

- [ ] Create `starsight-layer-2/src/tick.rs`
- [ ] Implement `wilkinson_extended(dmin, dmax, target_count) -> Vec<f64>`
- [ ] Implement scoring: `0.2 * simplicity + 0.25 * coverage + 0.5 * density + 0.05 * legibility`
- [ ] Implement pruning optimization (skip candidates whose upper bound < best score)
- [ ] Write unit test: ticks for (0, 100, target=5)
- [ ] Write unit test: ticks for (0.0, 1.0, target=5)
- [ ] Write unit test: ticks for negative range
- [ ] Write unit test: zero-width range edge case
- [ ] Write property test: ticks are always monotonically increasing
- [ ] Write property test: tick step is always a nice number

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

- [ ] Create `LineMark` struct in `starsight-layer-3/src/line.rs` with `x: Vec<f64>`, `y: Vec<f64>`, `color`, `width` fields
- [ ] Implement `Mark` trait for `LineMark`
- [ ] Implement NaN gap handling: start a new `MoveTo` when encountering NaN (breaks the line)
- [ ] Write snapshot test: basic line chart
- [ ] Write snapshot test: line chart with NaN gaps
- [ ] Write snapshot test: multi-series line chart
- [ ] Implementation reference (NaN gap handling):

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

- [ ] Create `PointMark` struct in `starsight-layer-3/src/point.rs` with `x: Vec<f64>`, `y: Vec<f64>`, `color`, `size` fields
- [ ] Implement `Mark` trait for `PointMark`
- [ ] Optimize: batch all circles into one path for single `fill_path` call
- [ ] Write snapshot test: basic scatter plot
- [ ] Write snapshot test: scatter with varying point sizes
- [ ] Implementation reference (batched circles):

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

- [ ] Create `starsight/src/lib.rs` facade: re-export Figure, Color, Point, Result, Error
- [ ] Create `starsight/src/prelude.rs` with essential re-exports
- [ ] Wire the `plot!` macro in `starsight/src/lib.rs`
- [ ] Write integration test: `plot!([1,2,3], [4,5,6]).save("test.png")` produces valid PNG
- [ ] Verify `cargo test --workspace` passes with the full pipeline
- [ ] Facade wiring reference:

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




## 0.2.0 — Core chart types part 1

Exit criteria: bar charts, area charts, histograms, and heatmaps render correctly with snapshot tests.

### Layer 3: BarMark

- [ ] Create `BarMark` in `starsight-layer-3/src/marks/bar.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BarMark {
        x: Vec<String>,          // category labels
        y: Vec<f64>,             // bar heights
        color: Option<Color>,
        width: Option<f32>,      // bar width as fraction of band (0.0-1.0, default 0.8)
        orientation: Orientation, // Vertical | Horizontal
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub enum Orientation {
        #[default]
        Vertical,
        Horizontal,
    }

    impl BarMark {
        pub fn new(x: Vec<String>, y: Vec<f64>) -> Self {
            Self { x, y, color: None, width: None, orientation: Orientation::default() }
        }
        pub fn horizontal(mut self) -> Self { self.orientation = Orientation::Horizontal; self }
        pub fn color(mut self, color: impl Into<Color>) -> Self { self.color = Some(color.into()); self }
        pub fn width(mut self, w: f32) -> Self { self.width = Some(w); self }
    }
    ```

- [ ] Implement `Mark` for `BarMark` — vertical variant first:

    ```rust
    // For vertical bars:
    let band_width = band_scale.bandwidth();
    let bar_width = band_width * self.width.unwrap_or(0.8);
    let x_center = band_scale.map(&label);
    let x_left = x_center - bar_width / 2.0;
    let y_top = y_scale.map(value);
    let y_bottom = y_scale.map(0.0);  // bars grow from baseline
    let rect = Rect::from_ltrb(x_left, y_top, x_left + bar_width, y_bottom);
    backend.fill_rect(rect, color)?;
    ```

- [ ] Implement horizontal bar variant: swap x and y axes, bars grow from left to right
- [ ] Add grouped bars: accept a `group` field, subdivide each band into sub-bands per group, offset each sub-group's bar within the band
- [ ] Add stacked bars: accept a `stack` field, accumulate y values per category, each bar's baseline is the top of the previous bar
- [ ] Write snapshot test: single vertical bar chart
- [ ] Write snapshot test: horizontal bar chart
- [ ] Write snapshot test: grouped bar chart
- [ ] Write snapshot test: stacked bar chart

### Layer 3: AreaMark

- [ ] Create `AreaMark` in `starsight-layer-3/src/marks/area.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct AreaMark {
        x: Vec<f64>,
        y: Vec<f64>,
        baseline: AreaBaseline,
        color: Option<Color>,
        alpha: f32,  // fill opacity, default 0.4
    }

    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    #[non_exhaustive]
    pub enum AreaBaseline {
        #[default]
        Zero,                    // fill between y and y=0
        Fixed(f64),              // fill between y and a fixed value
    }
    ```

- [ ] Build closed area path: move to (x0, baseline) → line to (x0, y0) → line through all points → line to (xn, baseline) → close
- [ ] Fill the closed path with semi-transparent color (default alpha 0.4)
- [ ] Stroke the top edge only (from first to last data point) with full opacity

- [ ] Implement stacked area: accept `Vec<AreaMark>`
- [ ] Compute cumulative baselines: each area's bottom edge is the previous area's top edge
- [ ] Render stacked areas bottom-to-top so earlier series are behind later ones
- [ ] Write snapshot test: basic area chart
- [ ] Write snapshot test: stacked area chart

### Layer 3: Histogram stat transform

- [ ] Create `BinTransform` in `starsight-layer-3/src/stats/bin.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BinTransform {
        bins: BinMethod,
    }

    #[derive(Debug, Clone, Copy)]
    #[non_exhaustive]
    pub enum BinMethod {
        Count(usize),           // exact number of bins
        Width(f64),             // exact bin width
        Auto,                   // Sturges' rule: ceil(1 + log2(n))
        Fd,                     // Freedman-Diaconis: 2 * IQR * n^(-1/3)
    }

    impl BinTransform {
        pub fn compute(&self, data: &[f64]) -> Vec<Bin> {
            let n = self.resolve_count(data);
            let (min, max) = data_range(data);
            let width = (max - min) / n as f64;
            let mut bins = vec![Bin { left: 0.0, right: 0.0, count: 0 }; n];
            for i in 0..n {
                bins[i].left = min + i as f64 * width;
                bins[i].right = min + (i + 1) as f64 * width;
            }
            for &val in data {
                if val.is_nan() { continue; }
                let idx = ((val - min) / width).floor() as usize;
                let idx = idx.min(n - 1); // clamp last edge
                bins[idx].count += 1;
            }
            bins
        }
    }
    ```

- [ ] Create `HistogramMark` wrapping BinTransform + BarMark
- [ ] Accept raw `Vec<f64>`, run binning, render as vertical bars with no gap between bins
- [ ] Add optional KDE overlay: if `kde: true`, compute kernel density and overlay as a line
- [ ] Write snapshot test: basic histogram
- [ ] Write snapshot test: histogram with KDE overlay
- [ ] Write snapshot test: histogram with Freedman-Diaconis bins

### Layer 3: HeatmapMark

- [ ] Create `HeatmapMark` in `starsight-layer-3/src/marks/heatmap.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct HeatmapMark {
        data: Vec<Vec<f64>>,    // row-major 2D matrix
        colormap: ColormapRef,  // reference to a prismatica colormap
        annotate: bool,         // draw value text in each cell
    }
    ```

- [ ] Implement cell rendering: normalize each value, sample colormap, fill rect
- [ ] Implement annotation: render value as centered text in each cell when `annotate: true`
- [ ] Implement auto text color: choose black or white based on WCAG contrast ratio against cell color

- [ ] Write snapshot test: basic heatmap with sequential colormap
- [ ] Write snapshot test: annotated heatmap with value text
- [ ] Write snapshot test: heatmap with diverging colormap centered at zero

---

## 0.3.0 — Core chart types part 2

Exit criteria: statistical chart types (box, violin, pie, candlestick) render correctly.

### Layer 3: BoxPlotMark

- [ ] Create `BoxPlotMark` with five-number summary computation:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BoxPlotStats {
        pub min: f64,
        pub q1: f64,
        pub median: f64,
        pub q3: f64,
        pub max: f64,
        pub outliers: Vec<f64>,  // points beyond 1.5 * IQR from Q1/Q3
    }

    impl BoxPlotStats {
        pub fn compute(data: &mut [f64]) -> Self {
            data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let data: Vec<f64> = data.iter().filter(|v| !v.is_nan()).copied().collect();
            let n = data.len();
            let q1 = percentile(&data, 0.25);
            let median = percentile(&data, 0.50);
            let q3 = percentile(&data, 0.75);
            let iqr = q3 - q1;
            let lower_fence = q1 - 1.5 * iqr;
            let upper_fence = q3 + 1.5 * iqr;
            let min = data.iter().filter(|&&v| v >= lower_fence).copied().next().unwrap_or(q1);
            let max = data.iter().filter(|&&v| v <= upper_fence).copied().last().unwrap_or(q3);
            let outliers: Vec<f64> = data.iter().filter(|&&v| v < lower_fence || v > upper_fence).copied().collect();
            Self { min, q1, median, q3, max, outliers }
        }
    }
    ```

- [ ] Render box body: filled rect from Q1 to Q3
- [ ] Render median line: horizontal line across the box at median y
- [ ] Render whiskers: vertical lines from min to Q1 and Q3 to max, with horizontal caps at endpoints
- [ ] Render outliers: small circles at outlier positions
- [ ] Write snapshot test: basic box plot
- [ ] Write snapshot test: grouped box plot (multiple boxes per category)
- [ ] Write snapshot test: box plot with outliers visible

### Layer 3: ViolinMark

- [ ] Create `ViolinMark` in `starsight-layer-3/src/marks/violin.rs`. Depends on KDE stat transform.

    ```rust
    #[derive(Debug, Clone)]
    pub struct ViolinMark {
        groups: Vec<ViolinGroup>,
        bandwidth: Option<f64>,      // KDE bandwidth (None = auto via Silverman's rule)
        show_box: bool,              // overlay inner box plot (default true)
        show_median: bool,           // show median line (default true)
        cut: f64,                    // extend KDE beyond data range by cut*bw (default 2.0)
        scale: ViolinScale,          // area | count | width normalization
    }

    #[derive(Debug, Clone)]
    pub struct ViolinGroup {
        pub label: String,
        pub data: Vec<f64>,
    }
    ```

- [ ] Implement KDE computation: 256 evaluation points spanning data range +/- cut*bandwidth
- [ ] Build mirrored path: left side is negative density, right side is positive density
- [ ] Fill mirrored path with semi-transparent color
- [ ] Implement box overlay: narrow rect from Q1 to Q3, median line, whisker lines (when show_box is true)

- [ ] Implement split violins: when two groups share a category, render left/right halves from different groups
- [ ] Write snapshot test: basic violin plot
- [ ] Write snapshot test: grouped violins
- [ ] Write snapshot test: split violin
- [ ] Write snapshot test: violin with box overlay disabled

### Layer 3: PieMark and DonutMark

- [ ] Implement PieMark: compute start/end angles from cumulative proportions
- [ ] Implement arc path approximation using cubic beziers (subdivide arcs > PI/2)
- [ ] Implement DonutMark: PieMark with configurable inner radius
- [ ] Write snapshot test: basic pie chart
- [ ] Write snapshot test: donut chart

    ```rust
    fn arc_path(cx: f32, cy: f32, r: f32, start_rad: f32, end_rad: f32) -> Path {
        // Approximate arc with cubic bezier
        // For arcs <= PI/2, one cubic is sufficient
        // For larger arcs, subdivide into segments
        let mut pb = PathBuilder::new();
        let segments = ((end_rad - start_rad) / (std::f32::consts::FRAC_PI_2)).ceil() as usize;
        // ... build arc segments ...
        pb.finish().unwrap()
    }
    ```

### Layer 3: CandlestickMark

- [ ] Create `CandlestickMark` in `starsight-layer-3/src/marks/candlestick.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct CandlestickMark {
        data: Vec<OHLC>,
        up_color: Color,      // default green (#26a69a)
        down_color: Color,    // default red (#ef5350)
        body_width: f32,      // candle body width as fraction of available space (default 0.7)
        wick_width: f32,      // wick line width in pixels (default 1.0)
    }

    #[derive(Debug, Clone, Copy)]
    pub struct OHLC {
        pub timestamp: f64,   // x-axis position (epoch seconds or index)
        pub open: f64,
        pub high: f64,
        pub low: f64,
        pub close: f64,
    }
    ```

- [ ] Determine candle color: up_color if close >= open, down_color otherwise
- [ ] Draw body: filled rect from open to close
- [ ] Draw upper wick: vertical line from max(open, close) to high
- [ ] Draw lower wick: vertical line from min(open, close) to low
- [ ] Support both DateTimeScale and LinearScale for x-axis

- [ ] Write snapshot test: basic candlestick chart
- [ ] Write snapshot test: candlestick with custom colors

### Layer 5: Polars integration

- [ ] Create data acceptance layer in `starsight-layer-5/src/data/polars.rs` (behind `polars` feature flag):

    ```rust
    use polars::prelude::*;

    pub fn extract_f64(df: &DataFrame, col: &str) -> Result<Vec<f64>> {
        let series = df.column(col)
            .map_err(|e| StarsightError::Data(format!("Column '{}': {}", col, e)))?;
        let ca = series.f64()
            .or_else(|_| series.cast(&DataType::Float64).and_then(|s| s.f64()))
            .map_err(|e| StarsightError::Data(format!("Cannot convert '{}' to f64: {}", col, e)))?;
        Ok(ca.into_no_null_iter().collect())
    }

    pub fn extract_strings(df: &DataFrame, col: &str) -> Result<Vec<String>> {
        let series = df.column(col)?;
        let ca = series.str()?;
        Ok(ca.into_no_null_iter().map(|s| s.to_string()).collect())
    }
    ```

- [ ] Integrate DataFrame path with `plot!` macro
- [ ] Auto-detect column types: numeric columns → LineMark, categorical x → BarMark
- [ ] Support `color = "column"` for automatic grouping

- [ ] Accept eager `DataFrame` directly
- [ ] Accept `LazyFrame`: call `.collect()` before extraction, log warning about materialization

- [ ] Convert Polars null values to f64 NaN for mark rendering pipeline compatibility

- [ ] Write snapshot test: line chart from DataFrame
- [ ] Write snapshot test: scatter plot with color grouping from DataFrame

---

## 0.4.0 — Layout and composition

Exit criteria: faceted charts and multi-panel layouts render correctly.

### Layer 4: GridLayout

- [ ] Create `GridLayout` in `starsight-layer-4/src/grid.rs`:

    ```rust
    #[derive(Debug)]
    pub struct GridLayout {
        figures: Vec<Vec<Option<Figure>>>,  // row-major, None for empty cells
        row_heights: Vec<f32>,              // proportional weights (default: equal)
        col_widths: Vec<f32>,               // proportional weights (default: equal)
        gap: f32,                           // pixels between panels (default: 10.0)
        title: Option<String>,              // overall title above the grid
    }

    impl GridLayout {
        pub fn new(rows: usize, cols: usize) -> Self { /* ... */ }
        pub fn set(&mut self, row: usize, col: usize, figure: Figure) { /* ... */ }
        pub fn row_height(mut self, row: usize, weight: f32) -> Self { /* ... */ }
        pub fn col_width(mut self, col: usize, weight: f32) -> Self { /* ... */ }
        pub fn gap(mut self, pixels: f32) -> Self { /* ... */ }
    }
    ```

- [ ] Compute pixel bounds for each grid cell from proportional weights and gaps
- [ ] Render each non-None figure into its cell using clipping mask + translation transform

- [ ] Implement `GridLayout::row(figures)` convenience (1xN layout)
- [ ] Implement `GridLayout::column(figures)` convenience (Nx1 layout)
- [ ] Implement `GridLayout::from_figures(figures, ncol)` convenience (auto-wrap)

- [ ] Write snapshot test: 2x2 grid with mixed chart types
- [ ] Write snapshot test: 1x3 row layout
- [ ] Write snapshot test: grid with unequal column widths

### Layer 4: FacetWrap and FacetGrid

- [ ] Create `FacetWrap` in `starsight-layer-4/src/facet.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct FacetWrap {
        column: String,             // column name to split by
        ncol: Option<usize>,        // number of columns (None = auto sqrt)
        scales: FacetScales,        // Free | FreeX | FreeY | Fixed
        label_position: FacetLabelPosition,  // Top (default) | Bottom
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub enum FacetScales {
        #[default]
        Fixed,       // all panels share same axes
        FreeX,       // each panel has independent x scale
        FreeY,       // each panel has independent y scale
        Free,        // fully independent scales
    }
    ```

- [ ] Implement FacetWrap: split data by unique values of one categorical variable
- [ ] Create one subplot panel per unique value
- [ ] Auto-determine grid size: ncol defaults to ceil(sqrt(n_panels))
- [ ] Arrange panels left-to-right, top-to-bottom
- [ ] Render title strip above each panel showing the facet value

- [ ] Implement FacetGrid: split by two variables (row and column)
- [ ] Create subplot matrix: row var determines rows, col var determines columns

    ```rust
    #[derive(Debug, Clone)]
    pub struct FacetGrid {
        row: String,                // row facet variable
        col: String,                // column facet variable
        scales: FacetScales,
        margin_titles: bool,        // row titles on right margin, col titles on top
    }
    ```

- [ ] Implement shared axes: single x/y scale across all panels when FacetScales::Fixed
- [ ] Render x tick labels only on bottom row panels
- [ ] Render y tick labels only on left column panels

- [ ] Write snapshot test: facet wrap with 6 panels (2x3 grid)
- [ ] Write snapshot test: facet grid with 2 rows x 3 columns
- [ ] Write snapshot test: facet wrap with free y scales

### Layer 4: Legend

- [ ] Create `Legend` in `starsight-layer-4/src/legend.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct Legend {
        entries: Vec<LegendEntry>,
        position: LegendPosition,
        title: Option<String>,
    }

    #[derive(Debug, Clone)]
    pub struct LegendEntry {
        pub label: String,
        pub swatch: LegendSwatch,   // colored square, line segment, or circle
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub enum LegendPosition {
        #[default]
        TopRight,
        TopLeft,
        BottomRight,
        BottomLeft,
        OutsideRight,    // outside the plot area
        OutsideBottom,
    }
    ```

- [ ] Measure widest label text to compute legend bounding box
- [ ] Render semi-transparent background rect for inside positions (readability)
- [ ] Adjust plot area margins for outside positions
- [ ] Render each entry: colored swatch (rect/line/circle per mark type) + label text

- [ ] Implement auto-generation: Figure creates Legend from marks' color/label mappings when more than one series is present
- [ ] Allow user to override position, hide legend, or customize entries
- [ ] Write snapshot test: legend inside top-right
- [ ] Write snapshot test: legend outside right
- [ ] Write snapshot test: legend with mixed line and scatter entries

### Layer 4: Colorbar

- [ ] Create `Colorbar` in `starsight-layer-4/src/colorbar.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct Colorbar {
        colormap: ColormapRef,       // prismatica colormap reference
        domain: (f64, f64),          // data range
        label: Option<String>,
        orientation: Orientation,     // Vertical (default) | Horizontal
        width: f32,                  // bar width in pixels (default 20)
        tick_count: usize,           // target number of ticks (default 5)
    }
    ```

- [ ] Render gradient strip: N thin rectangles (N >= 256), each sampled from colormap
- [ ] Place tick marks and labels along the strip using Wilkinson ticks
- [ ] Position colorbar to the right (vertical) or below (horizontal) the plot area
- [ ] Adjust figure margins to accommodate the colorbar

- [ ] Write snapshot test: vertical colorbar with sequential colormap
- [ ] Write snapshot test: horizontal colorbar with diverging colormap

---

## 0.5.0 — Scale infrastructure

Exit criteria: all scale types render correctly. Tick locator and formatter traits enable custom tick logic. Log and datetime scales produce correct tick positions.

### LogScale and SymlogScale

- [ ] Implement `LogScale::map()`: apply log10, then linear interpolation
- [ ] Validate domain is strictly positive, return `StarsightError::Scale` otherwise
- [ ] Implement `LogLocator`: ticks at powers of 10 and sub-ticks at 2 and 5

    ```rust
    impl Scale for LogScale {
        fn map(&self, val: f64) -> f64 {
            let log_val = val.log10();
            let log_min = self.domain.0.log10();
            let log_max = self.domain.1.log10();
            (log_val - log_min) / (log_max - log_min) * (self.range.1 - self.range.0) + self.range.0
        }
    }
    ```

- [ ] `SymlogScale` uses `sign(x) * log10(1 + |x| / threshold)` with a configurable linear threshold near zero.

### DateTimeScale

- [ ] Create `DateTimeScale` in `starsight-layer-2/src/scale/datetime.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct DateTimeScale {
        domain: (f64, f64),          // epoch seconds
        range: (f64, f64),           // pixel range
        granularity: Option<TimeGranularity>,  // None = auto-detect
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum TimeGranularity {
        Second, Minute, Hour, Day, Week, Month, Quarter, Year,
    }
    ```

- [ ] Implement auto-detection: compute span in seconds, select granularity by threshold

- [ ] Implement tick generation per granularity (Year → Jan 1, Month → 1st, Day → midnight, etc.)
- [ ] Skip ticks that fall outside the domain

- [ ] Implement label formatting per granularity (Year → "2024", Month → "Jan", Day → "Jan 15", etc.)

- [ ] Write snapshot test: 10-year time series (year ticks)
- [ ] Write snapshot test: 6-month time series (month ticks)
- [ ] Write snapshot test: 48-hour time series (hour ticks)

### BandScale and CategoricalScale

- [ ] Create `BandScale` in `starsight-layer-2/src/scale/band.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BandScale {
        domain: Vec<String>,          // category labels in order
        range: (f64, f64),            // pixel range
        inner_padding: f64,           // gap between bands as fraction of step (default 0.1)
        outer_padding: f64,           // gap at edges as fraction of step (default 0.05)
    }

    impl BandScale {
        pub fn bandwidth(&self) -> f64 {
            let n = self.domain.len() as f64;
            let total = self.range.1 - self.range.0;
            total / (n + (n - 1.0) * self.inner_padding + 2.0 * self.outer_padding)
        }

        pub fn map(&self, label: &str) -> Option<f64> {
            let idx = self.domain.iter().position(|l| l == label)?;
            let bw = self.bandwidth();
            let step = bw * (1.0 + self.inner_padding);
            Some(self.range.0 + self.outer_padding * bw + idx as f64 * step + bw / 2.0)
        }
    }
    ```

- [ ] Implement `CategoricalScale`: map categories to point positions (band centers)

- [ ] Write property test: all band positions are within range
- [ ] Write property test: bandwidth * n_categories does not exceed range
- [ ] Write property test: map returns None for unknown labels

- [ ] Write snapshot test: bar chart using BandScale
- [ ] Write snapshot test: scatter plot with categorical x-axis

### ColorScale

- [ ] Create `ColorScale` in `starsight-layer-2/src/scale/color.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct ColorScale {
        domain: (f64, f64),
        colormap: ColormapRef,       // reference to a prismatica colormap
        midpoint: Option<f64>,       // for diverging: the neutral center value
    }

    impl ColorScale {
        pub fn sequential(domain: (f64, f64), colormap: ColormapRef) -> Self { /* ... */ }
        pub fn diverging(domain: (f64, f64), colormap: ColormapRef, midpoint: f64) -> Self { /* ... */ }

        pub fn map(&self, val: f64) -> Color {
            let t = match self.midpoint {
                None => (val - self.domain.0) / (self.domain.1 - self.domain.0),
                Some(mid) => {
                    if val < mid { 0.5 * (val - self.domain.0) / (mid - self.domain.0) }
                    else { 0.5 + 0.5 * (val - mid) / (self.domain.1 - mid) }
                }
            };
            self.colormap.eval(t.clamp(0.0, 1.0) as f32)
        }
    }
    ```

- [ ] Implement diverging midpoint mapping: below midpoint → 0.0-0.5, above → 0.5-1.0

- [ ] Write snapshot test: heatmap with sequential ColorScale
- [ ] Write snapshot test: heatmap with diverging ColorScale centered at zero

### TickLocator and TickFormatter traits

- [ ] Define traits in `starsight-layer-2/src/tick/mod.rs`:

    ```rust
    pub trait TickLocator {
        fn locate(&self, domain: (f64, f64), target_count: usize) -> Vec<f64>;
    }

    pub trait TickFormatter {
        fn format(&self, value: f64) -> String;
    }
    ```

- [ ] Implement `WilkinsonLocator` (default TickLocator)
- [ ] Implement `AutoFormatter` with step-size-based precision auto-detection

- [ ] Built-in locators: `LogLocator` (ticks at powers of 10 and optionally at 2 and 5 sub-ticks), `DateLocator` (ticks at time boundaries based on granularity), `FixedLocator` (user-specified positions).

- [ ] Implement `PercentFormatter` (multiply by 100, append %)
- [ ] Implement `SIFormatter` (k, M, G, T suffixes for large; m, u, n for small)
- [ ] Implement `DateFormatter` (epoch seconds → human-readable)
- [ ] Implement `FixedFormatter` (user-specified label strings)

- [ ] Ensure traits are object-safe: Axis holds `Box<dyn TickLocator>` and `Box<dyn TickFormatter>`

- [ ] Write snapshot test: default Wilkinson ticks
- [ ] Write snapshot test: log ticks on LogScale
- [ ] Write snapshot test: percentage-formatted ticks
- [ ] Write snapshot test: SI-formatted ticks (k, M, G suffixes)



### 0.6.0 — GPU and interactivity

Exit criteria: charts render in a native window with hover tooltips. GPU backend produces identical output to CPU backend.

- [ ] Create `WgpuBackend` struct in `starsight-layer-1/src/backend/wgpu/mod.rs`
- [ ] Initialize wgpu device, queue, and render pipeline
- [ ] Write vertex shader: transform data coordinates to clip space
- [ ] Write fragment shader: handle solid colors, gradients, and colormaps
- [ ] Implement Path-to-triangle tessellation (via lyon or manual ear-clipping)
- [ ] Implement `DrawBackend` for `WgpuBackend`
- [ ] Verify GPU output is visually identical to CPU backend

    ```rust
    pub struct WgpuBackend {
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: Option<wgpu::Surface<'static>>,  // None for headless
        texture: wgpu::Texture,                     // render target
        pipeline: wgpu::RenderPipeline,
    }
    ```

- [ ] Implement GPU texture readback: copy texture to CPU buffer
- [ ] Implement `save_png()` for WgpuBackend using readback buffer

- [ ] Create `InteractiveWindow` struct in `starsight-layer-6/src/window.rs`

    ```rust
    pub struct InteractiveWindow {
        event_loop: EventLoop<()>,
        window: Window,
        backend: WgpuBackend,
        figure: Figure,
        hover_state: Option<HoverInfo>,
        zoom: ZoomState,
    }
    ```

- [ ] Implement winit event loop: create window, initialize WgpuBackend, enter render loop
- [ ] Implement frame lifecycle: check input events → update state → re-render if dirty
- [ ] Implement hit-testing: spatial index for mark bounding boxes (grid for scatter, sequential for lines)
- [ ] Implement hover tooltips: on mouse move, hit-test marks, render tooltip box near cursor with data values
- [ ] Implement box zoom: click-drag draws selection rect, on release update scale domains, double-click resets
- [ ] Implement wheel zoom: scroll scales both axes around cursor, shift = x only, ctrl = y only
- [ ] Implement pan: middle-click-drag or shift-click-drag translates both axes
- [ ] Implement legend toggle: click legend entry to hide/show corresponding mark, dim hidden entries
- [ ] Implement `Figure::push_data(series_id, new_points)` for streaming data
- [ ] Implement rolling window: ring buffer for O(1) append and trim, re-render on each push
- [ ] Write snapshot test: static render through WgpuBackend matches SkiaBackend output
- [ ] Write integration test: window opens, renders chart, closes without panic

### 0.7.0 — 3D visualization

Exit criteria: 3D surface, scatter, wireframe, and line charts render with camera orbit.

- [ ] Create `Scene3D` in `starsight-layer-3/src/marks3d/mod.rs`. Uses nalgebra for camera transforms (projection matrix, view matrix, model matrix).

    ```rust
    pub struct Camera3D {
        position: nalgebra::Point3<f64>,
        target: nalgebra::Point3<f64>,
        up: nalgebra::Vector3<f64>,
        fov: f64,            // field of view in degrees
        near: f64,
        far: f64,
    }

    impl Camera3D {
        pub fn view_projection(&self, aspect: f64) -> nalgebra::Matrix4<f64> {
            let view = nalgebra::Isometry3::look_at_rh(&self.position, &self.target, &self.up);
            let proj = nalgebra::Perspective3::new(aspect, self.fov.to_radians(), self.near, self.far);
            proj.as_matrix() * view.to_homogeneous()
        }
    }
    ```

- [ ] Implement `Surface3DMark`: accept 2D grid of z-values
- [ ] Tessellate grid into triangles
- [ ] Project each vertex through camera view-projection matrix
- [ ] Sort triangles by depth (painter's algorithm) for CPU backend
- [ ] Color each face by z-value using prismatica colormap

- [ ] Implement `Scatter3DMark`: project 3D points to 2D screen
- [ ] Attenuate circle size by depth
- [ ] Sort back-to-front for correct overlap

- [ ] `Wireframe3DMark`: same as Surface3D but renders edges only (no filled faces).

- [ ] Implement `Line3DMark`: project 3D polyline to 2D

- [ ] Implement camera orbit: click-drag rotates via spherical coordinates
- [ ] Implement camera zoom: scroll moves camera toward/away from target

### 0.8.0 — Terminal backend

Exit criteria: `figure.show_terminal()` renders a chart inline in a terminal emulator.

- [ ] Protocol detection cascade in `starsight-layer-7/src/terminal/mod.rs`:

    ```rust
    pub fn detect_protocol() -> TerminalProtocol {
        if kitty_supported() { return TerminalProtocol::Kitty; }
        if sixel_supported() { return TerminalProtocol::Sixel; }
        if iterm2_supported() { return TerminalProtocol::ITerm2; }
        TerminalProtocol::Braille  // universal fallback
    }
    ```

    Detection: query the terminal with escape sequences and parse the response. Kitty sends `\x1b_Gi=31;OK\x1b\\` in response to a query. Sixel support is indicated by Device Attributes response containing `4`. iTerm2 responds to `\x1b[>0q` with version info.

- [ ] Implement Kitty backend: encode PNG bytes with Kitty graphics protocol
- [ ] Send `\x1b_Gf=100,a=T,t=d,s=W,v=H;BASE64DATA\x1b\\` escape sequence

- [ ] Implement Sixel backend: convert Pixmap to Sixel using `icy_sixel`

- [ ] iTerm2 backend: encode the PNG bytes as base64, send `\x1b]1337;File=inline=1;size=N:BASE64DATA\x07`.

- [ ] Implement Braille backend: map pixel brightness to Braille dot patterns (U+2800-U+28FF)
- [ ] Each character represents 2x4 pixel grid for sub-character resolution

- [ ] Implement half-block backend: use U+2580/U+2584 with ANSI 24-bit color
- [ ] Each cell represents two vertically stacked pixels

- [ ] Implement `ratatui::Widget` for `StarsightWidget`
- [ ] Render Figure into ratatui buffer area using detected protocol
- [ ] Write integration test: StarsightWidget renders in ratatui app

### 0.9.0 — All chart types

Exit criteria: every chart type from the gallery reference (70 types) has an implementation and a snapshot test.

- [ ] ErrorBarMark: vertical and horizontal error bars (struct + Mark impl + snapshot test)
- [ ] StepMark: staircase interpolation between points (struct + Mark impl + snapshot test)
- [ ] RidgelineMark: overlapping KDE distributions per category (struct + Mark impl + snapshot test)
- [ ] StripMark: categorical scatter with random jitter (struct + Mark impl + snapshot test)
- [ ] SwarmMark: beeswarm layout for categorical scatter (struct + Mark impl + snapshot test)
- [ ] RugMark: tick marks along an axis edge (struct + Mark impl + snapshot test)
- [ ] LollipopMark: stem + dot (struct + Mark impl + snapshot test)
- [ ] DumbbellMark: two dots connected by a line (struct + Mark impl + snapshot test)
- [ ] WaterfallMark: cumulative bar chart with color for up/down (struct + Mark impl + snapshot test)
- [ ] PolarMark: data on polar coordinates (struct + Mark impl + snapshot test)
- [ ] RadarMark: multi-variable radar/spider chart (struct + Mark impl + snapshot test)
- [ ] TreemapMark: nested rectangles via squarified algorithm (struct + Mark impl + snapshot test)
- [ ] SunburstMark: nested arcs for hierarchical data (struct + Mark impl + snapshot test)
- [ ] SankeyMark: flow diagram with node-link layout (struct + Mark impl + snapshot test)
- [ ] ChordMark: circular flow between groups (struct + Mark impl + snapshot test)
- [ ] NetworkMark: force-directed graph layout (struct + Mark impl + snapshot test)
- [ ] ParallelCoordsMark: multi-axis parallel lines (struct + Mark impl + snapshot test)
- [ ] StreamgraphMark: stacked area with baseline centering (struct + Mark impl + snapshot test)
- [ ] SlopeMark: paired line segments between two positions (struct + Mark impl + snapshot test)
- [ ] FunnelMark: horizontally centered decreasing bars (struct + Mark impl + snapshot test)
- [ ] GaugeMark: arc-based indicator (struct + Mark impl + snapshot test)
- [ ] Run `cargo xtask gallery` to generate all reference images
- [ ] Compare gallery output against GALLERY_REFERENCE.md entries

### 0.10.0 — Export and WASM

Exit criteria: charts export to PDF and render in a web browser via WASM.

- [ ] Create `PdfBackend` struct wrapping krilla's Document and Page
- [ ] Implement `DrawBackend::fill_rect()` for `PdfBackend`
- [ ] Implement `DrawBackend::draw_path()` for `PdfBackend`
- [ ] Implement `DrawBackend::draw_text()` for `PdfBackend`
- [ ] Embed fonts as subsets in PDF output
- [ ] Write snapshot test: PDF output opens correctly

    ```rust
    pub struct PdfBackend {
        document: krilla::Document,
        page: krilla::Page,
        font: krilla::Font,
    }

    impl DrawBackend for PdfBackend {
        fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
            self.page.fill_rect(rect.to_krilla(), color.to_krilla());
            Ok(())
        }
        // ...
    }
    ```

- [ ] Generate self-contained HTML file with embedded SVG chart
- [ ] Write JS runtime (< 5KB) for pan, zoom, and tooltip interactions
- [ ] Ensure HTML file has zero external dependencies

- [ ] Implement GIF frame rendering: render each frame as Pixmap
- [ ] Implement color quantization: median-cut algorithm to 256 colors
- [ ] Encode frames with `gif` crate
- [ ] Accept frame iterator or per-frame closure API

- [ ] Compile starsight to `wasm32-unknown-unknown`
- [ ] Verify wgpu backend works via WebGPU in browsers
- [ ] Implement SVG fallback for browsers without WebGPU (via web-sys)
- [ ] Bundle font data for cosmic-text (no system font access in WASM)
- [ ] Write test: chart renders correctly in WASM target

### 0.11.0 — Polish

Exit criteria: the API is clean, all major input formats are supported, and the recipe system works.

- [ ] Design recipe proc macro: `#[starsight::recipe]` on a function
- [ ] Generate struct from function parameters
- [ ] Generate builder-style setters for each parameter
- [ ] Generate `Mark` trait implementation
- [ ] Write the proc macro in a separate `starsight-macros` crate
- [ ] Write test: recipe macro compiles and produces a usable mark type

    ```rust
    #[starsight::recipe]
    fn sparkline(data: &[f64], width: f32, height: f32, color: Color) -> Figure {
        // The macro generates SparklineRecipe struct with builder methods
        // and registers it as a mark type
    }
    // Usage: figure.add(SparklineRecipe::new(&data).width(100.0).color(Color::BLUE));
    ```

- [ ] Implement `From<Array1<f64>>` for DataSource (ndarray integration)
- [ ] Implement `From<Array2<f64>>` for MatrixSource (ndarray integration)

- [ ] Accept `arrow::RecordBatch` as data source
- [ ] Extract columns by name from RecordBatch
- [ ] Use zero-copy conversion from Arrow arrays where possible

- [ ] Walk through every public type against Rust API Guidelines checklist
- [ ] Fix naming inconsistencies
- [ ] Add missing trait implementations (Debug, Clone, Display, Default)
- [ ] Ensure all builders follow the same pattern consistently

### 0.12.0 — Documentation

Exit criteria: every public item has documentation. The gallery is complete.

- [ ] Run `RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace`
- [ ] Fix every missing_docs warning
- [ ] Ensure each doc comment has: one-line summary, parameter descriptions, example, links to related types

- [ ] Write `examples/quickstart.rs` — the simplest possible chart (3 lines)
- [ ] Write `examples/line_chart.rs` — basic line chart with title and labels
- [ ] Write `examples/scatter.rs` — scatter plot with colored groups
- [ ] Write `examples/bar_chart.rs` — grouped bar chart
- [ ] Write `examples/histogram.rs` — histogram with KDE overlay
- [ ] Write `examples/heatmap.rs` — annotated heatmap with diverging colormap
- [ ] Write `examples/statistical.rs` — box plot + violin side by side
- [ ] Write `examples/faceting.rs` — faceted scatter by category
- [ ] Write `examples/custom_theme.rs` — applying a chromata theme
- [ ] Write `examples/terminal.rs` — inline terminal rendering
- [ ] Write `examples/interactive.rs` — windowed chart with hover and zoom
- [ ] Write `examples/polars_integration.rs` — chart from a Polars DataFrame

- [ ] Verify `cargo xtask gallery` runs all examples and saves PNGs to `gallery/`
- [ ] Update GALLERY_REFERENCE.md with generated images

- [ ] docs.rs configuration in workspace Cargo.toml:

    ```toml
    [package.metadata.docs.rs]
    all-features = true
    rustdoc-args = ["--cfg", "docsrs"]
    ```

    Use `#[cfg_attr(docsrs, doc(cfg(feature = "gpu")))]` to annotate feature-gated items so docs.rs shows which features enable which types.

### 1.0.0 — Stable release

Exit criteria: the library is production-ready. The public API is stable and will not change without a major version bump.

- [ ] cargo-semver-checks clean pass: no public API changes that violate semver since the last pre-release.

- [ ] CI green on stable Rust (Linux, macOS, Windows)
- [ ] CI green on MSRV 1.85 (Linux, macOS, Windows)
- [ ] All feature combinations compile (`cargo hack check --each-feature`)
- [ ] All tests pass (`cargo test --workspace --all-features`)
- [ ] All snapshot baselines match (`cargo insta test`)

- [ ] Create benchmark suite with criterion
- [ ] Benchmark: 1000-point line chart at 800x600 renders under 50ms
- [ ] Benchmark: 100,000-point scatter plot renders under 500ms

- [ ] Security audit: `cargo audit` and `cargo deny check advisories` report no known vulnerabilities in the dependency tree.

- [ ] Generate complete changelog with git-cliff from all pre-release versions
- [ ] Review every changelog entry
- [ ] Publish changelog on GitHub releases

- [ ] Write announcement blog post with example charts and comparisons
- [ ] Post to Reddit r/rust
- [ ] Post to Hacker News
- [ ] Post to Rust users forum
- [ ] Post to Twitter/Mastodon


---
---



---
---

# Part 3 — Look up

Quick-reference for type signatures, API details, conversion formulas, and dependency specifics. Come here mid-implementation when you need to check something.

---

## tiny-skia 0.12 API reference

### Color types

tiny-skia has four color types. Use `from_rgba8` for starsight (infallible, takes u8 values). Use `from_rgba` only when accepting user-supplied floats (returns None if out of range). The premultiplied type is internal to tiny-skia — you should never need to construct one directly.

| Type | Fields | Alpha | Constructor | Returns |
|------|--------|-------|-------------|---------|
| `Color` | f32 × 4 | Straight | `from_rgba(r,g,b,a)` | `Option<Self>` (None if out of 0.0-1.0) |
| `Color` | f32 × 4 | Straight | `from_rgba8(r,g,b,a)` | `Self` (infallible) |
| `ColorU8` | u8 × 4 | Straight | `from_rgba(r,g,b,a)` | `Self` (const, infallible) |
| `PremultipliedColorU8` | u8 × 4 | Premultiplied | `from_rgba(r,g,b,a)` | `Option<Self>` (None if channel > alpha) |

### Drawing methods (all take `Option<&Mask>` as final param)

These are the four drawing methods on `Pixmap`. Every method takes a `Transform` (use `identity()` for no transformation) and an optional `&Mask` for clipping (pass `None` to draw everywhere, or `Some(&mask)` to restrict drawing to the plot area).

```rust
pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
pixmap.fill_rect(rect, &paint, Transform::identity(), None);
pixmap.draw_pixmap(x: i32, y: i32, pixmap_ref, &pixmap_paint, Transform::identity(), None);
```

### PathBuilder

Build paths incrementally using move/line/curve commands. Always start with `move_to`. `finish()` returns `Option<Path>` — returns `None` if no segments were added. Static constructors `from_rect` and `from_circle` are shortcuts for common shapes.

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

All fields are public. Create with struct literal syntax.

```rust
Stroke {
    width: 2.0,                  // line thickness in pixels
    miter_limit: 4.0,            // prevents spikes at acute angles (default 4.0)
    line_cap: LineCap::Round,    // Butt | Round | Square — endpoint shape
    line_join: LineJoin::Round,  // Miter | MiterClip | Round | Bevel — corner shape
    dash: StrokeDash::new(vec![10.0, 5.0], 0.0), // returns Option — [visible, gap] pattern
}
```

LineCap variants: `Butt` (flat cut at endpoint, default), `Round` (semicircle extending beyond endpoint), `Square` (rectangle extending by half stroke width). For chart axis lines, use `Butt`. For data lines that end at the plot boundary, use `Butt` to avoid visual overshoot.

LineJoin variants: `Miter` (sharp point, constrained by miter_limit), `MiterClip` (mitered but clipped at miter_limit distance), `Round` (circular arc at corner), `Bevel` (flat cut at corner). For chart lines with sharp turns, `Round` prevents spikes. For rectangular elements like bars, any join works since corners are 90 degrees.

StrokeDash: the array alternates between visible and gap lengths. `[10.0, 5.0]` means 10px visible, 5px gap. `[5.0, 3.0, 1.0, 3.0]` means 5px dash, 3px gap, 1px dot, 3px gap. The offset shifts the starting position along the pattern. Returns `None` if the array is empty.

### Transform — DEGREES not radians

Six fields: sx, kx, ky, sy, tx, ty. All f32. All public. Represents an affine transformation matrix.

```rust
Transform::identity()                        // no transformation
Transform::from_translate(tx, ty)            // shift position
Transform::from_scale(sx, sy)               // stretch/shrink
Transform::from_rotate(degrees)              // NOT radians — this is the #1 gotcha
Transform::from_rotate_at(degrees, cx, cy)  // rotate around a point
Transform::from_row(sx, ky, kx, sy, tx, ty) // manual matrix (note: ky before kx)
```

Composition methods (all return a new Transform, do not mutate):

```rust
t.pre_translate(tx, ty)   // apply translate BEFORE existing transform (to input points)
t.pre_scale(sx, sy)       // apply scale BEFORE existing
t.pre_rotate(degrees)     // apply rotate BEFORE existing
t.post_translate(tx, ty)  // apply translate AFTER existing transform (to output points)
t.post_scale(sx, sy)      // apply scale AFTER existing
t.post_concat(other)      // combine: other applied AFTER self
```

For chart rendering: start with `identity()`, then `pre_translate` to position the element, then `pre_scale` for DPI scaling. The composition order matters: `translate then rotate` produces different results than `rotate then translate`.

### PNG export

```rust
// Save directly to file
pixmap.save_png("file.png")?;

// Encode to memory (for snapshot tests, HTTP responses, embedding)
let bytes: Vec<u8> = pixmap.encode_png()?;

// DPI metadata: PNG stores pixels-per-meter in the pHYs chunk
// 72 DPI  = 2835 pixels/meter
// 96 DPI  = 3780 pixels/meter (standard screen)
// 150 DPI = 5906 pixels/meter
// 300 DPI = 11811 pixels/meter (print quality)
```

The `encode_png` method automatically converts from premultiplied alpha (tiny-skia's internal format) to straight alpha (PNG's format). You do not need to manually demultiply.

For snapshot testing, use `encode_png()` to get bytes and pass them to `insta::assert_binary_snapshot!(".png", bytes)`. The PNG encoding is deterministic: the same pixel data always produces the same bytes, making byte-for-byte comparison reliable.

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

Call this after `shape_until_scroll` to get the bounding box of the laid-out text. The width is the widest line. The height is the bottom of the last line. Use these values for margin calculation (measuring y-axis tick label widths and title heights).

```rust
let (mut w, mut h) = (0.0f32, 0.0f32);
for run in buffer.layout_runs() {
    w = w.max(run.line_w);
    h = run.line_top + run.line_height;
}
```

### Draw onto tiny-skia (NO channel swap for file output)

The `draw` callback fires once per glyph rectangle. Each rectangle has position, size, and a color with alpha representing pixel coverage. Paint each rectangle onto the pixmap. Do NOT swap red and blue channels — the channel swap in cosmic-text's example code is only needed for softbuffer display targets, not for PNG or SVG file output.

```rust
buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    if let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
});
```

### Embed custom font

Call this before creating any `Buffer` to ensure the font is available for text layout. Use `include_bytes!` to bundle the font into the binary at compile time. For snapshot test determinism, always embed a specific font rather than relying on system fonts (which differ across OS).

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

Choose the colormap family based on the data semantics. Sequential maps encode magnitude (low to high). Diverging maps encode deviation from a central value (negative through zero to positive). Cyclic maps wrap around (useful for angles and phases). Discrete palettes assign distinct colors to categories.

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

Computing margins is a two-pass process. First pass: create scales from data ranges, run ticks, format labels, measure text widths using cosmic-text. This gives the actual margin sizes. Second pass: recompute the plot area with the real margins and re-run scales if needed. The formulas below describe the first pass.

```
left_margin   = pad + y_label_height + label_pad + max_ytick_width + tick_pad
bottom_margin = pad + x_label_height + label_pad + xtick_height + tick_pad
plot_width    = figure_width  - left_margin - right_margin
plot_height   = figure_height - top_margin  - bottom_margin
max_ytick_width = max(len(format(tick))) * font_size * 0.6
```

---

## Error handling pattern

The StarsightError enum has seven variants covering all failure modes. All variants are non-exhaustive so new error kinds can be added without breaking downstream code. The Result type alias saves typing throughout the codebase. Use `#[from]` for automatic conversion from std io Error. All other errors wrap a descriptive String.

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

The plot! macro has three arms: DataFrame input with named columns, two-array input (x and y slices), and single-array input (y values only, x auto-generated as 0..n). The DataFrame arm accepts arbitrary key-value pairs that forward to builder methods. All arms return a Figure ready for `.save()` or `.show()`.

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

## Links and references

Primary dependencies and their documentation.

| Crate | docs.rs | GitHub |
|-------|---------|--------|
| tiny-skia | https://docs.rs/tiny-skia | https://github.com/linebender/tiny-skia |
| cosmic-text | https://docs.rs/cosmic-text | https://github.com/pop-os/cosmic-text |
| svg | https://docs.rs/svg | https://github.com/bodoni/svg |
| palette | https://docs.rs/palette | https://github.com/Ogeon/palette |
| image | https://docs.rs/image | https://github.com/image-rs/image |
| thiserror | https://docs.rs/thiserror | https://github.com/dtolnay/thiserror |
| insta | https://docs.rs/insta | https://github.com/mitsuhiko/insta |
| prismatica | — | https://github.com/resonant-jovian/prismatica |
| chromata | — | https://github.com/resonant-jovian/chromata |
| wgpu | https://docs.rs/wgpu | https://github.com/gfx-rs/wgpu |
| ratatui | https://docs.rs/ratatui | https://github.com/ratatui/ratatui |
| polars | https://docs.rs/polars | https://github.com/pola-rs/polars |
| krilla | https://docs.rs/krilla | https://github.com/LaurenzV/krilla |
| winit | https://docs.rs/winit | https://github.com/rust-windowing/winit |

### Theory and standards

- Wilkinson Extended ticks: https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/checklist.html
- Cargo SemVer compatibility: https://doc.rust-lang.org/cargo/reference/semver.html
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html
- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Edition 2024: https://doc.rust-lang.org/edition-guide/rust-2024/index.html
- Kitty graphics protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- Sixel: https://vt100.net/docs/vt3xx-gp/chapter14.html
- Crameri colormaps: https://www.fabiocrameri.ch/colourmaps/
- WCAG contrast: https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html

### API design

- https://rust-lang.github.io/api-guidelines/ — Rust API Guidelines (canonical)
- https://rust-lang.github.io/api-guidelines/type-safety.html — Type Safety (newtypes, builders)
- https://rust-lang.github.io/api-guidelines/interoperability.html — Common traits to implement
- https://rust-lang.github.io/api-guidelines/future-proofing.html — non_exhaustive, sealed traits
- https://deterministic.space/elegant-apis-in-rust.html — Elegant Library APIs (Pascal Hertleif)
- https://www.lpalmieri.com/posts/error-handling-rust/ — Error Handling Deep Dive (Luca Palmieri)
- https://burntsushi.net/rust-error-handling/ — Error Handling (Andrew Gallant)
- https://cliffle.com/blog/rust-typestate/ — Typestate Pattern (Cliff Biffle)
- https://rust-unofficial.github.io/patterns/patterns/creational/builder.html — Builder Pattern
- https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/ — Sealed Traits
- https://microsoft.github.io/rust-guidelines/ — Microsoft Pragmatic Rust Guidelines
- https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/ — SemVer edge cases
- https://www.lurklurk.org/effective-rust/ — Effective Rust (free online)
- https://mmapped.blog/posts/12-rust-error-handling — Designing Error Types
- https://www.philipdaniels.com/blog/2019/rust-api-design/ — API Design with AsRef, Into, Cow

### Architecture

- https://matklad.github.io/2021/08/22/large-rust-workspaces.html — Large Rust Workspaces
- https://matklad.github.io/2021/09/04/fast-rust-builds.html — Fast Rust Builds
- https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html — ARCHITECTURE.md advocacy
- https://corrode.dev/blog/tips-for-faster-rust-compile-times/ — Faster Compile Times
- https://nnethercote.github.io/perf-book/ — The Rust Performance Book
- https://github.com/johnthagen/min-sized-rust — Minimizing binary size
- https://rust-analyzer.github.io/book/contributing/architecture.html — rust-analyzer Architecture
- https://doc.rust-lang.org/cargo/reference/build-scripts.html — Build Scripts

### Patterns

- https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html — Newtype Pattern
- https://www.lurklurk.org/effective-rust/newtype.html — Newtype (Effective Rust)
- https://rust-unofficial.github.io/patterns/anti_patterns/deref.html — Deref Polymorphism anti-pattern
- https://doc.rust-lang.org/std/convert/trait.From.html — From trait (when to implement)
- https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html — Extension Traits (RFC 445)
- https://corrode.dev/blog/dont-use-preludes-and-globs/ — Don't Use Preludes And Globs

### Testing and safety

- https://docs.rs/insta — insta (snapshot testing)
- https://docs.rs/proptest — proptest (property testing)
- https://doc.rust-lang.org/nomicon/ — The Rustonomicon (unsafe code)
- https://github.com/rust-lang/miri — Miri (undefined behavior detection)
- https://burntsushi.net/unwrap/ — Using unwrap() in Rust is Okay

### Books

- https://rust-for-rustaceans.com/ — Rust for Rustaceans (Jon Gjengset)
- https://marabos.nl/atomics/ — Rust Atomics and Locks (Mara Bos, free)
- https://www.lurklurk.org/effective-rust/ — Effective Rust (David Drysdale, free)
- https://www.zero2prod.com/ — Zero to Production in Rust (Luca Palmieri)
- https://doc.rust-lang.org/book/ — The Rust Book (official)
- https://doc.rust-lang.org/rust-by-example/ — Rust By Example

### Community

- https://this-week-in-rust.org/ — This Week in Rust
- https://blessed.rs — Curated crate recommendations
- https://lib.rs/ — Alternative crates.io frontend
- https://users.rust-lang.org/ — Rust Users Forum
- https://play.rust-lang.org/ — Rust Playground

### Key RFCs

| RFC | Topic |
|-----|-------|
| 344 | Method naming conventions |
| 430 | Naming conventions (CamelCase, snake_case) |
| 505 | API documentation conventions |
| 1105 | API evolution / SemVer policy |
| 1270 | Deprecation attribute |
| 2495 | Minimum Supported Rust Version |
---
---


---

## Scale math formulas

### Linear scale

Maps a value from the data domain to the pixel range using linear interpolation. The inverse formula maps back from pixels to data coordinates (used for hover tooltips).

```
output = (input - domain_min) / (domain_max - domain_min) * (range_max - range_min) + range_min
inverse = (output - range_min) / (range_max - range_min) * (domain_max - domain_min) + domain_min
```

### Log scale

Maps using the logarithm of the input value. Spreads out small values and compresses large values. Essential for data spanning multiple orders of magnitude (e.g., frequency spectra, population sizes).

```
output = (log10(input) - log10(domain_min)) / (log10(domain_max) - log10(domain_min)) * range_extent + range_min
```

Domain must be strictly positive. Values <= 0 must be clipped or masked before mapping. Attempting to take log10 of zero or a negative number produces NaN or -infinity.

### Symlog scale

Symmetric log: behaves linearly near zero (within the threshold) and logarithmically beyond it. Handles data that crosses zero, unlike log scale which requires strictly positive values. The threshold parameter controls how wide the linear region is.

```
T(x) = sign(x) * log10(1 + |x| / threshold)
output = linear_map(T(input), T(domain_min), T(domain_max), range_min, range_max)
```

Default threshold = 1.0. Linear region width equals threshold on each side of zero.

### Band scale

Maps n categories to evenly spaced bands with configurable gaps. inner_padding is the gap between bands as a fraction of the step size. outer_padding is the gap at the edges. The bandwidth is the width of each band in pixels.

```
bandwidth = range_extent / (n + (n - 1) * inner_padding + 2 * outer_padding)
step = bandwidth * (1 + inner_padding)
position(i) = range_min + outer_padding * bandwidth + i * step + bandwidth / 2
```

---

## Color conversion formulas

### sRGB to linear

Input: sRGB channel value in [0, 1]. Output: linear light intensity in [0, 1]. The threshold 0.04045 prevents the power function from producing a discontinuity near zero. Use this when computing luminance or doing color blending (blend in linear space, convert back to sRGB for display).

```
if srgb <= 0.04045:
    linear = srgb / 12.92
else:
    linear = ((srgb + 0.055) / 1.055) ^ 2.4
```

### Linear to sRGB

Input: linear light intensity in [0, 1]. Output: sRGB channel value in [0, 1]. This is the inverse of the sRGB-to-linear conversion. Apply after blending or luminance computation to get back to display-ready values.

```
if linear <= 0.0031308:
    srgb = linear * 12.92
else:
    srgb = 1.055 * linear ^ (1/2.4) - 0.055
```

### WCAG relative luminance

Relative luminance measures perceived brightness on a 0-to-1 scale, where 0 is black and 1 is white. The coefficients reflect human eye sensitivity (most sensitive to green, least to blue). R, G, B must be linearized first (see sRGB-to-linear above). Use this for WCAG contrast ratio calculation and for choosing black vs white text over a colored background.

```
L = 0.2126 * R_linear + 0.7152 * G_linear + 0.0722 * B_linear
```

### WCAG contrast ratio

The contrast ratio ranges from 1:1 (identical colors) to 21:1 (black on white). Always put the lighter luminance in the numerator. The 0.05 term prevents division by zero and accounts for ambient light. starsight uses this to auto-select annotation text color in heatmaps: if the cell luminance is below 0.5, use white text; otherwise use black.

```
ratio = (L_lighter + 0.05) / (L_darker + 0.05)
AA normal text: >= 4.5
AA large text:  >= 3.0
AAA normal:     >= 7.0
```

### Premultiplied alpha

tiny-skia stores pixels in premultiplied alpha format internally. In premultiplied form, each RGB channel is already multiplied by the alpha value. This makes compositing faster (one multiply instead of three) but means raw pixel values look wrong if you interpret them as straight alpha. For fully opaque pixels (alpha=255), premultiplication is a no-op. The `encode_png` method automatically converts back to straight alpha.

```
premul_r = straight_r * alpha / 255
premul_g = straight_g * alpha / 255
premul_b = straight_b * alpha / 255

Source-over compositing (premultiplied):
result = source + destination * (1 - source_alpha / 255)
```

---

## Feature flag reference

| Feature | Crate | Optional dependency | Default |
|---------|-------|--------------------| --------|
| `default` | starsight | tiny-skia, cosmic-text, svg | Yes |
| `gpu` | starsight-layer-1 | wgpu | No |
| `terminal` | starsight-layer-7 | ratatui, crossterm, ratatui-image | No |
| `polars` | starsight-layer-5 | polars | No |
| `ndarray` | starsight-layer-5 | ndarray | No |
| `arrow` | starsight-layer-5 | arrow | No |
| `3d` | starsight-layer-3 | nalgebra | No |
| `pdf` | starsight-layer-7 | krilla | No |
| `interactive` | starsight-layer-6 | winit | No |
| `stats` | starsight-layer-3 | statrs | No |
| `image` | starsight-layer-7 | image | No |
| `gif` | starsight-layer-7 | gif | No |
| `serde` | all layers | serde | No |

---

## Testing patterns

### Snapshot test template

Every chart type needs a snapshot test. Create the backend at a fixed size, render the chart with deterministic data, encode to PNG, and pass to insta. On first run, `cargo insta review` shows the image and you accept it. On subsequent runs, any visual change fails the test.

```rust
#[test]
fn test_line_chart_basic() {
    let mut backend = SkiaBackend::new(800, 600).unwrap();
    backend.fill(Color::WHITE);
    
    // ... build and render chart ...
    
    let bytes = backend.png_bytes().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}
```

### Property test template

Property tests generate thousands of random inputs and check that invariants hold. Use these for mathematical code: scale mapping/inverse roundtrips, tick monotonicity, color clamping, rectangle intersection. proptest automatically shrinks failing inputs to the minimal case.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn scale_roundtrip(val in -1e6f64..1e6f64) {
        let scale = LinearScale::new((0.0, 100.0), (0.0, 800.0));
        let px = scale.map(val);
        let back = scale.inverse(px);
        prop_assert!((val - back).abs() < 1e-10);
    }
    
    #[test]
    fn ticks_are_monotonic(min in -1e6f64..0.0f64, max in 0.1f64..1e6f64) {
        let ticks = wilkinson_extended(min, max, 5);
        for pair in ticks.windows(2) {
            prop_assert!(pair[0] < pair[1]);
        }
    }
}
```

### Approximate float comparison

Never use `==` to compare floating point values in tests. Use a tolerance-based comparison. The formula below uses both absolute and relative tolerance: it handles values near zero (where relative tolerance fails) and large values (where absolute tolerance is too tight). A typical tolerance for scale roundtrip tests is 1e-10.

```rust
fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol || (a - b).abs() < tol * a.abs().max(b.abs())
}
```

---

## Common tiny-skia patterns

### Fill background

Set the entire pixmap to a solid color. Call this first before drawing anything. Without it, the pixmap is transparent black (all zeros), which produces a black background in PNG output.

```rust
pixmap.fill(tiny_skia::Color::from_rgba8(255, 255, 255, 255));
```

### Draw a horizontal line

Used for axis lines, tick marks, and grid lines. Set `anti_alias = false` for perfectly axis-aligned lines to keep them crisp (a single pixel wide instead of blurred across two).

```rust
let mut pb = PathBuilder::new();
pb.move_to(x1, y);
pb.line_to(x2, y);
let path = pb.finish().unwrap();
let mut paint = Paint::default();
paint.set_color_rgba8(0, 0, 0, 255);
paint.anti_alias = false; // crisp for axis-aligned lines
pixmap.stroke_path(&path, &paint, &Stroke::default(), Transform::identity(), None);
```

### Draw a filled rectangle

Used for bar chart bars, heatmap cells, legend swatches, and background fills. `from_xywh` returns `None` if width or height is zero or negative — always handle the Option. For bars growing downward from a value, use `from_ltrb(left, top, right, bottom)` instead.

```rust
let rect = tiny_skia::Rect::from_xywh(x, y, w, h).unwrap();
pixmap.fill_rect(rect, &paint, Transform::identity(), None);
```

### Create a clipping mask

Clipping restricts all subsequent drawing to the mask region. Use this for the plot area: create a mask matching the plot rectangle, then pass `Some(&mask)` to all mark rendering calls. This prevents data lines from overflowing into the axis/margin area. Create the mask once per render, not per draw call.

```rust
let mut mask = Mask::new(width, height).unwrap();
let clip_rect = PathBuilder::from_rect(
    tiny_skia::Rect::from_ltrb(left, top, right, bottom).unwrap()
);
mask.fill_path(&clip_rect, FillRule::Winding, false, Transform::identity());
// Pass Some(&mask) to drawing methods
```

### Render text

```rust
let mut font_system = FontSystem::new();
let mut cache = SwashCache::new();
let metrics = Metrics::new(font_size, line_height);
let mut buffer = Buffer::new(&mut font_system, metrics);
buffer.set_text(&mut font_system, text, &Attrs::new(), Shaping::Advanced, None);
buffer.set_size(&mut font_system, Some(max_width), Some(max_height));
buffer.shape_until_scroll(&mut font_system, true);

let mut paint = Paint::default();
buffer.draw(&mut font_system, &mut cache, cosmic_text::Color::rgba(0, 0, 0, 255), |x, y, w, h, color| {
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    if let Some(rect) = tiny_skia::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
});
```

---

## Margin computation algorithm

```
1. Create scales from data ranges
2. Run tick algorithm on each scale
3. Format tick labels as strings
4. Measure widest y-tick label: max_ytick_w = max(label_width(tick)) 
5. Measure x-tick label height: xtick_h = font_line_height
6. Compute margins:
   left   = padding + y_label_height + label_gap + max_ytick_w + tick_gap
   bottom = padding + x_label_height + label_gap + xtick_h + tick_gap
   top    = padding + title_height + title_gap (if title exists)
   right  = padding
7. Plot area = figure area minus margins
8. Update scale ranges to match plot area pixel dimensions
9. Re-run tick algorithm if scale ranges changed significantly
```


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

## Quick reference: which crate do I edit?

| I want to... | Edit this crate |
|---|---|
| Add a primitive type (Point, Color, Vec2) | starsight-layer-1 |
| Add or modify a backend (Skia, SVG, wgpu) | starsight-layer-1 |
| Add a scale or tick algorithm | starsight-layer-2 |
| Add a mark or stat transform | starsight-layer-3 |
| Add layout, faceting, or legends | starsight-layer-4 |
| Add data source support (Polars, Arrow) | starsight-layer-5 |
| Add interactivity or windowing | starsight-layer-6 |
| Add export format (PDF, WASM, terminal) | starsight-layer-7 |
| Add a public API type or the plot! macro | starsight (facade) |
| Add a build/codegen task | xtask |

## Common operations

| Operation | Command |
|---|---|
| Check everything compiles | `cargo check --workspace` |
| Run all tests | `cargo test --workspace` |
| Run tests for one crate | `cargo test -p starsight-layer-2` |
| Run with all features | `cargo test --workspace --all-features` |
| Format code | `cargo fmt --all` |
| Lint | `cargo clippy --workspace --all-targets` |
| License check | `cargo deny check` |
| Review snapshot changes | `cargo insta review` |
| Generate gallery | `cargo xtask gallery` |
| Check MSRV | `cargo +1.85.0 check --workspace` |

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

