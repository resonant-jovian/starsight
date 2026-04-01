# Everything you need to know to build starsight

Authored by Claude Opus 4.6 (Anthropic) with Albin Sjogren. A complete teaching document designed for text-to-speech. No code blocks, no backticks, no tables, no formatting that breaks screen readers. Just words.

This document assumes you know that in Rust, an impl block attaches methods to a type so you can call variable dot method, that String new creates an empty string, and that you have seen basic Rust syntax like let, fn, struct, and enum. It assumes high-school-level math and physics. It does not assume you know anything about computer graphics, data visualization, library design, or advanced Rust.

---

## What starsight is and why it needs to exist

Imagine you are a physicist running simulations in Rust. You have computed a million data points describing the trajectory of particles in a plasma. Now you want to see what the data looks like. In Python, you would type import matplotlib, call plot, and get a chart. In Rust, there is no equivalent. This is the gap that starsight fills.

starsight is a scientific visualization library for Rust. It takes numerical data and turns it into charts: line charts, scatter plots, bar charts, histograms, heatmaps, box plots, violin plots, contour maps, 3D surfaces, and about fifty other chart types. It produces PNG images, SVG vector graphics, PDF documents, and interactive displays in terminal windows and native GUI windows.

The Rust ecosystem has a few existing plotting libraries, but each has limitations. Plotters is the most mature, but its API is verbose, development has slowed, and it has a design flaw where its rendering trait requires the Sized bound, preventing runtime backend selection. plotly-rs and charming generate JavaScript specifications and require a web browser to render, defeating the purpose of a compiled language. egui-plot is locked to the egui GUI framework and cannot produce static images for publications. textplots draws only ASCII art in the terminal.

starsight aims to be the matplotlib of Rust: one library covering the full range of scientific visualization, from quick exploratory plots to publication-quality figures. It is organized into seven layers, each a separate crate. Layer one is rendering primitives and backends. Layer two is scales and axes. Layer three is marks (the visual elements like lines and bars). Layer four is layout (grids, faceting, legends). Layer five is the high-level API. Layer six is interactivity. Layer seven is animation and export. Each layer depends only on layers below it, enforced by the package manager.

starsight belongs to the resonant-jovian ecosystem. Its sister crates are prismatica (308 scientific colormaps as compile-time data) and chromata (1104 editor themes as compile-time data). These provide the actual color and theme systems starsight uses internally.

## Why you need to understand Rust deeply to build this

Building a visualization library exercises nearly every feature of the Rust language. You need ownership and borrowing to manage pixel buffers efficiently. You need traits and trait objects to support multiple rendering backends. You need generics to write reusable scales. You need error handling to report rendering failures gracefully. You need iterators to process data. You need closures because the text rendering library delivers glyph pixels through a callback. You need lifetimes because some types borrow data from others. You need smart pointers because marks are stored as boxed trait objects. You need modules and visibility to organize nine crates. You need macros for the plot shorthand. You need conditional compilation for optional features.

None of these topics is individually hard, but they interact in ways that only become apparent when building something substantial. This document teaches each piece and then shows how they fit together.


## How ownership works and why it matters for pixel buffers

Every value in Rust has exactly one owner. The owner is the variable that holds the value. When the owner goes out of scope, the value is automatically dropped: its memory is freed, file handles are closed, network connections are shut down. This automatic cleanup is called RAII: Resource Acquisition Is Initialization.

When you write let y equals x for most types, ownership transfers from x to y. This is called a move. After the move, x is no longer valid, and the compiler rejects any attempt to use it. This prevents double frees (freeing memory twice), use-after-free (accessing freed memory), and data races (concurrent unsynchronized access).

For starsight, think about a pixel buffer. A pixel buffer stores the color of every pixel in an image. For an 800 by 600 chart with 4 bytes per pixel (red, green, blue, alpha), that is nearly two megabytes of heap memory managed by a Vec. When you create a pixel buffer, the variable owns that allocation. If you move it to another variable, the original becomes invalid. You cannot accidentally have two variables pointing to the same buffer, which would create confusion about which one should free it.

What if you want to pass the buffer to a function that draws on it without giving up ownership? This is where borrowing comes in.

## How borrowing works and why rendering needs mutable references

Instead of transferring ownership, you can lend a value by creating a reference. A shared reference gives read-only access. You can have as many simultaneous shared references as you want. A mutable reference gives exclusive read-write access. You can have exactly one mutable reference at a time, and while it exists, no shared references can exist either.

This rule, sometimes called aliasing XOR mutability, is the cornerstone of Rust's memory safety. It prevents data races (two pieces of code reading and writing the same memory simultaneously), iterator invalidation (modifying a collection while iterating through it), and aliased mutation (two references disagreeing about what the data looks like).

For starsight, when you pass the pixel buffer to a drawing function, you pass a mutable reference. The function modifies pixels through this reference and returns it when done. The tiny-skia library's fill-path method takes a mutable reference to the Pixmap (the pixel buffer being modified) and shared references to the Path (the shape) and Paint (the color). The mutable reference ensures exclusive access during rendering. The shared references ensure the shape and color data is not modified mid-render.

This means you cannot use the same pixmap as both the rendering target and a pattern source simultaneously. That would require both a mutable reference (to write) and a shared reference (to read pattern data) to the same object. The compiler rejects this. The solution is to copy the pattern data first, or use separate buffers.

## What Copy and Clone mean

When you assign one variable to another, the default behavior is a move: the old variable becomes invalid. But some types are so small and simple that this restriction is unnecessary. These implement the Copy trait.

Copy is a marker trait with no methods. It tells the compiler: instead of moving, duplicate the value bit-for-bit. Both variables remain valid. Types that implement Copy include integers, floating point numbers, booleans, characters, shared references, and tuples or arrays where every element is also Copy. String, Vec, Box, and anything owning heap memory cannot implement Copy because duplicating them would require duplicating the heap allocation, which is not a simple bit copy.

Clone is different. Clone requires you to explicitly call dot clone, and the operation can be expensive. Cloning a String allocates new heap memory and copies all bytes. Clone is always visible in the code, so you know when you are paying the cost.

For starsight, the geometry types (Point, Vec2, Rect, Size, Color) should implement Copy because they are small, contain no heap data, and copying them is trivially cheap. Types that hold data arrays (like LineMark with its Vec of f64) should implement Clone but not Copy.

## What Debug, PartialEq, Eq, and Hash mean

The Debug trait generates a text representation useful for debugging. When a test assertion fails, Rust prints the Debug representation of both values so you can see what went wrong. Every public type in starsight should derive Debug.

PartialEq generates an equality comparison. The word Partial refers to partial equivalence relations from mathematics: not all values need to be comparable to themselves. The canonical example is floating-point NaN (Not a Number), which is defined to not equal itself. So f32 and f64 implement PartialEq (you can compare them) but not Eq (the comparison is not reflexive because NaN does not equal NaN).

Eq is a marker trait asserting that equality is reflexive: every value equals itself. It is required for types used as HashMap keys. Do not derive Eq on types containing floating-point fields.

Hash generates a hash function for HashMap and HashSet. The critical rule: if two values are equal according to PartialEq, their hashes must also be equal. Because NaN breaks this (two NaN values might have different bit patterns but are both "not equal to themselves"), f32 and f64 do not implement Hash. starsight's Color type uses u8 channels, so Hash is safe.

Default generates a value using the default for each field: zero for numbers, false for booleans, empty for strings, None for Options. Only derive Default when the default value is meaningful. A default Point at zero, zero makes sense. A default Figure with no data does not.


## What traits are and why starsight needs them

A trait in Rust defines a set of methods that a type can implement. If you have worked with interfaces in Java or protocols in Swift, traits are similar. You declare the trait with the method signatures, then write impl blocks for each type that provides the behavior.

For starsight, traits are essential because the library supports multiple rendering backends. The tiny-skia backend rasterizes to a pixel buffer on the CPU. The SVG backend generates an XML document. The PDF backend produces a PDF file. The wgpu backend renders on the GPU. The terminal backend outputs escape sequences. Each of these backends needs to support the same set of drawing operations: fill a rectangle, stroke a path, draw text. But each implements these operations completely differently.

The solution is a trait called DrawBackend. It declares methods like fill-rect, draw-path, draw-text, dimensions, and save-png. Each backend implements the trait in its own way. The SkiaBackend implementation of fill-rect writes pixels into a Pixmap. The SvgBackend implementation writes an XML rectangle element into a document. Same method name, completely different behavior.

When a chart mark (like LineMark) needs to render itself, it does not need to know which backend it is talking to. It calls the DrawBackend methods and the correct implementation runs. This is the fundamental power of traits: they decouple the interface (what operations are available) from the implementation (how those operations work).

## How trait objects enable runtime backend selection

There are two ways to use traits in Rust: static dispatch with generics, and dynamic dispatch with trait objects.

With generics, you write a function that is parameterized by a type that implements a trait. The compiler generates a specialized version of the function for each concrete type used. This is called monomorphization. The generated code is as fast as if you had written separate functions by hand, because the compiler knows the exact type at compile time and can inline the method calls. The downside is that the generic function must know the type at compile time.

With trait objects, you use the dyn keyword: dyn DrawBackend. A trait object is a fat pointer consisting of two machine-sized words (16 bytes on a 64-bit system). The first word points to the actual data. The second word points to a vtable, which is a table of function pointers generated at compile time for each concrete-type-and-trait pair. When you call a method on a trait object, the runtime loads the function pointer from the vtable and calls it indirectly. This is slightly slower than a direct call (because of the indirection and the inability to inline) but it enables runtime polymorphism: the concrete type is determined at runtime.

For starsight, trait objects are essential because the user decides the backend at runtime. When you call save with a file path, starsight checks the file extension: if it ends in png, it creates a SkiaBackend; if it ends in svg, it creates an SvgBackend. This decision happens at runtime, not at compile time. The Figure stores the backend as a dyn DrawBackend reference and calls methods through the vtable.

## Why the DrawBackend trait must be object-safe

Not all traits can be used as trait objects. A trait is object-safe (also called dyn-compatible) if it follows certain rules. First, none of its methods can use Self as a parameter type or return type (because the concrete type is erased behind the trait object). Second, none of its methods can have generic type parameters (because you cannot store infinite vtable entries for every possible type parameter). Third, the trait cannot require the Sized bound on Self (because trait objects are unsized by definition).

The Plotters library made the mistake of requiring Sized on its DrawingBackend trait. This means you cannot write dyn DrawingBackend. Every function that accepts a backend must be generic over the backend type, which means every function in the call chain must also be generic, all the way up. This makes it extremely difficult to extract helper functions or store backends in data structures. It is one of the most common complaints about Plotters.

starsight avoids this by keeping DrawBackend object-safe from the beginning. No Self in return types, no generic type parameters on methods, no Sized bound. The render method on Scene takes ampersand mut dyn DrawBackend. This enables runtime backend selection, heterogeneous backend storage, and clean API design.

## How generics work and when to use them instead of trait objects

Generics create specialized copies of code for each concrete type. When you write a function with a type parameter T that implements a trait bound, the compiler generates a separate version of the function for each T it encounters. The generated code is identical to hand-written specialized code: all method calls can be inlined, the optimizer can see through the abstractions, and there is no indirection overhead.

Use generics when: performance matters and inlining is important, the concrete type is known at compile time, and you do not need to store heterogeneous collections of different types.

Use trait objects when: you need to store different types in the same collection (like different mark types in a Vec), the concrete type is determined at runtime (like backend selection), or you want to reduce binary size (one function body shared across all types, instead of N copies).

For starsight, the rule is: concrete types at the bottom (where performance matters), trait objects in the middle (where heterogeneous collections are needed), and generics at the top (where user ergonomics matter). The DrawBackend trait is used through trait objects. The Mark trait is used through trait objects (a Figure stores Vec of Box of dyn Mark). The data acceptance functions in layer five use generics (impl Into of DataSource) for ergonomic callers.

## How the From and Into conversion traits work

The From trait defines a conversion from one type to another. When you implement From of SomeType for YourType, you are saying: given a SomeType value, I can produce a YourType value, and this conversion always succeeds.

The Into trait is the reverse direction. Implementing From automatically provides Into through a blanket implementation in the standard library. You should always implement From rather than Into, because you get Into for free.

The practical use in function signatures is to accept impl Into of YourType as a parameter. This means the caller can pass either a YourType directly (the Into implementation is the identity) or any type that converts to YourType. For example, a function set-color that accepts impl Into of Color lets the caller pass a Color, a three-element tuple (if you implement From of tuple for Color), or a chromata Color (if you implement From of chromata Color for starsight Color).

TryFrom and TryInto are the fallible versions. Instead of always succeeding, they return a Result. Use TryFrom when the conversion can fail: for example, converting a string to a Color might fail if the string is not valid hexadecimal.

There is a rule called the orphan rule that limits where you can implement traits. You can only implement a trait for a type if your crate defines either the trait or the type (or both). This prevents two different crates from implementing the same trait for the same type, which would create ambiguity. For starsight, this means you can implement From of chromata Color for starsight Color (because starsight Color is your type), but you cannot implement From of starsight Color for tiny-skia Color (because neither type is yours). The workaround is to add a method like to-tiny-skia on your Color type instead of a trait implementation.

## What the question mark operator does and how errors propagate

The question mark operator is a shorthand for error propagation. When you write a function call followed by a question mark, it means: if the result is Ok, unwrap the value and continue; if the result is Err, convert the error using From and return it from the current function.

This is enormously useful because it lets you write a chain of fallible operations without explicitly matching each result. Instead of writing four lines of match or if-let for every operation that might fail, you write one line with a question mark at the end. The errors propagate automatically.

The conversion part is key. If your function returns Result of T, StarsightError, and you call a function that returns Result of T, std io Error, the question mark operator calls From of io Error for StarsightError. This converts the io error into your error type. You just need to implement the From conversion (or use thiserror's from attribute to generate it automatically).

For starsight, nearly every function that interacts with the rendering backend, the file system, or external libraries returns a Result with StarsightError as the error type. The question mark operator propagates errors through the rendering pipeline without boilerplate.

## How thiserror generates error types

thiserror is a library that provides a derive macro for the standard Error trait. You define your error enum with variants, annotate each variant with an error attribute containing the display message, and optionally annotate fields with from (to auto-generate From implementations) or source (to link to an underlying cause).

For starsight, the error type has seven variants: Render (for rendering backend failures), Data (for data format issues), Io (for file system errors, with a from attribute on the std io Error field), Scale (for invalid scale configurations), Export (for output format problems), Config (for invalid configuration), and Unknown (for unexpected situations). The from attribute on the Io variant means the question mark operator automatically converts io errors. The other variants take String messages that describe the specific failure.

The thiserror derive generates implementations of the standard Error trait (which provides method chaining through the source method), the Display trait (using the message format strings), and any From implementations specified by from attributes. This saves about 50 lines of boilerplate per error type.

The alternative to thiserror is anyhow, which provides a type-erased error type for application code. Libraries should use thiserror (to give callers typed errors they can match on). Applications should use anyhow (when you just need to report errors to the user). starsight is a library, so it uses thiserror.

---

## Chapter Five: Iterators and how they process data

Scientific visualization is fundamentally about data processing. You have arrays of numbers and you need to transform them: scale them, filter out invalid values, pair them up, compute statistics. Rust's iterator system is the primary tool for this work.

### What an iterator is

An iterator is any value that produces a sequence of items one at a time. The Iterator trait has one required method: next, which returns either Some of the next item or None when the sequence is exhausted. Every other method on Iterator (and there are dozens) is built on top of next.

When you call iter on a Vec, you get an iterator over shared references to the elements. When you call iter_mut, you get an iterator over mutable references. When you call into_iter (or use a for loop, which calls into_iter implicitly), you get an iterator that consumes the collection and yields owned values.

### Transforming data with map and filter

The map method transforms each element. If you have an iterator of f64 values and you want to convert each to a pixel position, you call map with a closure that performs the conversion. The map method does not do anything immediately. It creates a new iterator that will apply the transformation when elements are consumed. This is called laziness, and it means you can chain many transformations together without creating intermediate collections.

The filter method selects elements. If you have an iterator of f64 values and some of them are NaN (Not a Number, representing missing data), you call filter with a closure that checks whether each value is finite. Only values that pass the test are yielded by the resulting iterator.

You can chain map and filter and other transformations together. An iterator chain that filters NaN values, maps through a scale function, and collects into a Vec performs all three operations in a single pass through the data, with no intermediate allocations.

### Collecting results

The collect method drives an iterator to completion and gathers the results into a collection. The type of collection is determined by the context: collecting into a Vec creates a vector, collecting into a String concatenates characters, collecting into a HashMap creates key-value pairs from an iterator of tuples.

A particularly powerful pattern is collecting an iterator of Result values into a Result of Vec. If all elements are Ok, you get Ok of the collected vector. If any element is Err, you get the first error. This is exactly what you need when parsing a list of data values that might individually be invalid.

### Pairing data with zip and enumerate

The zip method combines two iterators element-by-element into an iterator of pairs. If you have separate x and y arrays and you need to process them together, zip pairs the first x with the first y, the second x with the second y, and so on. When either iterator runs out, zip stops.

The enumerate method adds an index to each element: it turns an iterator of values into an iterator of (index, value) pairs. This is useful when you need to know the position of each data point, for example when computing bar positions from indices.

### Reducing with fold and sum

The fold method reduces an iterator to a single value by repeatedly applying a function. You provide an initial accumulator value and a closure that takes the current accumulator and the next element and produces a new accumulator. After all elements are consumed, the final accumulator is returned.

The sum method is a specialized fold that adds numeric values. The product method multiplies them. The min and max methods find extremes. For starsight, these are used to compute data ranges: the minimum and maximum of the x values determine the x axis domain.

### Why iterators are zero-cost

Rust's iterators compile down to the same machine code as hand-written loops. The compiler inlines the closure, eliminates the iterator struct, and fuses the chain of transformations into a single loop body. A chain of filter, map, and collect produces exactly the same assembly as a for loop with an if statement and a push. This is not a theoretical claim; it has been verified repeatedly by examining compiler output.

This matters for starsight because data processing is in the hot path. When you have a million data points and need to scale each one, you cannot afford overhead per element. Iterators give you expressive, composable code with the performance of handwritten C.

---

## Chapter Six: Closures and callbacks in rendering code

A closure is a function that captures variables from its surrounding scope. When you write a two-line closure inside a function, the closure can read (and sometimes write) variables that belong to the function. This makes closures perfect for short, context-dependent transformations like "scale this value using the domain that I computed three lines above."

### The three closure traits

Every closure in Rust automatically implements one or more of three traits, depending on how it uses captured variables.

Fn (with a capital F) is for closures that only read their captured variables. These closures can be called multiple times and do not change anything. An example is a closure that computes pixel position from data value using a scale: it reads the scale parameters but does not modify them.

FnMut is for closures that modify their captured variables. These closures can be called multiple times but each call might change state. An example is a closure that counts how many NaN values it has filtered out: each call increments the counter.

FnOnce is for closures that consume their captured variables. These closures can only be called once because calling them destroys the captured state. An example is a closure that moves a Vec into a function: after the first call, the Vec is gone.

These three traits form a hierarchy. Every Fn closure also implements FnMut (reading is a special case of mutating). Every FnMut closure also implements FnOnce (mutating is a special case of consuming). This means a function that accepts FnOnce can take any closure, while a function that requires Fn is more restrictive.

### Why cosmic-text's draw callback uses FnMut

When cosmic-text renders text, it does not return a list of pixels. Instead, it calls a callback function once for each pixel of each glyph. The callback receives the x position, y position, width, height, and color of each pixel. Your job is to paint that pixel onto the pixel buffer.

This callback is called hundreds or thousands of times (once per pixel of each character). It cannot be FnOnce because it is called more than once. It needs to be FnMut because it modifies the pixel buffer: each call writes a pixel. The pixel buffer is captured by mutable reference, and each callback invocation modifies it.

Understanding this is crucial for integrating cosmic-text with tiny-skia. You create a closure that captures a mutable reference to the Pixmap, and inside the closure, you write each glyph pixel into the appropriate position in the buffer. The closure trait bounds in cosmic-text's API enforce that this pattern works correctly.

### The move keyword

Sometimes you want a closure to own its captured variables instead of borrowing them. The move keyword before the closure parameters forces all captured variables to be moved into the closure. This is necessary when the closure needs to outlive the scope that created it, for example when passing a closure to another thread or storing it in a struct.

For starsight, move closures are used when creating callback handlers for interactive events: the event handler closure owns the chart state and modifies it when the user zooms or pans.

---

## Chapter Seven: Lifetimes and references that stay valid

You know that references borrow data without owning it. But how does Rust know that a reference is still valid? How does it prevent you from using a reference to data that has already been freed? The answer is lifetimes.

### What a lifetime is

A lifetime is a compile-time concept that describes how long a reference is valid. Every reference has a lifetime, but most of the time Rust infers it automatically. You only need to write lifetime annotations when the compiler cannot figure out the relationships on its own.

A lifetime is written as a tick followed by a name, like tick a. When you see "ampersand tick a T," it means "a reference to T that is valid for the lifetime called a." The name is arbitrary; tick a is conventional for the first lifetime, tick b for the second, and so on.

### Lifetime elision rules

Rust has three rules that automatically assign lifetimes to function signatures, so you rarely need to write them explicitly.

Rule one: each reference parameter gets its own lifetime. A function taking two references gets two independent lifetimes.

Rule two: if there is exactly one input lifetime (one reference parameter), it is assigned to all output lifetimes. This handles the common case where a function borrows one value and returns a reference derived from it.

Rule three: if one of the parameters is ampersand self or ampersand mut self, the lifetime of self is assigned to all output lifetimes. This handles methods that return references to data owned by the struct.

If these rules do not resolve all output lifetimes, you must annotate manually. This happens most often when a function takes multiple references and returns a reference, and the compiler needs to know which input the output is derived from.

### Lifetimes in structs

If a struct contains a reference, it must declare a lifetime parameter. This tells the compiler that the struct cannot outlive the data it references. For example, a struct that holds a reference to a slice of data points must declare a lifetime: the struct is valid only as long as the data it points to exists.

For starsight, most structs own their data rather than borrowing it. The Figure struct owns its marks. Each mark owns its data arrays. The SkiaBackend owns its pixel buffer. This avoids lifetime complexity at the cost of some extra copying when data enters the library. The tradeoff is worth it: owned data is simpler to reason about and compose.

### The static lifetime

The tick static lifetime means "valid for the entire duration of the program." String literals have the tick static lifetime because they are embedded in the binary and exist for as long as the program runs.

As a trait bound (T colon tick static), it means the type does not contain any non-static references. All owned types (String, Vec, Box) satisfy this bound. A common misconception is that tick static means the value lives forever. It does not. A String created at runtime and dropped after five lines satisfies T colon tick static because it owns its data (no dangling references), even though it does not live forever.

For starsight, Box of dyn DrawBackend has an implicit tick static bound, meaning the backend cannot borrow short-lived data. Since backends own their pixel buffers, this is naturally satisfied.

---

## Chapter Eight: Modules and how code is organized

A Rust project is organized into modules, which form a tree rooted at the crate. Understanding this tree is essential for navigating and extending starsight's nine-crate workspace.

### Files and modules

Every Rust file is a module. The crate root is lib dot rs for a library or main dot rs for a binary. When you write "mod foo" in the crate root, the compiler looks for either foo dot rs or foo slash mod dot rs. The first form is for leaf modules (a single file). The second form is for modules with sub-modules (a directory).

In starsight-layer-1, the lib dot rs file declares three modules: backend, error, and primitives. The backend module has sub-modules for each rendering backend: skia, svg, pdf, wgpu, and terminal. The skia module has further sub-modules: raster, headless, and png. This creates a tree of modules that mirrors the directory structure.

### Visibility and pub

By default, everything in Rust is private. Only the module that defines an item (and its child modules) can access it. The pub keyword makes an item visible to the outside world.

There are fine-grained visibility modifiers. "pub crate" makes an item visible within the current crate but not to external users. This is useful for internal helper functions that multiple modules within a crate need to share but that should not be part of the public API. "pub super" makes an item visible to the parent module only.

For starsight, the public API is defined by what is re-exported through the facade crate. Internal crate types use pub crate for cross-module visibility within a layer crate. Only types that are meant for users to interact with directly are marked pub and re-exported through the facade.

### Re-exports and the facade pattern

The starsight facade crate (the one users add to their dependencies) re-exports types from the layer crates using "pub use." The user writes "use starsight prelude star" and gets access to Figure, Color, Point, the plot macro, and other commonly used types. They do not need to know about the internal layer structure.

This is the facade pattern: a simple public interface that hides internal complexity. The user depends on one crate (starsight) and gets a flat namespace of useful types. The internal organization into seven layers is an implementation detail that the user never needs to think about.

---

## Chapter Nine: How pixels and screens work

Now we leave the world of programming language features and enter the world of computer graphics. To build a visualization library, you need to understand what a pixel is, how images are stored in memory, and how shapes become colored dots on a screen.

### What a pixel actually is

A pixel is a point sample of a color at a specific position in a regular grid. It is not a little colored square, despite the common misconception. When you zoom into a digital image and see colored squares, you are seeing the result of a reconstruction filter (usually nearest-neighbor or bilinear), not the pixels themselves. The pixels are the data values at the grid intersections.

This distinction matters for understanding anti-aliasing and sub-pixel positioning, which we will discuss later. For now, the practical model is: an image is a rectangular grid of color values, each color value is associated with a position in the grid, and the grid has a width (number of columns) and a height (number of rows).

### How colors are represented

The most common color representation is RGBA: four numbers representing the intensity of red, green, blue, and alpha (transparency). Each channel is typically stored as an unsigned 8-bit integer (u8 in Rust), giving 256 levels per channel and approximately 16.7 million possible colors.

The alpha channel represents transparency. An alpha of 255 (the maximum) means fully opaque: you cannot see anything behind this pixel. An alpha of 0 means fully transparent: this pixel is invisible. An alpha of 128 means semi-transparent: you see a blend of this pixel's color and whatever is behind it.

### How pixel buffers work

A pixel buffer (also called a framebuffer or an image buffer) is a contiguous block of memory that holds all the pixels of an image. For an RGBA image, each pixel is four bytes, and the pixels are stored row by row from top to bottom, left to right within each row.

The total memory for an image is width times height times bytes-per-pixel. An 800 by 600 image at 4 bytes per pixel uses 1,920,000 bytes, approximately 1.83 megabytes. A high-DPI image at three times the resolution (2400 by 1800) uses 17,280,000 bytes, about 16.5 megabytes.

In tiny-skia, the pixel buffer is called a Pixmap. It owns a Vec of bytes in premultiplied RGBA format. The Pixmap new constructor takes a width and height and allocates the buffer filled with transparent black (all zeros).

### Screen coordinates versus mathematical coordinates

This is one of the most common sources of bugs in visualization code. Mathematical convention places the origin at the bottom left, with Y increasing upward. Screen convention places the origin at the top left, with Y increasing downward.

When you draw a chart, data values increase upward (temperature goes up, stock price goes up). But on screen, larger Y values move downward. This means the Y axis must be inverted when converting from data coordinates to screen coordinates. The conversion formula is: screen Y equals plot area bottom minus normalized Y times plot area height. The minus sign performs the inversion.

Every pixel-based graphics library (tiny-skia, Cairo, Skia, HTML Canvas, Vulkan) uses the top-left origin convention. SVG also uses it. PDF uses bottom-left, which is an exception that causes its own confusions. starsight uses top-left internally and handles the inversion in the coordinate mapping code so that marks and scales always work in mathematical coordinates.

### What premultiplied alpha is and why it matters

There are two ways to store a semi-transparent pixel. In "straight alpha" format, the RGB channels store the full color values, and the alpha channel is stored separately. To composite this pixel over a background, you first multiply the foreground color by alpha, then add the background color multiplied by one minus alpha.

In "premultiplied alpha" format, the RGB channels are already multiplied by the alpha value. A half-transparent red pixel in straight alpha is (255, 0, 0, 128). In premultiplied alpha, it is (128, 0, 0, 128). The red channel has been pre-multiplied by 128 divided by 255.

Premultiplied alpha is used by virtually all professional compositing systems and rendering libraries, including tiny-skia. The reason is that the compositing formula becomes simpler and faster. In premultiplied format, compositing source over destination is: result equals source plus destination times (1 minus source alpha). This is a single multiply-add per channel instead of the two multiplications and one addition needed for straight alpha. Over millions of pixels, this adds up.

The other advantage is that premultiplied alpha handles transparency gradients correctly. When you interpolate between two premultiplied colors, the result is correct. When you interpolate between two straight-alpha colors, bright fringes appear at the edges of transparent regions because the color and alpha are interpolated independently, producing colors that are brighter than they should be for their transparency level.

For starsight, the practical implication is: colors are stored internally as RGB u8 (with no alpha) matching the chromata and prismatica format. When painting onto the tiny-skia Pixmap, colors are converted to premultiplied RGBA. When encoding to PNG, tiny-skia demultiplies automatically. When reading pixels back for testing, remember that the stored values are premultiplied.

---

## Chapter Ten: How lines and curves are drawn

Drawing a line between two points on a pixel grid is more subtle than it sounds. The mathematical line is infinitely thin and continuous. The pixel grid is discrete. The rendering process must decide which pixels to color and how strongly.

### Bresenham's line algorithm

The classic algorithm for drawing lines on a pixel grid was developed by Jack Bresenham at IBM in 1962. It uses only integer addition and comparison (no multiplication, no division, no floating point) to determine which pixels to color.

The idea is simple. For a line that moves more in X than in Y (a shallow line), you step one pixel at a time in X and decide whether to also step in Y. The decision is based on an error term that tracks how far the actual line deviates from the current pixel row. When the error exceeds half a pixel, you step in Y and reset the error.

Bresenham's algorithm produces a staircase of pixels that approximates the line. For horizontal and vertical lines, the result is perfect. For diagonal lines at 45 degrees, the result is also perfect (alternating X and Y steps). For other angles, the staircase is visible as "jaggies" or "aliasing artifacts."

Modern renderers like tiny-skia do not use Bresenham's algorithm directly. They use coverage-based anti-aliasing, which we will discuss shortly. But understanding Bresenham's algorithm gives you the mental model of what line rasterization does: it decides which pixels to turn on.

### Bezier curves

Straight lines are not enough for smooth graphics. You need curves for chart annotations, arc charts, and smooth interpolation. The standard mathematical tool for curves in computer graphics is the Bezier curve.

A Bezier curve is defined by a set of control points. The curve starts at the first control point, ends at the last control point, and is "pulled toward" the intermediate control points without necessarily passing through them.

A linear Bezier curve is just a straight line between two points. Not very exciting, but it establishes the pattern.

A quadratic Bezier curve has three control points: start, control, and end. The curve starts at the start point, is pulled toward the control point, and ends at the end point. The shape is always a parabolic arc. TrueType fonts use quadratic Bezier curves to define letter shapes.

A cubic Bezier curve has four control points: start, control one, control two, and end. Having two control points gives much more flexibility. The curve can form S-shapes (inflection points) that are impossible with quadratic curves. PostScript, PDF, SVG, and OpenType CFF fonts all use cubic Bezier curves.

The mathematical formula for a cubic Bezier at parameter t (ranging from 0 to 1) is: B(t) equals (1-t) cubed times P0 plus 3 times (1-t) squared times t times P1 plus 3 times (1-t) times t squared times P2 plus t cubed times P3. At t equals 0, the curve is at P0 (the start). At t equals 1, the curve is at P3 (the end). The tangent at the start points from P0 toward P1, and the tangent at the end points from P2 toward P3.

### The de Casteljau algorithm

To evaluate a point on a Bezier curve, you can use the formula above, but there is a more elegant and numerically stable approach called de Casteljau's algorithm. It works by repeatedly applying linear interpolation.

For a cubic Bezier with four control points, at parameter t: first, linearly interpolate between adjacent pairs to get three new points. Then interpolate between those three to get two new points. Then interpolate between those two to get one final point. That final point lies on the curve.

The beauty of this algorithm is twofold. First, it only uses linear interpolation, which is numerically stable (no catastrophic cancellation). Second, the intermediate points form control points for two sub-curves that exactly subdivide the original curve at parameter t. This subdivision property is used in rendering: you recursively subdivide the curve until each piece is flat enough to approximate with a straight line, then draw the straight lines.

### Paths as sequences of commands

In computer graphics, shapes are described as paths: sequences of drawing commands. The commands are:

Move To, which lifts the pen and moves to a new position without drawing. Line To, which draws a straight line from the current position to a new position. Quad To, which draws a quadratic Bezier curve. Cubic To, which draws a cubic Bezier curve. Close, which draws a straight line back to the most recent Move To position, closing the shape.

A rectangle is four Line To commands between the four corners, followed by Close. A circle is approximated by four cubic Bezier arcs (each spanning 90 degrees), which is close enough to a true circle that the difference is invisible at any practical resolution.

In tiny-skia, paths are built using PathBuilder. You call move_to, line_to, quad_to, cubic_to, and close to accumulate commands. When you call finish, the builder validates the path and returns a Path value (or None if the path is empty).

### Fill rules: winding versus even-odd

When you have a closed path and you want to fill its interior with color, you need a rule to determine which pixels are "inside" the path. This is not always obvious, especially for paths that cross themselves.

The even-odd rule works by casting a ray from each pixel to infinity and counting how many times the ray crosses the path boundary. If the count is odd, the pixel is inside. If the count is even, the pixel is outside. This is simple and intuitive: each boundary crossing toggles inside and outside.

The winding rule also casts a ray but pays attention to the direction of each crossing. If the path crosses the ray from left to right, a counter increments. If it crosses from right to left, the counter decrements. If the final counter is non-zero, the pixel is inside. If it is zero, the pixel is outside.

The difference shows up with self-intersecting paths. Draw a five-pointed star (a pentagram) in a single continuous stroke. Under even-odd, the center of the star is not filled (the ray crosses the boundary an even number of times). Under winding, the center is filled (the winding number is non-zero throughout).

For charts, the winding rule is the default and the right choice. Most chart shapes (rectangles, circles, area fills) do not self-intersect, so the two rules give the same result. But the winding rule is slightly faster to evaluate and is the default in both tiny-skia and SVG.

### Stroking: how lines become visible shapes

A path itself is infinitely thin. To make it visible, you either fill its interior (producing a solid shape) or stroke it (drawing a visible outline along the path). Stroking is how chart lines, axis lines, and tick marks become visible.

Stroking a path means expanding it into a new filled path that represents the visible line. A line with width 2 means the visible area extends 1 pixel on each side of the mathematical path. The resulting filled area is like a ribbon centered on the original path.

Three properties control the appearance of strokes.

Line cap determines what happens at the open ends of a path. Butt cap (the default) cuts the line off exactly at the endpoint. Round cap adds a semicircle at the endpoint. Square cap extends the line by half the stroke width beyond the endpoint, creating a square end.

Line join determines what happens where two line segments meet at an angle. Miter join extends the outer edges until they meet at a point (sharp corners). Round join adds a circular arc at the corner. Bevel join cuts the corner flat. The miter join has a limit: if the angle is too acute, the miter point would extend too far, so it is automatically converted to a bevel. The default miter limit in tiny-skia is 4.0.

Dash pattern makes the line intermittent. You specify alternating lengths of visible and invisible segments. A dash pattern of 10, 5 means 10 pixels visible, 5 pixels invisible, repeating. An offset shifts the starting position of the pattern.

For chart rendering, the typical settings are: butt cap (clean line ends), miter join (sharp corners for axes), no dash pattern (solid lines for data, dashed for grid lines). Anti-aliasing is on for data lines (which are diagonal) and off for axis lines (which are horizontal or vertical and benefit from pixel-aligned crispness).

---

## Chapter Eleven: Anti-aliasing and why charts need it

Without anti-aliasing, diagonal lines and curves look jagged. The staircase pattern of discrete pixels is visible and distracting. Anti-aliasing smooths these edges by partially coloring pixels at the boundary of a shape.

### Coverage-based anti-aliasing

The idea is simple in principle. For each pixel near the edge of a shape, compute what fraction of the pixel area is covered by the shape. If a pixel is 70 percent covered, color it at 70 percent intensity (or more precisely, set its alpha to 70 percent and composite it over the background).

For a perfectly horizontal or vertical line that is aligned with the pixel grid, every pixel is either fully covered or not covered at all. Anti-aliasing adds nothing, and it can actually make the line look blurry (because pixels at the edge get partial coverage even though the line is perfectly sharp). This is why starsight disables anti-aliasing for axis lines and tick marks that are at exact integer coordinates.

For diagonal lines and curves, anti-aliasing transforms visible staircases into smooth gradients. The human eye perceives the partially colored edge pixels as a smooth line rather than a jagged staircase.

In tiny-skia, anti-aliasing is controlled by the anti_alias field on the Paint struct. It defaults to true. When enabled, the rasterizer computes per-pixel coverage rather than binary inside/outside. This is more expensive (the rasterizer must evaluate coverage at sub-pixel resolution) but produces much better visual quality.

### When to disable anti-aliasing

Disable anti-aliasing for: horizontal and vertical lines at integer coordinates (axis lines, tick marks, grid lines), rectangles with integer-aligned edges (bar chart bars), and text glyph compositing (the text rasterizer handles its own anti-aliasing).

Enable anti-aliasing for: diagonal lines (chart data lines), curves (smooth interpolation), circles (scatter plot points), and any shape with non-integer-aligned edges.

The practical rule: set anti_alias to true by default on the Paint, then set it to false in the specific drawing calls that render axis-aligned elements.

---

## Chapter Twelve: Color science for visualization

Color is one of the most important visual channels in data visualization, and one of the easiest to get wrong. Understanding how color works, both physically and perceptually, is essential for making charts that communicate data accurately.

### How human color vision works

The human retina contains three types of cone cells, each sensitive to a different range of wavelengths of light. The S cones (short wavelength) respond most to blue-violet light around 420 nanometers. The M cones (medium wavelength) respond most to green light around 530 nanometers. The L cones (long wavelength) respond most to red-orange light around 560 nanometers.

Because we have three types of color receptors, we can represent most visible colors by mixing three primary colors: red, green, and blue. This is the basis of the RGB color model. A computer monitor produces colors by varying the intensity of red, green, and blue light-emitting elements at each pixel.

### The sRGB color space

Not all RGB values represent the same physical colors. The sRGB standard (published in 1999) defines a specific mapping between RGB numbers and physical light intensities. It is the default color space for the web, for most computer monitors, and for PNG images.

The key property of sRGB is its transfer function, commonly called "gamma." The relationship between the number stored in a pixel (the encoded value) and the physical light intensity (the linear value) is nonlinear. Specifically, mid-gray is encoded as approximately 187 out of 255, not 128 as you might expect. This nonlinearity matches the human visual system's sensitivity: we can distinguish more shades of dark gray than light gray, so the encoding allocates more precision to dark values.

The exact formula is piecewise. For sRGB values at or below 0.04045, the linear value equals the sRGB value divided by 12.92. For sRGB values above 0.04045, the linear value equals the quantity (sRGB plus 0.055) divided by 1.055, raised to the power 2.4. The inverse formula converts from linear back to sRGB.

### Why color space matters for blending

When you mix two colors (for example, to create a gradient from blue to yellow, or to blend a semi-transparent scatter point over a background), the result depends on which color space you perform the arithmetic in.

Blending in sRGB space (which is what most renderers do by default, including tiny-skia) is technically incorrect because sRGB is nonlinear. Adding two nonlinear values does not give you the correct physical sum. The practical effect is that gradients appear slightly too dark in the middle and that transparent overlays are slightly dimmer than expected.

Blending in linear space (after converting from sRGB to linear, performing the arithmetic, and converting back) produces physically correct results. But it is slower because every pixel operation requires two gamma conversions. Most visualization libraries, including matplotlib, blend in sRGB space and accept the small inaccuracy.

For starsight, the default is sRGB blending (matching tiny-skia's default). Gamma-correct blending can be enabled via a configuration option for users who need it.

### WCAG contrast and luminance

The Web Content Accessibility Guidelines define a contrast ratio formula that measures how easily text can be read against its background. The formula uses relative luminance, which is a weighted sum of the linearized RGB channels: L equals 0.2126 times R linear plus 0.7152 times G linear plus 0.0722 times B linear. The weights reflect the human eye's sensitivity: green contributes most to perceived brightness, red contributes moderately, and blue contributes least.

The contrast ratio is the ratio of the lighter luminance plus 0.05 to the darker luminance plus 0.05. The result ranges from 1 (identical colors) to 21 (black and white). WCAG requires a contrast ratio of at least 4.5 to 1 for normal text and 3 to 1 for large text.

For starsight, contrast ratios determine whether axis labels are readable against the chart background, whether legend text is readable against its background, and whether data series colors are distinguishable from each other. The default theme uses dark gray text on a white background, which has a contrast ratio of about 12 to 1, well above the WCAG threshold.

### Why rainbow colormaps are harmful

The rainbow colormap (also called jet or HSV) maps data values to colors by sweeping through the hue spectrum: blue, cyan, green, yellow, red. It looks colorful and is immediately recognizable. It is also one of the worst colormaps for scientific visualization, for three reasons.

First, it is not perceptually uniform. The jump from green to yellow appears much more dramatic than the jump from blue to cyan, even though both represent the same change in data value. This creates artificial boundaries in the visualization where none exist in the data.

Second, it does not print well in grayscale. When printed on a black-and-white printer or viewed by someone with a monochrome display, green and red map to similar gray values, making them indistinguishable.

Third, it is inaccessible to colorblind users. About 8 percent of men and 0.5 percent of women have some form of color vision deficiency, most commonly red-green colorblindness. In a rainbow colormap, the red and green regions are indistinguishable to these users, hiding a large fraction of the data range.

The alternative is a perceptually uniform colormap like viridis, inferno, magma, or plasma. These colormaps vary primarily in lightness (bright to dark), with hue providing additional differentiation. They work in grayscale, are colorblind-safe, and do not create false boundaries. starsight uses viridis as the default sequential colormap because it satisfies all three criteria.


## How iterators process data

An iterator in Rust is any type that implements the Iterator trait. The Iterator trait has one required method: next, which returns either Some of the next value or None when the sequence is exhausted. Everything else is built on top of this single method.

Iterators are lazy: they do not compute anything until you ask for the next value. When you chain iterator operations together (like filter followed by map followed by collect), no actual computation happens at the chaining step. The chain just builds up a description of the computation. The work only happens when a consuming method (like collect, sum, or for-each) drives the iterator to produce values.

For starsight, iterators are the primary tool for data processing. When you have an array of data values and need to convert them to pixel coordinates, you iterate over the values, apply a scale mapping to each one, and collect the results. When you need to find the minimum and maximum of a data array (to compute the axis range), you use iterator methods like fold or the min-by and max-by methods.

The key iterator methods for data processing are: map, which transforms each element by applying a function; filter, which keeps only elements where a predicate returns true; zip, which pairs elements from two iterators (perfect for combining x-data and y-data into x-y pairs); enumerate, which adds an index to each element; chain, which appends one iterator after another; take, which stops after a certain number of elements; skip, which drops the first few elements; and collect, which gathers all elements into a collection like a Vec.

The fold method is particularly important. It takes an initial accumulator value and a function that combines the accumulator with each element to produce a new accumulator. You can compute sums, products, minimums, maximums, and any other reduction with fold. The sum method is a specialized fold with addition.

Iterator chains compile down to the same machine code as hand-written loops. The compiler monomorphizes each adapter (creating a concrete type for the specific chain), then inlines the next methods, and finally fuses everything into a single loop. A chain of filter, map, sum produces identical assembly to a for loop with an if statement and a plus-equals. This is what Rust calls zero-cost abstractions.

## How closures capture variables

A closure is an anonymous function that can capture variables from the scope where it is defined. When you write a closure like: the variable doubled equals a vertical bar followed by x followed by another vertical bar, then x times 2, you create a function that takes x and returns x times 2. This closure does not capture any variables from the surrounding scope.

But when you write: the variable offset equals 10, then the variable shifted equals vertical bar x vertical bar x plus offset, the closure shifted captures the variable offset from the surrounding scope. The compiler automatically determines how to capture it: by shared reference (if the closure only reads it), by mutable reference (if the closure modifies it), or by value (if the closure moves it or the variable is Copy).

There are three closure traits that describe what a closure does with its captures. FnOnce means the closure can be called once, consuming its captured variables. Every closure implements FnOnce. FnMut means the closure can be called multiple times and may mutate its captures. FnMut is a subtrait of FnOnce. Fn means the closure can be called multiple times and only reads its captures. Fn is a subtrait of FnMut.

The hierarchy is: every Fn closure can also be used as FnMut, and every FnMut closure can also be used as FnOnce. When choosing a bound for a function parameter, FnOnce is the most permissive (accepts any closure), and Fn is the most restrictive.

For starsight, closures matter most in the text rendering pipeline. cosmic-text's draw method takes an FnMut closure that receives the pixel coordinates, dimensions, and color of each glyph fragment. The closure is called once per pixel of each rendered glyph. It needs FnMut (not just Fn) because the closure typically writes to the pixel buffer, which requires mutable access to external state.

The move keyword on a closure forces all captured variables to be moved into the closure by value, even if the closure would normally borrow them. This is necessary when the closure needs to outlive the scope where it was created, such as when passing a closure to a thread or returning it from a function. The move keyword does not change which trait the closure implements; it only changes the capture mode.

## How lifetimes keep references valid

Lifetimes are the compiler's way of tracking how long references are valid. They are annotations that exist only at compile time; they generate no runtime code and are completely erased from the compiled binary.

When you create a reference, the compiler assigns it a lifetime: the region of code during which the reference is valid. The reference must not outlive the data it points to. If a function takes a reference and returns a reference, the compiler needs to know the relationship between their lifetimes: does the returned reference come from the input, or from somewhere else?

Most of the time, you do not need to write lifetimes explicitly because the compiler infers them through three rules. Rule one: each reference parameter gets its own lifetime. Rule two: if there is exactly one input reference, its lifetime is assigned to all output references. Rule three: if one of the inputs is ampersand self or ampersand mut self, the self lifetime is assigned to all output references. If these rules are not sufficient, you must annotate lifetimes explicitly.

For starsight, lifetimes mostly stay in the background. The primitive types (Point, Vec2, Color) are Copy and do not involve references. The rendering functions take references to paths, paints, and transforms, but the lifetime relationships are simple enough for the compiler to infer.

The main place lifetimes become explicit is in struct definitions that hold references. If a struct contains a reference to a slice of data (instead of owning a Vec), it must declare a lifetime parameter. For starsight, the design choice is to have marks own their data (using Vec) rather than borrow it, which avoids lifetime parameters on most types.

The special lifetime tick-static means "lives for the entire program." As a bound on a type parameter (T colon tick-static), it means the type does not contain any non-static references. All owned types satisfy this bound: a String created at runtime and dropped after five lines satisfies T colon tick-static because it owns its data. The common misconception is that tick-static means "lives forever at runtime," but it actually means "contains no borrowed data with a limited lifetime."

## How Box, Rc, and Arc work

Box of T allocates a value of type T on the heap and gives you a pointer to it. Box is the simplest smart pointer. It owns the value, and when the Box goes out of scope, the value is dropped and the memory is freed. Box adds one level of indirection: the Box itself is a single pointer on the stack, and the data lives on the heap.

For starsight, Box is used for trait objects. A Figure holds a Vec of Box of dyn Mark. Each Box of dyn Mark is a heap-allocated mark (like a LineMark or PointMark) erased behind the Mark trait. The Box provides ownership: the Figure owns the marks. The dyn Mark provides polymorphism: the Figure does not need to know the concrete mark types.

Box of dyn Trait is a fat pointer: two machine words. The first word points to the data on the heap. The second word points to the vtable for the concrete-type-and-trait pair. Method calls go through the vtable for dynamic dispatch.

Rc of T (Reference Counted) enables shared ownership. Multiple Rc pointers can point to the same heap-allocated value. Each time you clone an Rc (using Rc colon colon clone), the reference count increments. Each time an Rc goes out of scope, the count decrements. When the count reaches zero, the value is dropped. Rc is cheap to clone (just incrementing an integer) but is limited to single-threaded use because the reference count is not atomic.

Arc of T (Atomically Reference Counted) is the thread-safe version of Rc. It uses atomic operations for the reference count, which makes it safe to share across threads but slightly more expensive to clone. Arc is needed when you want multiple threads to share read-only access to the same data.

For starsight, Rc and Arc are not commonly needed in the core library. Marks own their data. The Figure owns the marks. The backends own their pixel buffers. The ownership hierarchy is clean and single-threaded. Arc might be needed for thread-safe font caching in the future, but for version 0.1.0, straightforward ownership suffices.

## How modules organize code in a Rust project

Rust uses a module system to organize code within a crate. The crate root (lib.rs for libraries, main.rs for binaries) is the top-level module. You declare submodules with the mod keyword. When you write mod foo as a statement (without a body block), the compiler looks for the module's code in a file called foo.rs in the same directory, or in a file called mod.rs inside a directory called foo.

Modules create a hierarchy of namespaces. Items (functions, types, traits, constants) defined inside a module are private by default, accessible only from the module itself and its children. The pub keyword makes an item public, accessible from outside the module. There are also restricted visibility modifiers: pub parenthesis crate means visible within the current crate but not externally, and pub parenthesis super means visible to the parent module.

For starsight, the module structure maps to the layer architecture. starsight-layer-1's lib.rs declares three modules: backend, error, and primitives. The backend module in turn declares submodules for each backend: skia, svg, pdf, wgpu, and terminal. The skia module declares submodules for different rendering modes: raster, headless, and png.

The pub use statement re-exports items from a submodule. This is how the facade crate (called starsight) presents a clean API: it re-exports the most important types from the layer crates so users only need to depend on the single starsight crate and import from its prelude.

## How Cargo workspaces manage multi-crate projects

A Cargo workspace is a set of related crates that share a single Cargo.lock file and output directory. The workspace is defined by a Cargo.toml at the project root that lists all member crates.

For starsight, the workspace contains nine members: the facade crate (starsight), seven layer crates (starsight-layer-1 through starsight-layer-7), and the xtask crate (a development automation tool). They all share the same version number, edition, license, and other metadata through workspace inheritance.

Workspace inheritance lets you define common settings once in the root Cargo.toml and reference them from member crates. In the root, you write a workspace.package section with the version, edition, authors, and license. In each member crate, you write version.workspace equals true instead of hardcoding the version. This keeps all crates in sync.

Dependencies can also be inherited. The root Cargo.toml has a workspace.dependencies section listing shared dependency versions. Member crates reference them with dependency.workspace equals true. This ensures all crates use the same version of each dependency and prevents version conflicts.

The workspace also inherits lint configuration. The root defines workspace.lints with clippy and rustc lint levels. Each member opts in with lints.workspace equals true. This is how starsight enforces consistent code quality: unsafe code is forbidden across all crates, and clippy pedantic warnings are enabled everywhere.

When you build or test the workspace (with cargo build or cargo test), Cargo builds all member crates together, sharing the target directory and the dependency cache. This is faster than building each crate independently because common dependencies are compiled only once.

## How feature flags enable optional functionality

Feature flags in Rust let you conditionally compile parts of your code. A feature is declared in Cargo.toml and can enable optional dependencies, activate other features, or gate code behind conditional compilation attributes.

For starsight, features control which backends and integrations are available. The default feature set includes CPU rendering via tiny-skia and SVG output. Optional features include: gpu (enables the wgpu rendering backend), terminal (enables Kitty, Sixel, and Braille terminal output), polars (enables Polars DataFrame input), ndarray (enables ndarray input), 3d (enables 3D chart types with nalgebra), pdf (enables PDF export with krilla), interactive (enables windowed interactive charts with winit), and stats (enables statistical chart types with statrs).

Features must be additive: enabling a feature must never remove functionality. This means you cannot use features for exclusive choices (like "use either backend A or backend B, but not both"). Both backends should be available simultaneously, and the user chooses at runtime.

A subtlety called feature unification affects workspaces. When Cargo builds a workspace, it builds shared dependencies with the union of all features requested by all member crates. If starsight-layer-7 enables the terminal feature and starsight-layer-2 does not, terminal is still enabled in the unified build because some crate needs it. This means a module that compiles in the workspace context might not compile standalone if it accidentally depends on a feature enabled by a sibling crate. Testing individual crates in isolation catches this.

## How conditional compilation works

The cfg attribute controls whether a piece of code is compiled. If the condition is false, the item is completely removed before the compiler processes it. Common conditions include: cfg of test (the code is only compiled when running tests), cfg of feature equals some-name (the code is only compiled when that feature is enabled), and cfg of target-os equals some-os (the code is only compiled for that operating system).

For starsight, conditional compilation gates backend modules. The terminal backend module is wrapped in cfg of feature equals terminal. When the terminal feature is not enabled, the entire module is skipped during compilation, and the ratatui, crossterm, and ratatui-image dependencies are not compiled. This keeps the default build fast and lightweight.

The cfg-attr attribute conditionally applies other attributes. For example, you might want to derive Serialize from the serde library only when the serde feature is enabled, without requiring serde as a default dependency.

## How macros generate code

Rust has two kinds of macros: declarative macros (defined with macro-rules) and procedural macros (defined as separate crates that manipulate token streams). starsight uses both.

Declarative macros use pattern matching on syntax. You define a macro with rules, where each rule has a pattern (the input) and an expansion (the output). The pattern uses fragment specifiers like expr (any expression), ident (an identifier), ty (a type), and tt (any single token tree). Repetition patterns use a dollar sign followed by the pattern in parentheses, a separator, and a repetition operator: asterisk for zero or more, plus for one or more.

For starsight, the plot macro has two forms. The simple form takes two expressions (x data and y data) and expands to code that creates a Figure, adds a LineMark with the data, and returns the Figure. The DataFrame form takes a reference to a DataFrame and named parameters for the x and y columns, and expands to code that extracts the columns and builds the appropriate marks.

Macros are hygienic in Rust: local variables inside the macro expansion get their own syntax context and do not conflict with variables at the call site. This prevents a common class of bugs where macro expansion accidentally captures or shadows a user's variable.

Procedural macros are more powerful. They are implemented as Rust functions that take a token stream and return a token stream. The derive macros used throughout starsight (like thiserror's derive Error) are procedural macros. starsight plans to use a procedural macro for the recipe attribute (which lets users define custom chart types), but this is a later-milestone feature.

## What a pixel is and how images are stored in memory

A pixel is a point sample of color at a specific location in a grid. Think of a digital image as a two-dimensional grid of color values, like graph paper where each square has been colored in. An 800 by 600 image has 800 columns and 600 rows, for a total of 480,000 pixels. Each pixel stores a color value, typically as three numbers (red, green, blue) or four numbers (red, green, blue, alpha).

The memory layout of an image is a flat array of bytes. For a standard RGBA image with 8 bits per channel, each pixel is 4 bytes: one byte for red, one for green, one for blue, one for alpha. The pixels are stored in row-major order: all pixels of the first row, then all pixels of the second row, and so on. A row of pixels is called a scanline. To find the byte offset of a specific pixel at column x and row y in an image of width w, the formula is: offset equals (y times w plus x) times 4.

The alpha channel represents transparency. An alpha value of 255 means fully opaque: the pixel completely covers whatever is behind it. An alpha of 0 means fully transparent: the pixel is invisible. Values between 0 and 255 represent partial transparency, which is how anti-aliasing, translucent overlays, and smooth edges work.

tiny-skia stores its pixel data as premultiplied alpha. This is a crucial concept that affects how colors are stored and blended. In straight alpha (also called unassociated alpha), the red, green, and blue channels store the pure color, and the alpha channel is separate. A semi-transparent red pixel with 50 percent opacity stores red equals 255, green equals 0, blue equals 0, alpha equals 128.

In premultiplied alpha (also called associated alpha), each color channel is multiplied by the alpha value before storage. The same semi-transparent red pixel stores red equals 128, green equals 0, blue equals 0, alpha equals 128. The red channel was multiplied by 128 divided by 255 (approximately 0.5).

Why use premultiplied alpha? Because the compositing formula becomes much simpler and faster. Compositing is the operation of placing one image on top of another. The standard "source over" compositing formula for premultiplied alpha is: result equals source plus destination times (1 minus source alpha). This is three multiplications and three additions per pixel. The equivalent formula for straight alpha requires seven multiplications, three additions, and a division, plus a divide-by-zero check when alpha is zero.

Premultiplied alpha also handles transparency correctly during interpolation. If you linearly interpolate between a transparent red pixel and a transparent blue pixel in straight alpha, the midpoint has color (128, 0, 128) with alpha 128. But this intermediate purple has full color intensity, which creates a visible purple halo around transparent edges. In premultiplied alpha, the interpolation produces the correct result without halos.

The downside of premultiplied alpha is that you lose color precision for semi-transparent pixels. A pixel with alpha equals 1 (almost fully transparent) can only store red values of 0 or 1, because the premultiplied red value must be less than or equal to the alpha value. This precision loss is negligible in practice because nearly-transparent pixels are nearly invisible.

When tiny-skia saves a Pixmap to PNG, it automatically converts from premultiplied alpha to straight alpha, because the PNG format uses straight alpha. You do not need to do this conversion yourself. When reading pixels from a Pixmap for testing or debugging, remember that the values are premultiplied.

## How lines are drawn on a pixel grid

Drawing a line on a pixel grid is called rasterization. The challenge is that a mathematical line is infinitely thin and can go at any angle, but pixels are discrete points on a grid. The line must be approximated by choosing which pixels to "turn on."

The classic algorithm for line rasterization is Bresenham's line algorithm, developed in 1962. For a line that is more horizontal than vertical (meaning the absolute change in x is greater than the absolute change in y), the algorithm steps one pixel at a time in the x direction and decides whether to step up or stay at the same y position. It makes this decision using an error term that tracks the accumulated difference between the true line and the approximated line.

At each step, the algorithm adds the slope (dy divided by dx) to the error term. If the error exceeds 0.5, it means the true line has moved more than half a pixel in the y direction, so the algorithm steps up (or down) and subtracts 1 from the error. The clever part of Bresenham's algorithm is that it uses only integer arithmetic by multiplying everything by 2 times dx, avoiding floating-point division entirely.

For starsight, you do not need to implement Bresenham's algorithm yourself. tiny-skia handles all rasterization internally. But understanding the concept helps you reason about anti-aliasing and pixel precision.

Anti-aliasing addresses the "jagged staircase" appearance of diagonal lines on a pixel grid. Instead of turning each pixel fully on or fully off, anti-aliasing computes the fraction of the pixel that is covered by the line and uses that fraction as the pixel's intensity. A pixel that is half-covered by the line gets half the line's color intensity. This creates a smooth gradient at the edges of the line, making it appear smoother to the human eye.

tiny-skia supports anti-aliasing through the anti_alias field on the Paint struct. When anti_alias is true (the default), the rasterizer computes per-pixel coverage. When anti_alias is false, each pixel is either fully inside or fully outside the shape, producing crisp but jagged edges.

For chart rendering, anti-aliasing should be on for diagonal lines, curves, and circles (where jagged edges are visible and distracting). It should be off for horizontal and vertical lines (like axis lines and grid lines), because anti-aliasing a perfectly aligned line makes it appear blurry instead of crisp. The anti-aliasing algorithm spreads the line across two pixels instead of keeping it on one, which reduces contrast.

## How curves work in computer graphics

Straight lines are simple, but charts need curves: smooth distribution lines, area chart boundaries, pie chart arcs, and more. Computer graphics uses a mathematical tool called Bezier curves (named after French engineer Pierre Bezier, who used them to design car body shapes at Renault in the 1960s).

A Bezier curve is defined by a set of control points. The curve starts at the first control point, ends at the last control point, and is "pulled toward" the intermediate control points without necessarily passing through them.

A linear Bezier curve has two control points and is just a straight line segment. The formula is: P of t equals (1 minus t) times P0 plus t times P1, where t goes from 0 to 1. At t equals 0, the formula gives P0 (the start). At t equals 1, it gives P1 (the end). At t equals 0.5, it gives the midpoint.

A quadratic Bezier curve has three control points: a start, a control, and an end. The formula is: P of t equals (1 minus t) squared times P0 plus 2 times (1 minus t) times t times P1 plus t squared times P2. The curve starts at P0, ends at P2, and is pulled toward P1 without passing through it. Quadratic Beziers can represent arcs and simple curves but cannot represent S-shapes (inflection points).

A cubic Bezier curve has four control points: a start, two controls, and an end. The formula involves third-degree polynomials. The curve starts at P0, ends at P3, and the tangent (direction) at the start points from P0 toward P1, while the tangent at the end points from P2 toward P3. Cubic Beziers are the standard curve type in computer graphics because they can represent S-shapes, sharp turns, and smooth arcs with four points.

The de Casteljau algorithm evaluates a Bezier curve at any parameter t through recursive linear interpolation. For a cubic curve with four control points, you first linearly interpolate between adjacent pairs (P0 to P1, P1 to P2, P2 to P3) at parameter t, producing three intermediate points. Then you interpolate between those three points to get two points. Then you interpolate between those two to get the final point on the curve. This algorithm is numerically stable and is how most renderers evaluate Bezier curves internally.

In SVG and tiny-skia, paths are constructed using commands: MoveTo (start a new sub-path at a position), LineTo (draw a straight line to a position), QuadTo (draw a quadratic Bezier to a position with one control point), CubicTo (draw a cubic Bezier to a position with two control points), and Close (draw a straight line back to the start of the sub-path).

For starsight, paths are the fundamental drawing primitive. A line chart is a path consisting of a MoveTo at the first data point and LineTo commands at each subsequent data point. A filled area chart is a closed path: the data points form the top, then a line drops to the baseline, runs along the baseline, and closes. A circle (for scatter plot points) can be approximated by four cubic Bezier curves. tiny-skia's PathBuilder has a push_circle method that does this automatically.

## How paths are filled and stroked

Once you have a path (a sequence of MoveTo, LineTo, and curve commands), you can fill it, stroke it, or both.

Filling a path means coloring the interior region. But what is "the interior" of a self-intersecting or complex path? Two fill rules provide different answers.

The non-zero winding rule (the default in SVG and tiny-skia) counts path crossings. Imagine standing at a point and casting a ray in any direction. Every time the path crosses the ray from left to right, add 1. Every time it crosses from right to left, subtract 1. If the total (the winding number) is non-zero, the point is inside. If it is zero, the point is outside. For a simple non-crossing path, this always fills the interior. For a pentagram drawn with one continuous stroke, the winding rule fills the entire shape including the center.

The even-odd rule simply counts how many times the ray crosses the path, regardless of direction. An odd count means inside, even means outside. For the same pentagram, the even-odd rule leaves the center unfilled, creating a star with a hole.

Stroking a path means drawing a line of a specified width along the path. The stroke is not the path itself but a filled region centered on the path. A path stroked with width 4 produces a filled band 2 units wide on each side of the path's mathematical centerline.

Stroke properties include the line cap, the line join, and the dash pattern. The line cap determines what happens at the open ends of a path. Butt cap ends the stroke exactly at the endpoint (the default). Round cap adds a semicircle at the endpoint. Square cap extends the stroke by half the width beyond the endpoint.

The line join determines what happens where two path segments meet at a corner. Miter join extends the outer edges of the stroke until they intersect, creating a sharp point (the default). Round join adds a circular arc at the corner. Bevel join cuts the corner flat. The miter limit controls how far the miter join can extend: if the angle is very acute, the miter becomes very long, and the miter limit caps it by falling back to a bevel.

The dash pattern specifies alternating lengths of visible and invisible stroke. A pattern of 10 comma 5 means 10 units of visible stroke, then 5 units of gap, repeating. A dash offset shifts the starting position of the pattern.

For starsight, fill is used for bar charts, area charts, pie charts, and scatter plot circles. Stroke is used for line charts, axis lines, tick marks, and borders. Some elements use both: a bar might be filled with a color and stroked with a darker border.

## How coordinate systems work in two-dimensional graphics

A coordinate system maps numeric positions to locations on the screen or image. Understanding coordinate systems is essential because starsight must convert between several of them.

Screen coordinates (also called pixel coordinates or device coordinates) place the origin at the top-left corner of the image. The x axis increases to the right. The y axis increases downward. This convention comes from CRT monitors and teletypes, where the electron beam or print head moved from left to right and from top to bottom. All modern graphics APIs, including tiny-skia, use this convention.

Mathematical coordinates (also called Cartesian coordinates) place the origin at the bottom-left, with the y axis increasing upward. This is what you learned in school: positive y is up. Scientific data is naturally expressed in mathematical coordinates: a temperature of 20 degrees at time 10 is "above" a temperature of 10 at the same time.

Chart coordinates are a hybrid. The plot area (where data is drawn) uses a coordinate system where data values map to pixel positions. The x axis maps data values to horizontal pixel positions (increasing rightward, same as screen coordinates). The y axis maps data values to vertical pixel positions, but inverted: higher data values correspond to pixel positions closer to the top of the image (lower y in screen coordinates). This inversion is the single most common source of bugs in chart rendering code. If your chart appears upside down, you forgot to invert the y axis.

An affine transform is a mathematical operation that converts between coordinate systems. It can translate (shift), scale (resize), rotate, and skew. An affine transform is represented by six numbers (a 2-by-3 matrix or a 3-by-3 matrix with the bottom row fixed as 0 comma 0 comma 1). Applying the transform to a point (x, y) produces a new point (x prime, y prime) computed as: x prime equals a times x plus b times y plus c, y prime equals d times x plus e times y plus f. Where a through f are the six transform values.

tiny-skia's Transform type represents an affine transform. It has methods for creating common transforms: identity (no change), from_translate (shift by dx and dy), from_scale (multiply by sx and sy), and from_rotate (rotate by an angle). The critical detail: tiny-skia's from_rotate method takes the angle in degrees, not radians. This is different from almost every other math library, which uses radians. If you pass pi divided by two (1.5708) expecting a 90-degree rotation, you will get a 1.5708-degree rotation, which is almost no rotation at all. Always use degrees with tiny-skia transforms.

Transforms compose by multiplication. If you apply transform A and then transform B, the combined transform is B times A (applied right to left). tiny-skia provides pre_ methods (apply before the existing transform) and post_ methods (apply after). For chart rendering, you typically start with the identity transform and apply translations and scales to map from data coordinates to pixel coordinates.

## How color works in computer graphics

Color is the perception of different wavelengths of light by the human eye. The visible spectrum ranges from about 380 nanometers (violet) to about 780 nanometers (red). The human eye has three types of color receptors (cone cells) in the retina, each sensitive to a different range of wavelengths: roughly short (blue-ish), medium (green-ish), and long (red-ish). This is why three primary colors (red, green, blue) are sufficient to reproduce a wide range of perceived colors by stimulating the three cone types in different proportions.

In the RGB color model, each color is specified by three values: the intensity of red, green, and blue light. In an 8-bit-per-channel system, each value ranges from 0 (none) to 255 (maximum). Pure red is (255, 0, 0). Pure green is (0, 255, 0). Pure blue is (0, 0, 255). White is (255, 255, 255). Black is (0, 0, 0). Yellow is (255, 255, 0) because yellow light stimulates both the red and green cones.

The sRGB color space (standardized as IEC 61966-2-1:1999) is the standard color space for the web, most monitors, and most image formats. It defines a specific set of red, green, and blue primary colors, a white point (D65, which approximates daylight), and a transfer function (often called a gamma curve) that maps between linear light intensity and the stored value.

The transfer function exists because human vision is non-linear: we are much more sensitive to differences in dark colors than in bright colors. If you use 256 equally-spaced levels of linear light intensity, most of the levels would be spent on bright values that look nearly identical, while the dark values would have visible banding. The sRGB transfer function compresses the bright end and expands the dark end, matching the levels to human perception.

The sRGB to linear conversion formula is: for values less than or equal to 0.04045, divide by 12.92. For values above 0.04045, the formula is ((value plus 0.055) divided by 1.055) raised to the power 2.4. The reverse formula (linear to sRGB) is: for linear values less than or equal to 0.0031308, multiply by 12.92. For values above 0.0031308, the formula is 1.055 times the value raised to the power (1 divided by 2.4) minus 0.055.

Why does this matter for starsight? Because color blending (computing the intermediate color between two colors) should ideally happen in linear space, not in sRGB space. If you linearly interpolate between dark red (128, 0, 0) and dark blue (0, 0, 128) in sRGB space, the midpoint is (64, 0, 64), which is a dark purple that appears too dark. This is because averaging the sRGB-encoded values is not the same as averaging the light intensities. The correct approach is to convert both colors to linear space, interpolate, and convert back to sRGB.

However, most visualization libraries (including matplotlib, D3, and Plotly) perform interpolation in sRGB space because the visual difference is small for typical chart colors and the computational cost of conversion is significant. starsight follows this convention for 0.1.0: the Color lerp method interpolates in sRGB space. Perceptually correct interpolation via the Oklab color space can be added later as an option.

The WCAG (Web Content Accessibility Guidelines) contrast ratio formula uses relative luminance, which requires the linear conversion. The formula for relative luminance is: 0.2126 times R linear plus 0.7152 times G linear plus 0.0722 times B linear. The contrast ratio between two colors is: (the brighter luminance plus 0.05) divided by (the darker luminance plus 0.05). The WCAG requires a contrast ratio of at least 4.5 to 1 for normal text readability and 3 to 1 for large text.

For starsight, the luminance and contrast_ratio methods on Color perform this calculation. They are used to ensure that chart text is readable against the background color.

## How PNG encoding works

When starsight saves a chart to a PNG file, the pixel data goes through a compression pipeline. Understanding this at a high level helps you understand why PNG files vary in size and why some charts compress better than others.

PNG uses a two-stage compression process. The first stage is filtering, which does not compress the data but transforms it to be more compressible. Each row of pixels (scanline) is independently filtered using one of five filter types. The None filter passes the bytes unchanged. The Sub filter stores the difference between each byte and the corresponding byte from the pixel to the left. The Up filter stores the difference from the pixel directly above. The Average filter uses the average of the left and above neighbors. The Paeth filter uses a predictor that selects whichever of left, above, or upper-left is closest to a linear prediction.

The purpose of filtering is to transform the pixel data so that most values are near zero. Smooth gradients produce sequences of small differences, which compress much better than sequences of large absolute values. The encoder tries each filter for each row and picks the one that produces the most compressible output (typically measured by the sum of absolute values).

The second stage is Deflate compression (the same algorithm used in ZIP files and gzip). Deflate combines two techniques: LZ77 (finding repeated byte sequences within a 32-kilobyte window and replacing them with back-references) and Huffman coding (replacing fixed-length symbols with variable-length codes based on frequency). The filtering stage creates many repeated small values, which LZ77 and Huffman coding exploit efficiently.

Charts with large solid-color regions (like bar chart fills and white backgrounds) compress very well because the filtering produces long runs of zeros. Charts with many colors and fine detail (like dense scatter plots with anti-aliased edges) compress less well because there are fewer repeated patterns.

## How text rendering works from character to pixel

Text rendering is one of the most complex parts of any graphics library. The pipeline from a text string to pixels on screen involves four stages: font selection, text shaping, text layout, and glyph rasterization.

Font selection starts with a request: "14 pixel sans-serif regular." The font system (cosmic-text's FontSystem in starsight) searches its database of installed fonts to find the best match. On Linux, it queries the fontconfig database. On macOS, it uses Core Text. On Windows, it reads the font registry. The result is a specific font file (like DejaVu Sans Regular or Arial) at the requested size.

Text shaping converts a string of Unicode characters into a sequence of positioned glyphs. A glyph is the visual representation of a character (or group of characters) in a specific font. Shaping is more complex than just looking up each character in a font table because many scripts require context-dependent glyph selection.

For example, in Arabic, each letter has up to four different forms depending on whether it appears at the beginning, middle, or end of a word, or stands alone. In many Latin fonts, the letter pair "fi" has a special combined glyph (a ligature) that looks better than two separate glyphs placed next to each other. Kerning adjusts the spacing between specific letter pairs: the letters "AV" are often placed closer together than the default spacing because the diagonal strokes of A and V mesh naturally.

The shaping engine (harfrust in cosmic-text, which is a Rust port of the HarfBuzz library) handles all of this. It reads the font's OpenType tables to determine which glyphs to use and how to position them. For starsight's typical use case (chart labels with digits and Latin letters), shaping is relatively simple. But it still matters because kerning affects the width of text, and text width determines how much space to allocate for axis labels and titles.

Text layout arranges the shaped glyphs into lines. For chart tick labels, layout is trivial because each label is a single short string that fits on one line. For multi-line titles or wrapped annotations, the layout engine decides where to break lines based on the available width, the Unicode line breaking rules, and the word wrapping mode.

cosmic-text's Buffer handles layout. You create a Buffer with Metrics (font size and line height in pixels), set the text and attributes, optionally set a maximum width, and call shape_until_scroll to perform shaping and layout. After this, you can query the layout runs (one per visible line) to get the text dimensions, and call the draw method to rasterize the glyphs.

Glyph rasterization converts the mathematical outlines of each glyph into pixel coverage values. Each glyph is a set of Bezier curves (quadratic for TrueType fonts, cubic for OpenType CFF fonts). The rasterizer evaluates these curves at the resolution of the target image and computes, for each pixel, how much of the pixel area is covered by the glyph outline. A pixel fully inside the glyph gets coverage 255. A pixel fully outside gets 0. A pixel on the edge gets a value proportional to the covered area.

cosmic-text uses the swash library for rasterization. Rasterized glyphs are cached (by the SwashCache type) so that the same glyph at the same size is only rasterized once, regardless of how many times it appears in the text.

The final step is compositing: painting the rasterized glyph onto the pixel buffer. For each pixel of the glyph, the text color is blended with the existing background using the glyph's coverage as the alpha value. This is where the Pixmap and the cosmic-text callback connect: the callback delivers each glyph pixel, and the closure writes it into the Pixmap using the appropriate blending formula.

A critical detail for starsight: cosmic-text and tiny-skia use different Color types. cosmic-text's Color has r, g, b, a as u8 values. tiny-skia's Color has r, g, b, a as f32 values. When painting glyph pixels onto a tiny-skia Pixmap, you receive cosmic-text Color values in the callback and need to set them on a tiny-skia Paint. Use the set_color_rgba8 method on Paint, which takes u8 values and does the conversion internally.

There is a persistent myth that you need to swap the red and blue channels when compositing cosmic-text glyphs onto a tiny-skia Pixmap. This is not true for PNG and SVG output. The channel swap exists in some cosmic-text example code because those examples render to a softbuffer (a platform-native framebuffer), which may use a different byte order (BGRA instead of RGBA). For file output, pass the channels straight through.


### Sequential versus diverging versus qualitative palettes

Not all colormaps serve the same purpose. The type of data determines which kind of colormap is appropriate.

Sequential palettes vary from light to dark (or vice versa) and are used for data that has a natural ordering from low to high. Temperature, elevation, population density, and concentration are all sequential data. Sequential palettes include viridis, inferno, plasma, and the Crameri scientific colormaps. The key property is monotonically increasing lightness: you can always tell which values are higher because they are darker (or lighter, depending on the palette).

Diverging palettes have two hues that diverge from a neutral center. They are used for data with a meaningful midpoint: anomalies (above or below average), residuals (positive or negative), correlation coefficients (from minus one to plus one). The center is usually white or light gray, and the two extremes are distinct colors (like blue and red). The classic example is a temperature anomaly map where blue means colder than average and red means warmer than average.

Qualitative palettes use distinct, unrelated colors for categorical data: labels, classes, groups. The colors should be maximally distinguishable but should not imply any ordering. Red, blue, green, orange does not suggest that red is "more than" blue. Qualitative palettes include Tableau10, Set2, and Paired. The challenge is finding enough distinguishable colors: beyond about 8 to 12 categories, qualitative palettes become unreliable and you should use other encoding channels (shape, pattern, label) instead.

For starsight, prismatica provides all three types. Sequential colormaps are accessed through modules like crameri, cet, and matplotlib. Diverging colormaps are in the same modules with names like berlin, vik, and cool_warm. Qualitative palettes are discrete, accessed through the palette module with methods like get and iter.

---

## Chapter Thirteen: How text becomes visible on screen

Text rendering is the hardest part of 2D graphics. It involves at least four separate systems working in sequence: a font database, a shaping engine, a layout engine, and a rasterizer. Each system is complex on its own, and their interactions add further complexity.

### What a font file contains

A font file (TrueType or OpenType format) is a structured binary database. It contains: a table of glyph outlines (the mathematical curves that define each letter's shape), a character-to-glyph mapping table (which maps Unicode code points to glyph indices), metric tables (which define how wide each glyph is and how much space to leave between lines), and feature tables (which describe ligatures, kerning pairs, and contextual alternatives).

The glyph outlines are defined in a coordinate system called "font design units" or "em units." The em size is typically 1000 or 2048 units. To convert to pixels, you multiply by the font size and divide by the em size. A glyph that is 600 em units wide at a 14-pixel font size with 1000 em units is 8.4 pixels wide.

### Shaping: from characters to glyphs

Shaping is the process of converting a sequence of Unicode characters into a sequence of positioned glyphs. This is more complex than a simple character-to-glyph lookup for several reasons.

Ligatures combine multiple characters into a single glyph. The character sequence f followed by i might become the fi ligature glyph, which looks different from an f next to an i. Whether this happens depends on the font and the script.

Kerning adjusts the spacing between specific letter pairs. The pair AV typically has negative kerning: the V tucks under the right side of the A, reducing the visual gap. Without kerning, the space between A and V looks too large.

Contextual shaping is essential for scripts like Arabic, which has four forms for each letter (isolated, initial, medial, and final), and Devanagari, which requires complex reordering and mark attachment.

For chart text (axis labels, titles, tick labels), shaping is usually simple because chart text consists of digits, Latin letters, and a few symbols. But it must still run because even Latin text benefits from kerning and ligature handling.

In starsight, cosmic-text handles shaping via its internal harfrust engine (a pure Rust port of HarfBuzz). You create a Buffer, set the text with attributes (font family, size, weight), and call shape_until_scroll. After shaping, the buffer contains positioned glyph runs that can be iterated and rendered.

### Layout: from glyph runs to lines

Layout determines where each line of text goes. For single-line chart labels, layout is trivial: the text starts at a given position and extends horizontally. For multi-line titles or wrapped annotations, layout involves line breaking (deciding where to break lines) and vertical positioning (computing the Y offset of each line based on the line height).

cosmic-text's Buffer manages layout. After shaping, you call layout_runs to iterate over the visible lines. Each layout run has a line_w field (the width of the line in pixels), a line_y field (the Y offset from the top of the buffer), and a line_height field (the height of the line). These values are what you need to position text on the chart.

For measuring text (which you need for computing axis margins), iterate the layout runs and find the maximum line_w (the total text width) and the total height (the last line's Y offset plus its line height).

### Rasterization: from outlines to pixels

Glyph rasterization converts the mathematical glyph outlines into pixel coverage values. Each pixel gets a value from 0 (outside the glyph) to 255 (inside the glyph). These coverage values become the alpha channel when compositing the glyph onto the pixel buffer.

In cosmic-text, rasterization is handled by the swash library through the SwashCache type. The cache stores rasterized glyph images indexed by font, glyph index, size, and sub-pixel position (glyphs are positioned at sub-pixel accuracy for better visual quality, with positions quantized to a 4-by-4 grid).

The draw method on Buffer calls a callback for each pixel of each rasterized glyph. The callback receives x, y, width, height, and color. For each callback invocation, you write the pixel into the tiny-skia Pixmap. The key detail: do not swap the red and blue channels. There is a persistent myth that you need to swap them because cosmic-text and tiny-skia use different byte orders. This myth comes from example code that renders to softbuffer (which uses a different byte order for display output). For PNG and SVG output, pass the channels straight through.

### Why font loading is slow and must be done once

cosmic-text's FontSystem constructor scans the operating system's font directories, reads metadata from every installed font, and builds a database for font matching. On a typical system with hundreds of installed fonts, this takes about one second in release mode and up to ten seconds in debug mode.

This means you must create the FontSystem once when the rendering backend initializes and reuse it for all text rendering operations. Creating a new FontSystem for each draw call would add seconds of overhead per chart.

For deterministic testing, you can create a FontSystem with embedded fonts rather than system fonts. Load a font file from disk or embed it in the binary, pass it to the FontSystem constructor, and all text will render using that specific font. This eliminates variation across operating systems and produces identical output on every machine.

---

## Chapter Fourteen: The grammar of graphics

The grammar of graphics is a theoretical framework that decomposes every chart into a small set of independent, composable components. Understanding it is essential for designing a visualization library that can produce any chart type from a single unified API.

### The core insight

The key idea is that a scatter plot, a line chart, a bar chart, and a histogram are not fundamentally different things. They are all combinations of the same building blocks: data, aesthetic mappings, geometric marks, statistical transforms, position adjustments, scales, coordinate systems, and facets. A scatter plot is a point mark with x and y aesthetic mappings. A histogram is a bar mark with a binning statistical transform. A pie chart is a bar mark in polar coordinates.

This decomposition means you do not need a separate function for every chart type. You need a small set of marks (point, line, bar, area, arc, text, rect), a small set of stats (bin, kde, aggregate, regression, boxplot), a small set of scales (linear, log, categorical, color, size), and a composition system that lets the user combine them.

### Aesthetic mappings connect data to visual properties

An aesthetic mapping says "this column of data should control this visual property." If you map x to temperature, the horizontal position of each mark is determined by the temperature value. If you map color to species, the color of each mark is determined by the species label.

The standard aesthetics are: x (horizontal position), y (vertical position), color or fill (the color of the mark), size (the area or radius of the mark), shape (the symbol used for point marks), alpha (transparency), and label (text displayed at each data point).

Each aesthetic has an associated scale that converts data values to visual values. The x aesthetic has a positional scale that maps temperature values to pixel positions. The color aesthetic has a color scale that maps species labels to specific colors. The size aesthetic has a size scale that maps population values to point radii.

### Geometric marks are the visual shapes

A mark is the visual shape drawn for each data point. The most common marks are:

Point mark: a dot, circle, or symbol at each data point. Used for scatter plots. The position comes from x and y aesthetics, the color from the color aesthetic, the size from the size aesthetic.

Line mark: a continuous line connecting data points in order. Used for line charts and time series. NaN values in the data break the line, creating gaps.

Bar mark: a rectangle from a baseline (usually zero) to the data value. Used for bar charts and histograms. The width comes from the band scale, the height from the y aesthetic.

Area mark: a filled region between a line and a baseline. Used for area charts and stacked area charts. Equivalent to a line mark with the region below it filled in.

Arc mark: a sector of a circle. Used for pie charts and donut charts. The angular extent comes from the data value, the radius is fixed or mapped to another aesthetic.

Text mark: a label at each data point. Used for annotations and direct labeling.

Rect mark: a rectangle defined by four edges. Used for heatmaps and tile plots.

Each mark type knows how to render itself given a coordinate system and a backend. The render method takes the data (already mapped through the aesthetic bindings), the coordinate system (which converts data values to pixel positions), and the backend (which draws paths and text onto the pixel buffer).

### Statistical transforms preprocess data

A stat transform runs before the mark renders. It takes the raw data and produces derived data. The most common stats are:

Bin: divides continuous data into bins and counts the number of values in each bin. This is the stat behind histograms. The input is a series of continuous values. The output is a series of bin centers and counts.

KDE (kernel density estimation): estimates a smooth probability density curve from discrete data points. Used for violin plots and density plots. The input is a series of values. The output is a smooth curve of x and y values.

Aggregate: computes summary statistics (mean, median, sum, count) for groups of data. Used for bar charts that show group means.

Regression: fits a line or curve to data. Linear regression fits a straight line. LOESS fits a smooth local regression.

Boxplot: computes the five-number summary (minimum, Q1, median, Q3, maximum) and identifies outliers. Used for box plots.

The stat and the mark are independent. You can pair any stat with any mark. A histogram is bin stat plus bar mark. But you could also pair bin stat with point mark to show bin counts as dots instead of bars. Or you could pair kde stat with area mark to show the density as a filled region. This combinatorial flexibility is the power of the grammar.

### Position adjustments handle overlap

When multiple marks occupy the same position, they need to be adjusted to avoid overlap. Position adjustments include:

Identity: no adjustment. Marks can overlap. This is the default.

Dodge: marks are placed side by side. Used for grouped bar charts where multiple series have bars at the same x position.

Stack: marks are placed on top of each other. Used for stacked bar charts and stacked area charts. Each mark starts where the previous one ended.

Jitter: marks are displaced by a small random amount. Used for scatter plots where many points share the same or similar coordinates, to make all points visible.

Fill: like stack, but normalized to 100 percent. Each group's marks fill the entire y range, showing proportions rather than absolute values.

### Coordinate systems interpret positions

The default coordinate system is Cartesian: x goes right, y goes up (inverted to screen coordinates during rendering). But other systems exist.

Polar coordinates interpret x as angle and y as radius. A bar chart in polar coordinates becomes a pie chart (or a rose chart). A line chart in polar coordinates becomes a radar chart.

Geographic coordinates interpret x as longitude and y as latitude, applying a map projection to convert from the curved earth surface to a flat display. The simplest projection is equirectangular (longitude and latitude map directly to x and y), but it distorts areas near the poles.

Flipped coordinates swap x and y, turning vertical bar charts into horizontal bar charts. This is purely a coordinate transformation, not a change to the mark or the data.

### Faceting creates small multiples

Faceting splits the data by a categorical variable and creates one chart panel for each value. If your data has a column called "region" with values "North," "South," "East," and "West," faceting on region creates four identical charts, each showing only the data for one region.

Facet wrap arranges the panels in a single row that wraps to multiple rows when it exceeds the available width. You specify the number of columns.

Facet grid uses two variables: one for rows and one for columns, creating a matrix of panels. This is useful for exploring interactions between two categorical variables.

The key design choice is whether the axes are shared (all panels use the same axis range, enabling direct comparison) or free (each panel zooms to its own data range, revealing local patterns). This is a user-configurable option.

---

## Chapter Fifteen: Scales and how data maps to visual space

A scale is a function that converts a data value into a visual value. The most common use is converting data coordinates into pixel positions, but scales also map data values to colors, sizes, and shapes.

### Linear scales

The simplest and most common scale. The formula is: output equals (input minus domain minimum) divided by (domain maximum minus domain minimum) times (range maximum minus range minimum) plus range minimum. This normalizes the input to a zero-to-one range, then stretches it to the output range.

For example: the data range is 0 to 100 (domain minimum equals 0, domain maximum equals 100). The pixel range is 50 to 750 (range minimum equals 50, range maximum equals 750). The data value 25 maps to: (25 minus 0) divided by (100 minus 0) times (750 minus 50) plus 50 equals 0.25 times 700 plus 50 equals 225. The data value 25 is at pixel 225.

The inverse scale converts back from pixel to data value: data equals (pixel minus range minimum) divided by (range maximum minus range minimum) times (domain maximum minus domain minimum) plus domain minimum. This is used for interactive charts where you need to convert a mouse click position back to a data value.

### Logarithmic scales

A logarithmic scale takes the logarithm of the data value before applying the linear mapping. This compresses large values and expands small values, making it possible to visualize data that spans many orders of magnitude.

The formula is: normalized equals (log of input minus log of domain minimum) divided by (log of domain maximum minus log of domain minimum). Then apply the linear mapping as before: output equals normalized times range extent plus range minimum.

Logarithmic scales require strictly positive data. The logarithm of zero is negative infinity, and the logarithm of a negative number is undefined for real numbers. If your data contains zero or negative values, use a symmetric logarithmic (symlog) scale instead.

### Symmetric logarithmic scales

The symlog scale handles data that spans zero. Near zero, it uses a linear scale (to avoid the logarithmic singularity). Far from zero, it uses a logarithmic scale (to compress the wide range). The transition between linear and logarithmic is controlled by a threshold parameter.

The formula is: if the absolute value of x is less than or equal to the threshold C, the output is x divided by C (linear). If the absolute value of x is greater than C, the output is the sign of x times (1 plus the log of the absolute value of x divided by C) (logarithmic). The factor of 1 ensures the function is continuous at the transition point.

Symlog scales are useful for financial data (which can be positive or negative with extreme values), scientific residuals (which scatter around zero with occasional large deviations), and any measurement that can be positive, negative, and near zero.

### Categorical scales

A categorical scale maps discrete labels to positions. The labels "apple," "banana," "cherry" might map to positions 100, 300, 500. There is no interpolation between positions: you are either at "apple" or at "banana," never in between.

A band scale extends the categorical scale with the concept of width. Each category occupies a band of pixels. The bar mark uses the band width to determine how wide to draw each bar. The inner padding controls the gap between adjacent bands. The outer padding controls the gap at the edges.

### The Wilkinson Extended tick algorithm

One of the most important details in chart rendering is choosing where to place tick marks on an axis. Bad ticks (like 3.7, 19.4, 35.1, 50.8) look unprofessional and are hard to read. Good ticks (like 0, 10, 20, 30, 40, 50) are round numbers that are easy to read and compare.

The Wilkinson Extended algorithm, published in 2010 by Talbot, Lin, and Hanrahan, finds optimal tick positions by searching over a space of candidates and scoring them on four criteria.

Simplicity measures how round the tick values are. Ticks at multiples of 1 are simpler than multiples of 5, which are simpler than multiples of 2. The algorithm uses a preference list Q equals 1, 5, 2, 2.5, 4, 3, ordered from most preferred to least preferred. The skip factor j multiplies the base: skip 2 with base 5 gives a step of 10.

Coverage measures how well the tick range covers the data range. If the data goes from 3.7 to 97.2 and the ticks go from 0 to 100, the coverage is good. If the ticks go from 0 to 200, the coverage is poor (wasted space). Coverage penalizes both insufficient and excessive coverage using a squared-error formula.

Density measures how close the number of ticks is to the target. If the user wants about 5 ticks and the algorithm generates 6, the density is good. If it generates 12, the density is poor. Density has the highest weight (0.5) in the scoring formula because the number of ticks most directly affects readability.

Legibility evaluates formatting concerns like label overlap and orientation. In the standard implementation, it is simplified to a constant.

The overall score is a weighted sum: 0.2 times simplicity plus 0.25 times coverage plus 0.5 times density plus 0.05 times legibility. The algorithm searches over all combinations of step base, skip factor, number of ticks, and start position, pruning branches where the upper bound on the score falls below the current best. The average number of candidates evaluated is about 41, making the algorithm fast enough for real-time use.

No existing Rust crate implements this algorithm. starsight will be the first. The D3 JavaScript library uses a simpler algorithm with only three step bases. Plotters uses basic rounding. The Wilkinson Extended algorithm produces significantly better tick positions than either.

---

## Chapter Sixteen: Choosing the right chart for the data

Different data types call for different chart types. Using the wrong chart can mislead the reader or hide important patterns. This section covers the most common data situations and the charts that handle them well.

### Comparing quantities across categories

When you have a few categories (cities, products, departments) and a numeric value for each, use a bar chart. The length of each bar encodes the value, and length is the most accurately perceived visual encoding. Keep the axis starting at zero: truncating the axis exaggerates differences and is one of the most common forms of chart deception.

For many categories (more than about 15), use a horizontal bar chart. This gives each bar a readable label without rotation.

For comparing two related quantities (revenue and cost by department), use grouped bars: two bars side by side for each category.

For showing composition (each category's share of a total), use a stacked bar chart or a stacked area chart.

### Showing trends over time

Use a line chart. The x axis is time, the y axis is the measured value. Lines naturally suggest continuity and temporal connection. Multiple series can be plotted as separate colored lines.

Do not use a bar chart for time series unless the data represents cumulative or aggregated values (monthly totals, yearly counts). Bar charts imply discrete categories, while time is continuous.

### Showing distributions

For a single distribution, use a histogram (binned counts) or a density plot (smooth estimate). Histograms are easier to explain; density plots are smoother and easier to compare.

For comparing distributions across categories, use a box plot (compact summary: median, quartiles, outliers) or a violin plot (shows the full distribution shape). Violin plots are more informative but harder to read for audiences unfamiliar with them.

### Showing relationships between two variables

Use a scatter plot. Each data point is a dot positioned by its x and y values. Scatter plots reveal correlation, clusters, outliers, and nonlinear relationships. For large datasets (thousands of points), add transparency to handle overplotting, or use hexagonal binning to show density.

### Showing geographic data

Use a map with a color encoding. Choose a map projection appropriate for the region: Mercator for navigational context, equal-area for comparing regional values, equirectangular for simplicity.

### Showing hierarchical data

Use a treemap (nested rectangles), a sunburst chart (nested arcs), or a dendrogram (tree diagram). Treemaps are good for showing sizes. Dendrograms are good for showing structure.

---

## Chapter Seventeen: The tiny-skia rendering library in depth

tiny-skia is a pure Rust port of a subset of Skia, Google's 2D graphics library. It provides CPU-based rasterization of filled and stroked paths with solid colors, gradients, and patterns. It is the primary rendering backend for starsight.

### Creating a pixel buffer

The Pixmap type owns the pixel data. You create one by calling Pixmap new with a width and height. This allocates a buffer filled with transparent black. The method returns Option because it validates the dimensions: zero width, zero height, or dimensions that would overflow 32-bit multiplication return None.

For starsight, the backend constructor wraps this in a Result. If Pixmap new returns None, the constructor returns a StarsightError Render explaining that the dimensions are invalid.

The fill method on Pixmap fills the entire buffer with a single color. For chart rendering, the first step is always to fill the background: backend fill Color WHITE.

### Drawing shapes

To draw a filled rectangle, call fill_rect on the Pixmap (or PixmapMut) with a Rect, a Paint, a Transform, and an optional Mask. The Rect type in tiny-skia is defined by four f32 values: left, top, right, bottom. The constructor from_ltrb returns Option because it validates that left is less than right and top is less than bottom.

To draw a filled or stroked path, first build the path with PathBuilder. Call move_to, line_to, quad_to, cubic_to, and close as needed. Call finish to get the Path (returns None if the path is empty). Then call fill_path or stroke_path on the Pixmap with the path, a Paint, and either a FillRule (for fills) or a Stroke (for strokes).

The Paint struct controls the appearance. Its most important field is the shader, which determines the color. For solid colors, set it with set_color_rgba8 (taking four u8 values) or set_color (taking a tiny-skia Color, which uses f32 values). The anti_alias field controls whether anti-aliasing is applied.

The Transform parameter applies a geometric transformation to the path before rendering. Transform identity means no transformation. Transform from_translate shifts the path. Transform from_scale resizes it. Transform from_rotate rotates it. The critical gotcha: from_rotate takes degrees, not radians. If you pass the mathematical constant pi divided by two expecting a 90-degree rotation, you will get a 1.57-degree rotation instead.

### Saving output

To save the Pixmap as a PNG file, call save_png with a file path. To get the PNG data as bytes in memory, call encode_png, which returns a Vec of u8. The encoding handles the conversion from premultiplied alpha (the internal format) to straight alpha (the PNG format) automatically.

### The Mask type for clipping

A Mask is a grayscale image that controls which pixels can be drawn. White areas (value 255) allow drawing. Black areas (value 0) block drawing. Gray areas allow partial drawing (useful for anti-aliased clip edges).

For chart rendering, the mask defines the plot area. Any drawing that extends beyond the plot area (because data points are outside the axis range) is clipped by the mask. This keeps the chart clean without requiring the mark code to perform bounds checking.

You create a Mask with Mask new (which returns Option), then fill a rectangular area using fill_path or fill_rect. The mask is passed as the last parameter to every drawing method.

---

## Chapter Eighteen: The cosmic-text library for rendering text

cosmic-text is a Rust library for text shaping, layout, and rasterization. It handles everything from Unicode strings to positioned, rasterized glyphs.

### Initialization

The first step is creating a FontSystem. This scans the operating system's fonts and builds a database. It takes about one second in release mode. Create it once and store it in the rendering backend.

The second step is creating a SwashCache. This caches rasterized glyph images to avoid re-rasterizing the same glyph at the same size. Create it alongside the FontSystem and reuse it.

### Creating and shaping text

Create a Buffer with the FontSystem and a Metrics struct. The Metrics constructor takes two f32 values: font_size (the em size in pixels, for example 14.0 for body text) and line_height (the baseline-to-baseline distance in pixels, for example 20.0). If line height is zero, the constructor panics, so validate the value before passing it.

Set the text on the buffer by calling set_text with the FontSystem, the string, an Attrs struct (specifying font family, weight, style, and color), a Shaping mode (use Advanced for correct results), and an optional alignment.

Set the maximum width by calling set_size with the FontSystem and two Option of f32 values for width and height. Pass None for unbounded dimensions.

Call shape_until_scroll with the FontSystem and true to perform the shaping and layout.

### Measuring text

After shaping, iterate layout_runs on the buffer. Each run has a line_w field (width of the line in pixels) and a line_y field (Y position of the line). The total text width is the maximum line_w across all runs. The total text height is the last run's line_y plus its line_height.

This measurement is needed for computing chart margins: the left margin must be wide enough to fit the longest Y axis tick label, and the bottom margin must be tall enough for the X axis labels.

### Drawing text onto a Pixmap

Call buffer draw with the FontSystem, the SwashCache, a Color for the text, and a callback closure. The callback is called once per pixel of each rasterized glyph. The parameters are x (i32), y (i32), width (u32), height (u32), and color (cosmic-text Color).

Inside the callback, check that x and y are within the Pixmap bounds (they can be negative or beyond the edges). Convert the cosmic-text Color to tiny-skia format using set_color_rgba8 on a Paint. Fill the pixel rectangle on the Pixmap.

For starsight, this callback runs inside the SkiaBackend's draw_text method. The closure captures a mutable reference to the Pixmap and writes each glyph pixel directly.


## What the grammar of graphics is and why starsight uses it

The grammar of graphics is a theory of how to think about statistical charts. It was introduced by Leland Wilkinson in his 1999 book "The Grammar of Graphics." The core idea is that every chart is composed of independent, reusable components, and you can create any chart by combining these components in different ways.

Think of it like language. In English, you build sentences from subjects, verbs, and objects. "The dog chased the cat" and "the cat chased the dog" use the same words (components) but create different meanings (charts) by combining them differently. The grammar of graphics says every chart is built from data, aesthetic mappings, geometric marks, statistical transforms, position adjustments, scales, coordinate systems, and facets.

Data is the information you want to visualize. It is typically a table with columns of numbers or categories. A column might contain temperatures, dates, species names, or x-coordinates.

An aesthetic mapping connects a data column to a visual property. The mapping "x equals time" means the time column controls horizontal position. The mapping "y equals temperature" means the temperature column controls vertical position. The mapping "color equals species" means the species column controls the color of each data point. The mapping "size equals population" means the population column controls the size of each point. Aesthetic mappings are declarations: they describe the connection between data and visuals but do not compute anything.

A geometric mark is the visual shape drawn for each data point. A point mark draws a circle at each data position. A line mark connects consecutive data points with a line. A bar mark draws a rectangle from a baseline to the data value. An area mark fills the region between a line and the baseline. A text mark places text labels at data positions.

A statistical transform preprocesses data before the mark renders. The most common example is binning: dividing a continuous variable into intervals and counting how many data points fall into each interval. When you make a histogram, the data goes through a bin transform before reaching the bar mark. The input is a series of continuous values. The output is a series of bin centers and counts. The bar mark then draws one bar per bin, with the bar height equal to the count.

Other statistical transforms include kernel density estimation (which produces a smooth probability curve from discrete data), regression (which fits a line or curve to data), boxplot summary (which computes the five-number summary: min, Q1, median, Q3, max), and aggregation (which computes sum, mean, or other summaries per group).

A position adjustment handles overlapping marks. If you have a bar chart with two data series at the same x position, dodge places the bars side by side. Stack places them on top of each other (cumulative). Jitter adds random noise to prevent overplotting in scatter plots where many points share the same coordinates.

A scale maps data values to visual values. A linear scale maps data values to pixel positions proportionally. A log scale applies a logarithm first, compressing large values and expanding small values. A color scale maps data values to colors from a colormap.

A coordinate system defines the geometric space. Cartesian coordinates (the default) use perpendicular x and y axes. Polar coordinates use angle and radius, which is how pie charts and radar charts work. Geographic coordinates use latitude and longitude with a map projection.

Faceting splits data into subsets and creates a separate chart for each subset. If your data has a category column called "species" with values "setosa," "versicolor," and "virginica," faceting by species creates three charts side by side, each showing only the data for one species.

The power of this framework is composability. A histogram is not a special chart type. It is a bar mark applied to data that has been through a bin transform, displayed on Cartesian coordinates. A scatter plot is a point mark with x and y aesthetic mappings. A boxplot is a special boxplot mark with a boxplot summary transform. You do not need 66 separate chart implementations. You need about 15 marks, 10 transforms, 8 scales, 3 coordinate systems, and a composition system.

starsight implements this framework across its layers. Layer three contains the marks and transforms. Layer two contains the scales and coordinate systems. Layer four contains the faceting. Layer five contains the high-level API that assembles the pieces for common chart types, so users can write "plot of data" without manually composing marks and scales.

## How scales map data values to visual space

A scale is a function that converts a data value into a visual value. The input is a number from the data domain (like a temperature between 0 and 100 degrees). The output is a number in the visual range (like a pixel position between 50 and 750).

A linear scale is the simplest. The formula is: output equals (input minus domain_min) divided by (domain_max minus domain_min) times (range_max minus range_min) plus range_min. This is a two-step process. First, normalize the input to a fraction between 0 and 1 by dividing the distance from the domain minimum by the domain extent. Then, interpolate within the visual range by multiplying the fraction by the range extent and adding the range minimum.

For example, if the data domain is 0 to 100 and the visual range is 50 to 750 pixels, the value 50 maps to: (50 minus 0) divided by (100 minus 0) times (750 minus 50) plus 50, which equals 0.5 times 700 plus 50, which equals 400 pixels.

The inverse scale maps back from visual to data: input equals (output minus range_min) divided by (range_max minus range_min) times (domain_max minus domain_min) plus domain_min. This is needed for interactive charts: when the user clicks at pixel position 400, you need to know what data value that corresponds to.

A logarithmic scale applies a logarithm before the linear mapping. The formula is: output equals (log of input minus log of domain_min) divided by (log of domain_max minus log of domain_min) times range_extent plus range_min. This compresses large values and expands small values, making it possible to visualize data that spans several orders of magnitude.

Logarithmic scales have an important limitation: they cannot handle zero or negative values because the logarithm of zero is negative infinity and the logarithm of a negative number is undefined for real numbers. If your data includes zero, you need a different approach.

A symmetric logarithmic (symlog) scale handles data that crosses zero. It applies a logarithm to the absolute value and preserves the sign. Near zero, it uses a linear region to avoid the logarithmic singularity. The transition between linear and logarithmic is controlled by a threshold parameter. Below the threshold, the mapping is linear. Above the threshold, the mapping is logarithmic. This creates a smooth transition that works for data like stock returns (positive and negative), temperature anomalies, and other measurements that span zero.

A categorical scale maps discrete labels to evenly spaced positions. The labels "apple," "banana," "cherry" might map to pixel positions 100, 300, 500. There is no interpolation between positions because the data is discrete, not continuous. A band scale extends the categorical scale with a band width: each category occupies a range of pixels, not just a point. The bar width in a bar chart is determined by the band width.

A color scale maps data values to colors from a colormap. For sequential data (like temperature), a sequential colormap smoothly varies from one color to another (like viridis, which goes from dark purple through blue and green to bright yellow). For diverging data (like anomalies from a mean), a diverging colormap has two colors at the extremes and a neutral color at the center. For categorical data (like species names), a qualitative palette assigns distinct colors from a predetermined list.

## How the Wilkinson Extended tick algorithm works

When starsight creates an axis, it needs to decide where to place the tick marks and what numbers to label them with. This is not trivial because the tick positions should be "nice" numbers: multiples of 1, 2, 5, 10, 20, 50, and so on, rather than arbitrary values like 17.3, 34.6, 51.9.

The Wilkinson Extended algorithm, published by Talbot, Lin, and Hanrahan in 2010, solves this problem by searching over candidate tick sequences and scoring each one on four criteria.

Simplicity measures how "round" the tick values are. Ticks at 0, 20, 40, 60, 80, 100 are simpler (rounder) than ticks at 7, 27, 47, 67, 87, 107. The algorithm has a preference-ordered list of step bases: 1, 5, 2, 2.5, 4, 3. Steps of 1 (giving ticks at 10, 20, 30) are preferred over steps of 5 (giving ticks at 5, 10, 15, 20). If zero is included as a tick, a bonus is added because humans find axes that include zero easier to read.

Coverage measures how well the tick range covers the data range. If your data goes from 3.7 to 97.2 and the ticks go from 0 to 100, the coverage is good: the tick range includes all the data. If the ticks only go from 10 to 90, the coverage is poor: some data is outside the tick range. The algorithm also penalizes excessive extension: if the data goes from 3.7 to 97.2 but the ticks go from minus 100 to 200, most of the axis is wasted.

Density measures how close the number of ticks is to a target count. If the user wants about 5 to 7 ticks and the algorithm produces 6, the density is perfect. If it produces 12 (too many) or 3 (too few), the density score is low. Density has the highest weight in the scoring function (0.5 out of 1.0) because the number of ticks most directly affects readability.

Legibility measures formatting concerns: whether labels overlap, whether they fit in the available space, and whether they use appropriate number formats. In practice, most implementations simplify this to a constant because the other three criteria capture the most important factors.

The overall score is a weighted sum: 0.2 times simplicity plus 0.25 times coverage plus 0.5 times density plus 0.05 times legibility. The algorithm searches over step base, skip factor, number of ticks, power of ten, and starting position, evaluating the score at each combination. At each level of the search, it computes an upper bound on the achievable score and prunes branches where the upper bound is below the current best score.

The pruning makes the algorithm fast. Despite searching over a large combinatorial space, the average number of candidates evaluated is about 41, regardless of the data range or target tick count. This means the algorithm runs in effectively constant time, fast enough for real-time interactive charts.

No existing Rust crate implements this algorithm. D3 (the JavaScript visualization library) uses a simpler algorithm with only three step bases. Plotters uses basic rounding. starsight will be the first Rust implementation of the full Extended Wilkinson algorithm.

## What makes a chart effective versus misleading

Understanding how humans perceive visual information helps you design charts that communicate accurately and avoid common pitfalls.

Cleveland and McGill published a foundational study in 1984 ranking visual encodings by how accurately humans can decode them. The most accurate encoding is position along a common scale: comparing two values by their positions on the same axis (like a bar chart). The next most accurate is position on non-aligned scales: comparing positions on different axes (like a multi-panel chart). Then length: comparing the lengths of bars. Then angle and slope: comparing the slopes of lines. Then area: comparing the sizes of bubbles or regions. Then volume, curvature, and shading. Color saturation is among the least accurate.

This ranking has practical implications. A pie chart encodes data as angles, which humans are bad at decoding. A bar chart encodes the same data as length, which humans are much better at. This is why bar charts are almost always better than pie charts for comparing values. Pie charts are acceptable only when you have a few categories and the goal is to show parts of a whole, not to compare individual values precisely.

Truncated axes are one of the most common sources of misleading charts. If a bar chart's y axis starts at 50 instead of 0, the visual difference between a bar at 52 and a bar at 54 is greatly exaggerated. The bar at 54 appears twice as tall as the bar at 52, even though the actual value is only 4 percent larger. For bar charts, the y axis should always start at zero because bar charts encode data as length from the baseline.

For line charts, a non-zero baseline is acceptable because line charts encode data as position, not length. Starting the y axis at a non-zero value "zooms in" on the relevant range, which is appropriate for showing trends.

Aspect ratio affects the perceived slope of lines. Cleveland proposed "banking to 45 degrees": choosing the aspect ratio so that the average absolute slope of the line segments is close to 45 degrees. This maximizes the viewer's ability to perceive slope changes, which is the primary purpose of a line chart.

Rainbow colormaps (like the infamous "jet" colormap) are harmful because they create false features. The rainbow has perceptual peaks and valleys: yellow and cyan appear brighter than red, green, and blue. These perceptual non-uniformities create the illusion of bands or edges in the data where none exist. Additionally, rainbow colormaps fail completely in grayscale (because different hues can have the same lightness) and are confusing for the approximately 8 percent of men with color vision deficiency.

Perceptually uniform colormaps, like viridis, avoid these problems. Viridis varies monotonically in lightness from dark to light, meaning it works correctly in grayscale. It varies in hue through a range that is distinguishable by people with all common forms of color vision deficiency. And equal steps in data produce equal perceived color differences, eliminating false features.

For starsight, the default colormap is viridis (from the prismatica sister crate). The default color cycle for multiple data series uses a qualitative palette (like Tableau10 or Set2) that is colorblind-safe. Users who explicitly request a rainbow colormap can have one, but the defaults prioritize correctness and accessibility.

## How tiny-skia works in detail

tiny-skia is a pure Rust port of a subset of Google's Skia graphics library. It provides CPU-based 2D rendering: drawing filled and stroked shapes, text compositing, gradient fills, pattern fills, and masking. It does not use the GPU, which makes it deterministic (the same inputs always produce the same pixels) and portable (works on any platform with a Rust compiler).

The central type is Pixmap, which owns a rectangle of premultiplied RGBA pixels. You create a Pixmap with a width and height, draw shapes onto it, and then save it as a PNG file or read its pixel data directly.

Creating a Pixmap with Pixmap new takes a width and height as unsigned 32-bit integers and returns an Option. It returns None if either dimension is zero. The newly created Pixmap is filled with transparent black (all bytes are zero).

Filling the Pixmap with a solid color uses the fill method, which takes a Color value and overwrites all pixels. The Color type in tiny-skia uses f32 components in the range 0 to 1 for straight (non-premultiplied) alpha. There are two constructors: from_rgba, which takes four f32 values and returns Option (None if any value is outside the 0 to 1 range), and from_rgba8, which takes four u8 values and always succeeds (dividing each by 255 internally).

To draw a shape, you need three things: a Path (the shape to draw), a Paint (the color and blending settings), and either a fill rule (for fills) or a Stroke (for strokes).

A Path is built with PathBuilder. You create a PathBuilder, add commands (move_to, line_to, quad_to, cubic_to, close), and call finish to get an Option of Path. The finish method returns None if the path is empty. PathBuilder also has convenience methods like push_rect, push_circle, and push_oval that add pre-built shapes to the builder.

A Paint specifies the color and blending. Its most important field is the shader, which defaults to a solid black color. The set_color_rgba8 method on Paint is the easiest way to set a solid color from u8 values. The anti_alias field controls anti-aliasing (default true). The blend_mode field controls how the new pixels combine with existing pixels (default SourceOver, which is standard alpha compositing).

For fills, you call fill_path on the Pixmap (or PixmapMut), passing the path, the paint, a FillRule (Winding or EvenOdd), a Transform, and an optional Mask. The Transform positions and scales the path. Pass Transform identity for no transformation. The Mask clips the rendering to a specific region (pass None for no clipping).

For strokes, you call stroke_path, passing the path, the paint, a Stroke struct (which specifies width, line cap, line join, miter limit, and dash pattern), a Transform, and an optional Mask.

For rectangles, there is a shortcut: fill_rect takes a Rect, paint, Transform, and Mask. This is slightly faster than creating a path from a rect.

Saving to PNG uses the encode_png method, which returns a Result of Vec of u8. The method automatically converts from premultiplied alpha to straight alpha for PNG compatibility. You can also call save_png with a file path for convenience.

The Transform type requires special attention. As mentioned earlier, from_rotate takes degrees not radians. The transform is specified as six f32 values representing a 2-by-3 affine matrix. Most operations return a new Transform rather than modifying in place. The pre_ methods apply a new transform before the existing one (conceptually: first do the new thing, then do the existing thing). The post_ methods apply after.

A Mask is a grayscale image used for clipping. You create a Mask with the same dimensions as the Pixmap, fill shapes into it (white areas allow drawing, black areas block it), and pass it to the rendering methods. For chart rendering, the mask is used to clip data points to the plot area: any part of a line or point that extends beyond the chart boundaries is invisible because the mask blocks it.

## How cosmic-text handles text in starsight

cosmic-text is a text shaping and layout library for Rust. It handles the complex process of converting a string of Unicode characters into positioned, rasterized glyphs ready for display.

The first step is creating a FontSystem. This is an expensive operation because it scans all installed fonts on the system. On a typical desktop, this takes about one second in release mode and up to ten seconds in debug mode (because the code runs without optimizations). You must create the FontSystem once and reuse it for the entire lifetime of your application. Creating a new FontSystem for each text rendering operation would be catastrophically slow.

The second step is creating a SwashCache, which caches rasterized glyphs. Like FontSystem, it should be created once and reused.

The third step is creating a Buffer. A Buffer holds the text to render, its formatting attributes, and the layout results. You create a Buffer with a FontSystem reference and Metrics. The Metrics struct specifies the font size (the height of the em square in pixels) and the line height (the distance between baselines in pixels). Typical values for chart labels are a font size of 12 to 14 and a line height of 16 to 20.

The fourth step is setting the text. You call set_text on the Buffer, passing the FontSystem, the text string, the formatting attributes (Attrs), and the shaping mode. The Attrs specify the font family (SansSerif, Serif, Monospace, or a specific font name), weight (normal, bold, or a numeric value from 100 to 900), and style (normal, italic, or oblique). The Shaping mode should be Advanced for correct results (it uses the harfrust shaping engine with font fallback and complex script support).

The fifth step is computing the layout. You call set_size to optionally constrain the text to a maximum width (for line wrapping), then call shape_until_scroll, which performs shaping and layout. After this, the Buffer knows the exact position of every glyph.

To measure the text dimensions (needed for computing chart margins), you iterate over the layout_runs. Each run represents a visible line with a line width (the horizontal extent in pixels), a line height, and a vertical position. The total text width is the maximum line width across all runs. The total text height is the bottom of the last run.

To rasterize the text, you call the draw method on the Buffer, passing the FontSystem, the SwashCache, a Color (the text color), and a callback closure. The callback receives pixel coordinates and color values for each glyph pixel. Your closure's job is to paint those pixels onto the Pixmap.

## How starsight's seven-layer architecture fits together

Layer one (starsight-layer-1) contains the foundation: geometry primitives (Point, Vec2, Rect, Size, Color, Transform), error types (StarsightError, Result), the DrawBackend trait, and the backend implementations (SkiaBackend, SvgBackend, and stubs for PDF, wgpu, and terminal).

Layer two (starsight-layer-2) contains scales (LinearScale, LogScale, and later SymlogScale, CategoricalScale, etc.), the tick generation algorithm (Wilkinson Extended), axes (an Axis combines a Scale with tick positions and labels), and coordinate systems (CartesianCoord maps data coordinates to pixel coordinates using axes).

Layer three (starsight-layer-3) contains marks (the Mark trait and implementations like LineMark, PointMark, BarMark), statistical transforms (Bin, KDE, Boxplot summary), aesthetic mappings (connecting data columns to visual properties), and position adjustments (Dodge, Stack, Jitter).

Layer four (starsight-layer-4) contains layout (GridLayout for arranging multiple charts), faceting (FacetWrap, FacetGrid), legends (mapping visual encodings back to data labels), and colorbars (displaying continuous color scales).

Layer five (starsight-layer-5) contains the Figure builder (the main API users interact with), the plot macro (the one-liner entry point), data acceptance (converting Polars DataFrames, ndarray arrays, and raw slices into the internal representation), and chart type auto-inference.

Layer six (starsight-layer-6) contains interactivity: hover tooltips, zoom, pan, selection, and streaming data. This layer is optional and behind a feature flag.

Layer seven (starsight-layer-7) contains animation (frame recording for GIFs and videos), terminal inline rendering (Kitty, Sixel, iTerm2, half-block, Braille), and export (PDF via krilla, interactive HTML).

Each layer depends only on layers below it. This is enforced by the Cargo dependency declarations: starsight-layer-3's Cargo.toml lists starsight-layer-1 and starsight-layer-2 as dependencies but does not list layer-4 through layer-7. If you accidentally try to import something from a higher layer, the compiler will reject it.

The starsight facade crate depends on all seven layers and re-exports their public APIs. Users depend only on the starsight crate and import types through it. The layer crates are internal implementation details.

## Why Point and Vec2 are separate types

Point represents a position in space: "the pixel at (100, 200)." Vec2 represents a displacement or direction: "50 pixels to the right, 30 pixels down." They both have two f32 fields (x and y), but the valid operations on them are different.

Subtracting one Point from another gives a Vec2: the displacement between two positions. Your house minus the grocery store is the direction and distance from the store to your house. This is a displacement, not a location.

Adding a Vec2 to a Point gives a new Point: shifting a position by a displacement. Your house plus the displacement to the store gives the store's location. This is a new position.

Adding two Points together is meaningless. Your house plus the grocery store is not a place. The type system prevents this: the compiler rejects Point plus Point.

Multiplying a Vec2 by a scalar gives a Vec2: scaling a displacement. Half the distance to the store is a valid displacement. Multiplying a Point by a scalar is meaningless: double your house is not a place.

This distinction catches real bugs. In chart layout code, you work with positions (where does this label go) and offsets (how much margin to add). If both are just tuples of floats, nothing stops you from accidentally adding two positions and getting garbage. With Point and Vec2, the compiler catches this.

## How the full rendering pipeline works from data to pixels

This is the end-to-end flow that produces a chart image from data. Understanding it is more important than understanding any single component, because every bug you encounter will live at a boundary between stages.

Stage one: data enters through layer five. The user calls the plot macro with two arrays (x values and y values), or passes a Polars DataFrame with column names. Layer five does not render anything. It creates a description: "this data should become a line chart with blue lines."

Stage two: the Figure builder collects marks. A LineMark holds the x and y data, a color, and a line width. It does not hold pixel positions. It does not know the chart dimensions or margins.

Stage three: when save or show is called, layer five asks layer two to create scales. Layer two computes the data range (minimum and maximum of x and y values), runs the Wilkinson tick algorithm to find nice tick positions, and creates LinearScales with domains that extend to the tick boundaries.

Stage four: margin computation. The Figure needs to know how much space to reserve for axis labels, tick labels, and the title. This requires measuring text widths, which requires cosmic-text's FontSystem. The margin computation is the trickiest part because it is circular: you need the plot area size to determine the tick positions, but you need the tick positions to determine the tick labels, and you need the tick labels to determine the margins, and you need the margins to determine the plot area size.

The solution is two passes. First pass: create scales, generate tick labels, measure their widths with cosmic-text, compute margins, compute the plot area (figure dimensions minus margins). Second pass: with the plot area known, create a CartesianCoord that maps data coordinates to pixel coordinates within that area.

Stage five: each mark renders itself. The LineMark iterates its data points, calls data_to_pixel on the CartesianCoord for each point, and produces a sequence of PathCommand values: MoveTo for the first point, LineTo for each subsequent point. If any data value is NaN, the LineMark starts a new MoveTo, which breaks the line at the gap.

Stage six: the path commands hit the backend. The SkiaBackend converts them to a tiny-skia Path using PathBuilder, creates a Paint with the stroke color, creates a Stroke with the line width and line cap, and calls stroke_path on the Pixmap. The SvgBackend converts them to SVG path data and adds them to the document.

Stage seven: the backend serializes the result. The SkiaBackend calls encode_png to produce PNG bytes or save_png to write a file. The SvgBackend writes the SVG document to a file.

Every stage is a separate concern in a separate layer. When something goes wrong, the layer boundary tells you where to look.



## What a pixel actually is

A pixel is not a little square. This is a common misconception. A pixel is a point sample: a measurement of color at a specific location on a regular grid. Think of it like a weather station that measures temperature at one geographic point. The station does not "own" a square area around it; it simply records the value at its location.

When you display an image on a screen, the hardware reconstructs a continuous image from these discrete samples. On most screens, this reconstruction makes each pixel look like a colored square, which is why the misconception exists. But the mathematical reality is that pixels are points, and the reconstruction process (which determines how the image looks between sample points) is a separate concern.

For starsight, you work with pixels through a pixel buffer (called a Pixmap in tiny-skia). The buffer is a flat array of bytes in memory. Each pixel is stored as four consecutive bytes: red, green, blue, and alpha (transparency). For a buffer with a width of 800 and a height of 600, the total size is 800 times 600 times 4 equals 1,920,000 bytes. The bytes are arranged in rows (called scanlines), where each row contains 800 pixels (3,200 bytes).

To find the bytes for the pixel at column x and row y, you compute the index as (y times width plus x) times 4. This gives you the byte offset of the red channel. The green channel is at offset plus 1, blue at plus 2, and alpha at plus 3.

## What premultiplied alpha means and why it matters

Alpha is the transparency channel. An alpha value of 255 (the maximum for an 8-bit channel) means fully opaque: you cannot see anything behind this pixel. An alpha of 0 means fully transparent: this pixel is invisible. Values in between represent partial transparency.

There are two ways to store alpha. In straight alpha (also called unassociated alpha), the red, green, and blue channels store the actual color, and the alpha channel stores the transparency separately. A pixel that is 50 percent transparent red would be stored as red equals 255, green equals 0, blue equals 0, alpha equals 128.

In premultiplied alpha (also called associated alpha), the color channels are multiplied by the alpha before storage. The same 50 percent transparent red would be stored as red equals 128, green equals 0, blue equals 0, alpha equals 128. The red channel is 255 times 128 divided by 255, which equals 128.

Why does this matter? Because the compositing formula (how you combine a foreground pixel with a background pixel) is much simpler with premultiplied alpha. The standard compositing operation is called Source Over, and in premultiplied form it is: result equals source plus destination times (1 minus source alpha). In straight alpha, the formula requires additional multiplications and a division, and there is a risk of division by zero when alpha is zero.

tiny-skia stores all pixels in premultiplied alpha format. This is the standard in professional graphics: Photoshop, After Effects, and virtually all compositing software use premultiplied internally. When tiny-skia encodes a PNG, it converts from premultiplied back to straight alpha (because PNG uses straight alpha). This conversion is automatic.

For starsight, you mostly do not need to think about premultiplication. You specify colors in straight form (red equals 255, alpha equals 128), tiny-skia premultiplies them internally, and the compositing math works correctly. The only time it matters is if you read raw pixel data from the buffer for testing or processing: the values you see are premultiplied, not the colors you originally specified.

## How lines are drawn on a pixel grid

Drawing a straight line between two points on a pixel grid is one of the oldest problems in computer graphics. The challenge is that the mathematical line is infinitely thin and can pass through any point, but the pixel grid is discrete: you can only color whole pixels.

The classic solution is Bresenham's line algorithm, developed at IBM in 1962. For a line that is more horizontal than vertical (the horizontal distance is greater than the vertical distance), the algorithm steps through each column from left to right and decides whether to keep the same row or move up (or down) by one pixel. The decision is based on an error accumulator that tracks how far the chosen pixels are from the true mathematical line. When the error exceeds half a pixel, the row changes. The beauty of the algorithm is that it uses only integer addition and comparison, no multiplication or division.

In practice, modern rasterizers like tiny-skia do not use Bresenham's algorithm directly. They use a more sophisticated approach called scanline rasterization with anti-aliasing. Instead of coloring entire pixels as either fully on or fully off, the rasterizer computes the fraction of each pixel that is covered by the geometric shape. This coverage fraction becomes the alpha value, producing smooth edges that blend with the background. A pixel that is half covered by the line gets alpha equals 128, producing a smooth gradient from the line's color to the background color.

Anti-aliasing is what makes the difference between a line that looks like a staircase of jagged blocks (aliased) and a line that looks smooth and natural (anti-aliased). tiny-skia enables anti-aliasing by default on the Paint struct, and starsight should generally leave it on for diagonal lines and curves. The exception is perfectly horizontal or perfectly vertical lines at integer coordinates, where anti-aliasing actually makes the line look blurry (it bleeds into adjacent pixels instead of being a crisp single-pixel line). For axis lines and tick marks, anti-aliasing should be disabled.

## How curves work in computer graphics

A straight line is defined by two points: the start and the end. But many shapes in charts need smooth curves: the rounded corners of bar charts, the smooth density curves of violin plots, the arcs of pie charts.

The standard curve primitive in computer graphics is the Bezier curve, named after Pierre Bezier who used them for car body design at Renault in the 1960s. A Bezier curve is defined by control points. The curve starts at the first control point, ends at the last, and is pulled toward the intermediate control points without necessarily passing through them.

A linear Bezier is just a line segment: two control points, the start and end.

A quadratic Bezier has three control points: start, control, and end. The curve starts at the start point, bends toward the control point, and ends at the end point. Quadratic Beziers can represent simple bends but cannot create S-shapes within a single segment.

A cubic Bezier has four control points: start, first control, second control, and end. The two intermediate control points give you independent control over the curve's direction at the start and end. Cubic Beziers can represent S-shapes, which makes them the most common curve type in computer graphics. SVG paths, PDF paths, and tiny-skia paths all use cubic Beziers.

The mathematical formula for a cubic Bezier is a weighted combination of the four control points, where the weights are polynomials of the parameter t that ranges from 0 to 1. At t equals 0, you get the start point. At t equals 1, you get the end point. At intermediate values, you get points along the curve. The tangent direction at the start is from the start toward the first control point. The tangent direction at the end is from the second control point toward the end.

The de Casteljau algorithm evaluates a Bezier curve at a given t by repeatedly performing linear interpolation. For a cubic with four points, you interpolate between consecutive pairs to get three points, then interpolate between those to get two points, then one final point on the curve. This algorithm is numerically stable and also gives you a way to split a curve into two sub-curves.

For starsight, Bezier curves appear everywhere. The line mark connects data points with line segments (linear Beziers). Smooth line marks might use cubic Beziers to create splines through the data points. Bar charts with rounded corners use cubic Bezier arcs for the corners. Pie and donut charts use circular arcs, which are approximated by cubic Beziers (four arcs for a full circle). The path command types in starsight match tiny-skia's path types: MoveTo, LineTo, QuadTo (quadratic Bezier), CubicTo (cubic Bezier), and Close.

## How paths are filled and stroked

A path in computer graphics is a sequence of connected segments: lines and curves. A path can be open (it has distinct start and end points) or closed (the end connects back to the start).

Stroking a path means drawing along it with a certain width, like running a pen along a line. The stroke width is applied symmetrically: half on each side of the mathematical center line. At the endpoints of open paths, the stroke needs a cap: Butt (flat cut exactly at the endpoint), Round (semicircle extending beyond the endpoint), or Square (rectangular extension by half the stroke width). At corners where segments meet, the stroke needs a join: Miter (sharp point where outer edges meet), Round (circular arc), or Bevel (flat cut). The miter limit prevents extremely long spikes at very acute angles.

Filling a path means coloring the interior of a closed shape. But what counts as "interior" for a complex path that crosses itself? There are two rules. The even-odd rule casts an imaginary ray from each point and counts how many times it crosses the path boundary. If the count is odd, the point is inside; if even, it is outside. The non-zero winding rule also casts a ray but tracks direction: crossings in one direction add to the count, crossings in the other subtract. If the total is non-zero, the point is inside. For simple shapes, both rules give the same result. For self-crossing shapes like a star, they differ: even-odd leaves the center unfilled, while non-zero fills it.

For starsight, stroked paths are used for line charts (the data line), axis lines, tick marks, and chart borders. Filled paths are used for bar charts, area charts, pie slices, and background rectangles. The tiny-skia methods fill-path and stroke-path handle the rasterization. You build the path using a PathBuilder, configure the appearance using Paint (color, anti-aliasing) and Stroke (width, cap, join, dash pattern), and call the appropriate method on the Pixmap.

## How coordinate systems work in 2D graphics

In screen coordinates (used by virtually all graphics APIs including tiny-skia), the origin is at the top-left corner of the image. The x axis increases to the right. The y axis increases downward. This is counterintuitive if you are used to mathematical coordinates where y increases upward, but it matches how CRT monitors scanned: the electron beam moved left to right, top to bottom.

This means that to plot a data point where larger y values should appear higher on the chart, you need to invert the y coordinate. If the plot area spans from pixel 100 (top) to pixel 500 (bottom), and a data value maps to 70 percent of the way up the axis, the pixel y coordinate is 500 minus 0.7 times (500 minus 100) equals 500 minus 280 equals 220. The subtraction from the bottom flips the direction.

An affine transform combines translation (shifting), scaling (stretching), and rotation into a single mathematical operation represented by a matrix of six numbers. You can compose transforms by multiplying matrices: first scale, then translate, is a single combined transform. tiny-skia's Transform type handles this composition.

Critical gotcha: tiny-skia's rotation function takes the angle in degrees, not radians. This is unlike virtually every other math library. If you pass the mathematical constant pi divided by 2 expecting a 90-degree rotation, you will get a rotation of approximately 1.57 degrees instead, producing a nearly invisible tilt that is extremely confusing to debug.

For starsight, the coordinate transformation pipeline goes: data coordinates (the user's data values, like temperature from 0 to 100) are mapped to normalized coordinates (0 to 1) by the scale, then to pixel coordinates within the plot area, then optionally transformed by the tiny-skia Transform for the final rendering. The y-inversion happens during the data-to-pixel mapping, not in the transform.

## How color works, from light physics to bytes in memory

Color starts with electromagnetic radiation. Visible light spans wavelengths from about 380 nanometers (violet) to about 780 nanometers (red). White light is a mixture of many wavelengths. When light hits an object, some wavelengths are absorbed and others are reflected. The reflected wavelengths determine the color we perceive.

Human eyes have three types of color-sensitive cells called cones: S-cones (sensitive to short wavelengths, roughly blue), M-cones (medium wavelengths, roughly green), and L-cones (long wavelengths, roughly red). Because we have three independent color channels, we can represent most visible colors as combinations of three primaries. This is why screens use red, green, and blue (RGB) LEDs: each pixel has three sub-elements that independently stimulate the three cone types.

In a digital image, each color channel is typically stored as an 8-bit unsigned integer, giving values from 0 (no light from this primary) to 255 (maximum light from this primary). This gives 256 times 256 times 256 equals 16,777,216 possible colors, which is sufficient for most purposes.

But there is a subtlety. The relationship between the stored number and the physical light intensity is not linear. This is because of the sRGB standard, which defines a nonlinear transfer function (often loosely called "gamma") between stored values and physical intensity. The sRGB encoding compresses bright values and expands dark values, allocating more of the 256 steps to the perceptually important dark-to-mid range and fewer to the bright range.

The precise sRGB formula is: for linear values at or below 0.0031308, the sRGB value equals linear times 12.92. For linear values above 0.0031308, sRGB equals 1.055 times linear raised to the power of 1 divided by 2.4, minus 0.055. The inverse (decoding from sRGB to linear) is: for sRGB values at or below 0.04045, linear equals sRGB divided by 12.92. For sRGB values above 0.04045, linear equals the quantity (sRGB plus 0.055) divided by 1.055, raised to the power 2.4.

Why does this matter for starsight? Because blending (mixing two colors) should be done in linear space, not in sRGB space. If you average a bright red (sRGB 255, 0, 0) and black (sRGB 0, 0, 0) in sRGB space, you get (128, 0, 0), which corresponds to only about 21 percent of the physical red intensity. If you average in linear space, you get 50 percent physical intensity, which encodes back to sRGB as approximately (188, 0, 0). The linear blend looks correct; the sRGB blend looks too dark.

In practice, tiny-skia defaults to blending in sRGB space (despite having a confusingly named "Linear" colorspace option that means "no correction"). This matches the behavior of most legacy graphics systems. For starsight version 0.1.0, this is acceptable. Gamma-correct blending can be added as an option in a later version.

For accessibility, the WCAG (Web Content Accessibility Guidelines) define a contrast ratio formula using relative luminance. The relative luminance of a color is computed from its linearized RGB values: L equals 0.2126 times R-linear plus 0.7152 times G-linear plus 0.0722 times B-linear. The contrast ratio between two colors is (L-lighter plus 0.05) divided by (L-darker plus 0.05). WCAG requires a minimum contrast ratio of 4.5 to 1 for normal text and 3 to 1 for large text.

## How colormaps encode data as color

A colormap (also called a color scale or color palette) is a function that maps a numerical range to a range of colors. When you create a heatmap, each cell's color represents its data value: low values might be dark blue, medium values might be green, and high values might be bright yellow.

There are three main categories of colormaps. Sequential colormaps vary monotonically from one color to another, typically going from dark to light. They are appropriate for data that ranges from low to high, like temperature, elevation, or density. Diverging colormaps have two distinct colors at the extremes with a neutral center, appropriate for data with a meaningful midpoint like positive versus negative deviations, or above versus below average. Qualitative colormaps (also called categorical palettes) use distinct colors for categorical labels, with no implied ordering between them.

The choice of colormap has real consequences for how effectively the visualization communicates. Rainbow colormaps (like the infamous "jet" colormap) are widely recognized as harmful because they are not perceptually uniform (equal steps in data do not produce equal perceived color differences), they create false visual boundaries (the yellow band appears as a ridge), they fail completely in grayscale (multiple distinct colors map to the same gray), and they are indistinguishable for the approximately 8 percent of men who have color vision deficiency.

The viridis colormap, developed by the matplotlib team, is the gold standard for sequential data. It is perceptually uniform (uniform data produces uniform-looking color changes), monotonically increasing in lightness (it works in grayscale), and designed to be distinguishable by people with the most common forms of color blindness. The prismatica sister crate provides viridis and 307 other scientifically validated colormaps as compile-time data.

For starsight, the default colormap for continuous data should be viridis or a similar perceptually uniform option. The default color cycle for categorical data (distinguishing multiple series on the same chart) should use a palette designed for colorblind safety, like Tableau10 or Set2.

## How text rendering works from character to pixel

Rendering text is surprisingly complex. The pipeline has four stages: font selection, shaping, layout, and rasterization.

Font selection determines which font file provides the glyphs for the text. A font file (typically in TrueType or OpenType format) is a structured database containing glyph outlines (mathematical curve descriptions), metrics tables (sizes and spacing), character-to-glyph mapping tables, and feature tables for ligatures and contextual forms. When you request "14 pixel sans-serif," the font system searches installed fonts for one matching that description.

On each operating system, the font discovery mechanism is different. Linux uses fontconfig, macOS uses Core Text, and Windows uses the registry. cosmic-text (the text rendering library starsight uses) abstracts this through its FontSystem type, which scans all installed fonts on initialization. This initialization takes about one second in optimized builds and up to ten seconds in debug builds, which is why FontSystem must be created once and reused, not created per draw call.

Shaping converts a sequence of Unicode characters into a sequence of positioned glyphs. This is not a simple lookup because the mapping between characters and glyphs is context-dependent. In Arabic script, each letter has up to four forms depending on whether it appears at the beginning, middle, or end of a word, or stands alone. In Latin script, the pair "fi" may be replaced by a single ligature glyph. Kerning adjusts the spacing between specific letter pairs: the gap between A and V is typically reduced because their shapes nestle together.

cosmic-text uses harfrust (a pure Rust port of the HarfBuzz shaping engine) for this stage. For chart labels, which are usually short strings of digits and Latin letters, shaping mostly just assigns glyph indices and advance widths. But it still must run because even simple Latin text has kerning that affects spacing.

Layout arranges shaped glyphs into lines. For chart tick labels, each label is a single line, so layout is trivial. For multi-line titles or wrapped annotations, the layout engine decides where to break lines based on available width, hyphenation rules, and the Unicode Line Breaking Algorithm. cosmic-text's Buffer type manages this. You set the text, set the available width and height, and call shape-until-scroll to compute the layout.

Rasterization converts glyph outlines (mathematical curves) into pixel coverage values. Each glyph is rendered into a small image where each pixel's value represents how much of that pixel is covered by the glyph shape. A fully covered pixel gets value 255. A partially covered pixel (at the edge of a curve) gets a proportional value. This coverage image is then composited onto the main pixel buffer using the text color and the coverage as alpha.

cosmic-text uses the swash library for rasterization. The rendered glyphs are cached in a SwashCache so that the same glyph at the same size is only rasterized once. The draw method on Buffer calls a user-provided callback for each pixel of each glyph. The callback receives the x and y position, the width and height (typically both 1), and the color with alpha adjusted for coverage.

For starsight, measuring text dimensions (needed for computing axis margins) is done by iterating layout runs after shaping. Each run has a line width and a line height. The total text width is the maximum line width. The total height is the last line's y position plus its height.

A common myth is that you need to swap the red and blue channels between cosmic-text and tiny-skia. You do not. That swap appears in cosmic-text's example code because the example renders to softbuffer, which uses a different byte order. For PNG and SVG output, pass the color channels straight through.

## What the grammar of graphics means

The grammar of graphics is a theoretical framework developed by Leland Wilkinson in his 1999 book of the same name. The core idea is that any statistical chart can be described as a composition of independent components, just as any sentence in a natural language can be described as a composition of grammatical elements.

Instead of thinking of a scatter plot as one thing and a bar chart as a different thing and a histogram as yet another thing, the grammar of graphics decomposes them into shared building blocks. The components are: data (the information being visualized), aesthetic mappings (how data variables map to visual properties), geometric objects (the visual shapes drawn for each data point), statistical transformations (computations performed on the data before plotting), position adjustments (how overlapping objects are arranged), scales (functions that convert data values to visual values), coordinate systems (the spatial framework), and facets (how to split data into subsets for multiple panels).

The power of this decomposition is combinatorial. A scatter plot is a point geometry with x and y aesthetic mappings. A line chart is a line geometry with the same mappings. A bar chart is a bar geometry with x as categorical and y as quantitative. A histogram is a bar geometry with a binning statistical transform. A violin plot is an area geometry with a kernel density estimation transform, reflected vertically. A stacked bar chart is a bar geometry with a stacking position adjustment. Each component is independent and can be combined freely with any other.

For starsight, this decomposition maps directly to the library architecture. Aesthetic mappings are types in layer three. Geometric marks (Point, Line, Bar, Area, Arc) are types in layer three that implement the Mark trait. Statistical transforms (Bin, KDE, Boxplot, Regression) are types in layer three that transform data before it reaches the marks. Scales (Linear, Log, Band, Color) are types in layer two. Coordinate systems (Cartesian, Polar) are types in layer two. Faceting (Wrap, Grid) is in layer four.

The Figure builder in layer five is where the user composes these components. The convenience functions and the plot macro are shortcuts that assemble common combinations: plot of x, y internally creates a Figure with a CartesianCoord, a LinearScale for each axis, and a LineMark with the data. But the user can also build the Figure manually, choosing each component independently, which is how advanced chart customization works.

## How scales map data to visual space

A scale is a function that converts a data value into a visual value. The simplest scale is a linear scale. Given a data domain (the range of data values, say 0 to 100) and a visual range (the range of pixel positions, say 50 to 750), the linear scale maps the data value to a pixel position proportionally.

The formula is: pixel equals (data minus domain-min) divided by (domain-max minus domain-min), times (range-max minus range-min), plus range-min. The first part normalizes the data value to the 0-to-1 range. The second part scales it to the pixel range.

A logarithmic scale applies a logarithm before the linear mapping. This compresses large values and expands small values. A data range from 1 to 10,000 on a log scale spaces the values 1, 10, 100, 1000, 10000 evenly. Log scales are used for data that spans many orders of magnitude, like earthquake magnitudes, sound decibels, or pH levels. Log scales require strictly positive data; the logarithm of zero is negative infinity, and the logarithm of a negative number is undefined.

A symmetric log (symlog) scale handles data that crosses zero. It applies a logarithm to the absolute value, preserves the sign, and uses a linear region near zero to avoid the logarithmic singularity. The threshold parameter controls the width of the linear region. Symlog is useful for financial data (stock returns can be positive or negative), scientific data (residuals from a model), and any measurement that spans zero with a wide dynamic range.

A categorical scale (also called a band scale) maps discrete labels to evenly spaced positions. The categories "Apple," "Banana," and "Cherry" might map to pixel positions 150, 400, and 650. Each category occupies a band of pixels, not just a point. The band width determines how wide each bar can be in a bar chart.

A color scale maps data values to colors using a colormap. A sequential color scale might map temperature values from cold (blue) to hot (red). A diverging color scale might map deviations from zero, with negative values in one color and positive in another.

For starsight, each axis has a scale. The x axis might use a linear scale for continuous data or a band scale for categorical data. The y axis typically uses a linear scale but might use a log scale for data with wide ranges. Color scales are used when a third variable is mapped to color, as in heatmaps and colored scatter plots.

## How the tick algorithm decides where to put axis labels

When you draw an axis for a data range of, say, 3.7 to 97.2, you need to decide where to put the numbered labels (tick marks). You want the labels to be at "nice" round numbers (like 0, 20, 40, 60, 80, 100) rather than awkward values (like 3.7, 22.4, 41.1, 59.8, 78.5, 97.2). You want the labels to cover the data range without too much wasted space. And you want roughly the right number of labels (too few and the axis is hard to read; too many and the labels overlap).

starsight uses the Wilkinson Extended algorithm, published by Talbot, Lin, and Hanrahan in 2010. This algorithm searches over a large space of possible tick configurations and scores each one on four criteria.

Simplicity measures how "round" the tick values are. Ticks at 10, 20, 30 are simpler than ticks at 15, 30, 45, which are simpler than ticks at 12, 24, 36. The algorithm uses a preference list of step bases: 1 is preferred over 5, which is preferred over 2, which is preferred over 2.5, which is preferred over 4, which is preferred over 3.

Coverage measures how well the tick range matches the data range. If the data goes from 3.7 to 97.2 and the ticks go from 0 to 100, the coverage is good. If the ticks go from 0 to 200, the extra space from 100 to 200 is wasted. If the ticks go from 10 to 90, some data falls outside the tick range.

Density measures how close the number of ticks is to the desired target. If you want about 5 to 7 ticks and the algorithm produces 6, the density is perfect. If it produces 15, the labels will be too crowded.

Legibility measures formatting quality. In most implementations, this is simplified to a constant because the other three criteria capture the important factors.

The overall score is a weighted combination: 0.2 times simplicity, plus 0.25 times coverage, plus 0.5 times density, plus 0.05 times legibility. Density gets the highest weight because having the right number of ticks matters most for readability.

The algorithm uses nested loops with pruning. At each level, it computes an upper bound on the best possible score achievable by any remaining candidate. If this upper bound is less than the best score found so far, it skips the rest of that branch. This makes the algorithm fast: on average, about 41 candidates are evaluated regardless of the data range, which takes well under a millisecond.

No existing Rust crate implements this algorithm. D3 (the JavaScript visualization library) uses a simpler algorithm with fewer step bases. Plotters uses basic rounding. starsight will be the first Rust implementation of the full Extended Wilkinson algorithm.

## How charts go from data to pixels: the complete pipeline

Understanding the end-to-end pipeline is more important than understanding any single component, because most bugs live at the boundaries between stages.

Stage one: data arrives. The user provides arrays of numbers, or a DataFrame, or any supported data format. The data enters through layer five, either via the plot macro or the Figure builder.

Stage two: the Figure builder collects marks. A mark is a description, not a visual element. A LineMark holds the x data, the y data, a color, and a line width. It holds no pixel coordinates.

Stage three: when save or show is called, the Figure asks layer two to create scales from the data ranges. The scale computes the data domain (minimum and maximum) and runs the tick algorithm to find nice tick positions. The tick positions may extend beyond the data range to reach round numbers.

Stage four: the Figure computes the plot area. The full image might be 800 by 600 pixels, but the chart does not fill the entire image. There are margins for the title, the axis labels, the tick labels, and padding. Computing the margins requires knowing how wide the tick labels are, which requires measuring text, which requires the font system. This is a two-pass process: first compute scales and measure text to determine margins, then compute the plot area from the remaining space.

Stage five: each mark renders itself. The LineMark iterates its data points, calls the coordinate system's data-to-pixel mapping for each pair, and produces a sequence of path commands: move to the first pixel position, line to the second, line to the third. NaN values in the data produce gaps (a new move-to instead of a line-to).

Stage six: the path commands hit the backend. The tiny-skia backend converts them to a tiny-skia Path, creates a Paint with the appropriate color and a Stroke with the appropriate width, and calls stroke-path on the Pixmap. The SVG backend converts them to SVG path data strings.

Stage seven: the backend serializes the result. The tiny-skia backend calls encode-png. The SVG backend writes the document to a string.

Every step is a separate concern in a separate layer. Data acceptance is layer five. Scale computation is layer two. Mark description is layer three. Rendering is layer one. When something goes wrong, the layer boundary tells you where to look.

## What makes a chart effective versus misleading

Not all charts are created equal. A well-designed chart communicates data clearly and honestly. A poorly designed chart obscures the data or actively misleads.

The most important principle is that the visual encoding should match the data type. Position along a common axis (the encoding used in scatter plots and line charts) is the most accurate encoding for quantitative data. Length (used in bar charts) is slightly less accurate but still good. Angle (used in pie charts) is harder for humans to judge precisely. Area (used in bubble charts) is even harder. Color saturation (used in heatmaps) is the hardest to judge quantitatively, which is why heatmaps need a color scale legend and should use perceptually uniform colormaps.

Truncated axes are a common source of misleading charts. In a bar chart, the y axis should always start at zero because bars encode values through their length from the baseline. If the axis starts at 50, a bar representing 55 looks the same height as one representing 100, which is wildly misleading. For line charts showing trends, a non-zero baseline is acceptable because the user is comparing slopes and changes, not absolute lengths.

The aspect ratio of a chart affects how the data is perceived. A steep line in a tall narrow chart looks like a gentle slope in a wide flat chart. The advice from William Cleveland's 1988 paper is to choose an aspect ratio that makes the average line slope close to 45 degrees, which maximizes the viewer's ability to distinguish different slopes.

The data-ink ratio, a concept from Edward Tufte, says to maximize the proportion of "ink" (pixels) that represents data and minimize decorative elements. Heavy gridlines, 3D perspective effects, gradient fills, and ornamental borders all reduce the data-ink ratio. A clean chart with data, axes, and labels is almost always more effective than an embellished one.

For starsight, these principles inform the defaults. The default background is white. Gridlines are off by default. The default color cycle uses a colorblind-safe palette. The y axis in bar charts starts at zero. The default aspect ratio is 4:3. The user can override all of these, but the defaults should produce effective charts without customization.


## How tiny-skia renders shapes onto pixels

tiny-skia is a pure-Rust port of a subset of the Skia graphics library (which powers Chrome and Android). It provides CPU-based 2D rendering: you create a pixel buffer, draw shapes onto it, and encode the result as PNG. There is no GPU, no window system, and no external C dependencies. It is about 14,000 lines of Rust and adds approximately 200 kilobytes to a binary.

The central type is Pixmap, which owns the pixel buffer. You create one with Pixmap new, passing the width and height. This allocates a buffer of width times height times 4 bytes, initialized to transparent black. The method returns Option of Pixmap, returning None if the dimensions are zero or too large.

To draw on the Pixmap, you need three ingredients: a shape (what to draw), a paint (how to color it), and a transform (where to position it).

The shape is a Path, built using PathBuilder. You create a PathBuilder, call move-to to set the starting position, call line-to to add straight segments, call quad-to or cubic-to to add curves, and call close to connect back to the start. When done, you call finish, which returns Option of Path (None if the path is empty). There are convenience constructors: from-rect creates a rectangular path, from-circle creates a circular path.

The paint is a Paint struct with public fields that you set directly. The most important fields are: shader (defaulting to solid black; you set it with set-color or set-color-rgba8), anti-alias (defaulting to true), and blend-mode (defaulting to SourceOver, the standard alpha compositing mode).

The transform is a Transform struct representing an affine transformation: translation, scaling, rotation, and skewing combined into six numbers. Transform identity is the default (no transformation). The critical thing to remember is that Transform from-rotate takes degrees, not radians.

To render, you call methods on the Pixmap: fill-path (to fill the interior of a closed shape), stroke-path (to draw along the outline of a shape), or fill-rect (a convenience method for axis-aligned rectangles). Each method takes the shape, the paint, additional parameters (like the Stroke struct for stroke-path or the FillRule for fill-path), the transform, and an optional Mask for clipping.

The Stroke struct controls the appearance of stroked paths. Its fields include width (in pixels), line-cap (Butt, Round, or Square), line-join (Miter, MiterClip, Round, or Bevel), miter-limit (default 4.0), and dash (an optional StrokeDash with an array of dash-gap lengths and an offset).

The FillRule enum has two variants: Winding (the default, where direction matters) and EvenOdd (where only crossing count matters). For starsight, Winding is used for most shapes.

For clipping (restricting drawing to a specific region), you create a Mask of the same dimensions as the Pixmap, fill a rectangle into the mask (the allowed region), and pass the mask to the drawing methods. Pixels outside the mask are not modified. This is how starsight prevents line chart data from extending beyond the plot area.

To save the result, call encode-png on the Pixmap, which returns a Vec of bytes in PNG format. Or call save-png with a file path. The encoding automatically converts from premultiplied alpha (tiny-skia's internal format) to straight alpha (PNG's format).

The Color type in tiny-skia has private fields (four f32 values in straight alpha format, guaranteed to be in the 0 to 1 range). The constructor from-rgba takes four f32 values and returns Option of Color (None if out of range). The constructor from-rgba8 takes four u8 values (0 to 255) and always succeeds, dividing by 255 internally.

## How cosmic-text renders strings to glyphs

cosmic-text is the text rendering library that starsight uses to draw axis labels, tick labels, titles, and annotations. It handles the full pipeline from Unicode text to positioned glyph pixels.

You start by creating a FontSystem, which scans and loads all installed system fonts. This is expensive (about one second in release builds) and must be done only once. The FontSystem is then passed by mutable reference to all subsequent text operations.

Next, create a SwashCache, which caches rasterized glyph images. Glyphs are rasterized (converted from mathematical outlines to pixel coverage) on first use and cached for subsequent uses. This avoids re-rasterizing the same character every time it appears.

Then create a Buffer, which holds the text to be laid out. The Buffer constructor takes a mutable FontSystem reference and a Metrics struct. The Metrics struct specifies the font size (in pixels, as a float) and the line height (baseline-to-baseline distance in pixels, also as a float).

Set the text on the Buffer using set-text, passing the FontSystem reference, the text string, an Attrs struct (specifying font family, weight, and style), and a Shaping mode (Basic for fast simple shaping, or Advanced for full Unicode shaping with font fallback). Then call shape-until-scroll with the FontSystem reference and a boolean parameter (true to prune off-screen content).

After shaping, you can measure the text dimensions by iterating layout-runs. Each run has a line-w (width in pixels) and a line-height. The total width is the maximum line-w across all runs. The total height is the last run's line-top plus its line-height. This measurement is needed for computing chart margins.

To actually render the glyphs onto a tiny-skia Pixmap, call the draw method on the Buffer. It takes the FontSystem reference, the SwashCache reference, a default text Color, and a closure. The closure is called once per pixel of each rasterized glyph, receiving the x position, y position, width, height, and color. Inside the closure, you write the pixel to the Pixmap. The color already includes the glyph's coverage as alpha, so you just set the pixel using tiny-skia's Pixmap methods.

There is a persistent myth that you need to swap the red and blue channels when transferring pixels from cosmic-text to tiny-skia. You do not. That swap exists in cosmic-text's example code because the example renders to the softbuffer library, which uses a different byte order. For file output (PNG), pass the channels through unmodified.

## How starsight's seven-layer architecture works

The library is organized into seven layers, each a separate Cargo crate, plus a facade crate and a development tools crate. The layers form a strict dependency hierarchy: each layer depends only on layers below it.

Layer one (starsight-layer-1) is the foundation. It contains: the geometry primitives (Point, Vec2, Rect, Size, Color, ColorAlpha, Transform), the error type (StarsightError with seven variants), the DrawBackend trait (the interface all rendering backends implement), and the backend implementations (tiny-skia for CPU rendering, SVG for vector output, PDF for print, wgpu for GPU, and terminal for inline display). This layer has the fewest dependencies and the broadest reuse.

Layer two (starsight-layer-2) is scales and coordinates. It contains: the Scale trait and concrete implementations (LinearScale, LogScale, SymlogScale, BandScale, ColorScale), the tick generation algorithm (Wilkinson Extended), the Axis struct (bundling a scale with tick positions and labels), and the CartesianCoord struct (mapping data coordinates to pixel positions with Y-axis inversion).

Layer three (starsight-layer-3) is marks and statistics. It contains: the Mark trait (the interface all visual elements implement), concrete mark implementations (LineMark, PointMark, BarMark, AreaMark, and eventually all 60-plus chart types), statistical transforms (Bin for histograms, KDE for density estimation, Boxplot for five-number summaries), and position adjustments (Dodge, Stack, Jitter for handling overlapping elements).

Layer four (starsight-layer-4) is layout and composition. It contains: GridLayout for arranging multiple charts, FacetWrap and FacetGrid for splitting data into panels, Legend for mapping visual encodings back to data labels, and Colorbar for displaying continuous color scales.

Layer five (starsight-layer-5) is the high-level API. It contains: the Figure builder (the main interface users interact with), the plot macro (for quick one-line charts), data acceptance modules (converting from Polars DataFrames, ndarray arrays, Arrow RecordBatches, and plain slices), and auto-inference of chart types from data shape.

Layer six (starsight-layer-6) is interactivity. It contains: hover tooltips, box zoom, wheel zoom, pan, lasso selection, linked views between charts, and streaming data with rolling windows. This layer requires a windowing system (winit for native, web-sys for browser) and is entirely optional.

Layer seven (starsight-layer-7) is animation and export. It contains: frame recording, transition animations, static export to PNG/SVG/PDF, interactive HTML export, and terminal inline output.

The facade crate (starsight) re-exports everything and is the only crate users depend on. The xtask crate provides development automation (gallery generation, benchmarks) and is never published.

Why separate crates instead of modules within a single crate? Compile-time isolation: changing code in layer three does not require recompiling layers one and two. Feature gating: the GPU backend in layer one can be behind a feature flag without affecting layer three. API boundary enforcement: layer three literally cannot import anything from layer five because it is not in its dependency list. This is stronger than convention-based module boundaries because the compiler enforces it.

## How the development tools work

Rust has a rich ecosystem of development tools that automate testing, checking, and publishing. Understanding these tools is essential for maintaining a nine-crate workspace.

cargo fmt formats your code according to a standard style. It ensures all contributors write code that looks the same, eliminating debates about indentation, spacing, and brace placement. In CI, you run it in check mode, which reports files that are not formatted without modifying them.

cargo clippy is the Rust linter. It checks for common mistakes, suboptimal patterns, and style issues. It has hundreds of lint rules organized into groups: correctness (catches definite bugs), style (enforces conventions), complexity (suggests simplifications), and pedantic (enforces stricter rules that are optional). For starsight, clippy runs in pedantic mode with a few noisy lints selectively allowed.

cargo test runs all tests in the workspace: unit tests (in the same files as the code they test), integration tests (in the tests directory), and doc tests (in documentation comments). For starsight, this includes unit tests for every method on the primitive types, snapshot tests for rendered chart output, and property tests for mathematical invariants.

cargo-deny checks dependencies for license compliance, security vulnerabilities, duplicate versions, and untrusted sources. For starsight's GPL-3.0 license, the allow list includes MIT, Apache-2.0, BSD, ISC, Zlib, CC0, and other GPL-compatible licenses. Cargo-deny runs on every pull request and fails if a new dependency introduces an incompatible license.

cargo-insta provides snapshot testing. You render a chart to PNG bytes or an SVG string, call an assertion macro, and insta stores the result as a reference file. On subsequent runs, insta compares against the stored reference. If anything changed, the test fails. This catches visual regressions: subtle changes in layout, color, text positioning, or rendering that would be invisible to numerical assertions.

cargo-semver-checks compares your public API against the last published version on crates.io and reports any breaking changes. It catches removed public items, changed function signatures, removed trait implementations, and other API incompatibilities. Run it on every pull request to prevent accidental breaking changes.

cargo-llvm-cov measures code coverage: which lines of code are executed by your tests. It instruments the compiled code to count executions, then maps the counts back to source lines. For starsight, the coverage target is at least 80 percent of library code, with 100 percent coverage on the mathematical core (scales, ticks, coordinates, color conversion).

cargo-nextest is a faster test runner that executes each test as a separate process, enabling true parallelism across CPU cores. It provides better output formatting and built-in flaky test retry. It does not run doc tests, so you must combine it with cargo test dash-dash-doc in CI.

criterion is a benchmarking framework that runs code many times, collects timing measurements, performs statistical analysis, and detects performance regressions. For starsight, benchmarks measure rendering time at various data scales (100, 1000, 10000, 100000 points) to track performance characteristics.

cargo-flamegraph generates visual profiles showing where your program spends CPU time. It samples the call stack at high frequency and aggregates the results into a hierarchical chart. Wide bars indicate functions consuming significant CPU time. For starsight, typical hotspots are in tiny-skia's path rasterization and alpha blending.

git-cliff generates changelogs from conventional commit messages. If your commits follow the format (feat colon add line chart, fix colon correct axis inversion, etc.), git-cliff automatically groups them into Added, Fixed, Changed sections for each release.

cargo-hack tests feature flag combinations. The each-feature mode verifies each feature compiles independently. The feature-powerset mode tests combinations of features. This catches bugs where feature A accidentally depends on code only compiled when feature B is enabled.

## How GitHub Actions CI works for Rust projects

GitHub Actions is a continuous integration system that runs automated checks on every pull request and push. For starsight, the CI pipeline has several jobs that run in parallel.

The fmt job runs cargo fmt in check mode, verifying code formatting. The clippy job runs cargo clippy with warnings treated as errors. The check job runs cargo check across the workspace with all features enabled. The test job runs cargo test across a matrix of three operating systems (Linux, macOS, Windows) and three Rust versions (stable, beta, and 1.85 which is the minimum supported version). The deny job runs cargo-deny to check licenses and vulnerabilities. The snapshots job runs cargo insta test in check mode to catch visual regressions.

Each job starts by checking out the code, installing the Rust toolchain (using the dtolnay rust-toolchain action which reads the rust-toolchain.toml file), and restoring the dependency cache (using the Swatinem rust-cache action which caches the target directory between runs).

The test matrix ensures starsight compiles and passes tests on all supported platforms and Rust versions. If a new language feature accidentally enters the code that requires a newer Rust version than the declared minimum, the oldest toolchain in the matrix will fail, catching the regression.

Additional workflows run on different schedules. Coverage runs weekly, generating an LCOV report and uploading it to Codecov. The gallery workflow runs when examples change, generating rendered chart images for documentation. The release workflow runs when a version tag is pushed, publishing all crates to crates.io in dependency order and creating a GitHub release with generated changelog notes.

## How to publish crates to crates.io

crates.io is the Rust package registry. When you publish a crate, it becomes available for anyone to depend on. Publishing is permanent: a version can never be overwritten. You can yank a version (preventing new projects from depending on it) but you cannot delete it.

For starsight's nine crates, publishing must happen in dependency order: layer-1 first (it has no internal dependencies), then layer-2 (depends on layer-1), then layer-3, and so on up to the facade crate. Between each publish, there is a propagation delay (10 to 30 seconds) while the crates.io index updates. The cargo-release tool automates this entire process.

Each crate's Cargo.toml must specify both a path and a version for workspace dependencies. During development, Cargo uses the path (pointing to the local directory). During publication, Cargo strips the path and uses the version (requiring the dependency to be published on crates.io). If you publish layer-2 before publishing layer-1, the publish fails because layer-1 is not yet available on crates.io at the required version.

Pre-1.0 semver has a specific meaning in Rust. For version 0.x.y, x is the "major" version and y is the "minor" version. Bumping from 0.1.0 to 0.2.0 signals a breaking change. Bumping from 0.1.0 to 0.1.1 signals a compatible change. A user who depends on "0.1" (which Cargo interprets as at-or-above 0.1.0 and below 0.2.0) will automatically receive 0.1.1 but not 0.2.0.

## How to design APIs that survive years of evolution

A library's API is its contract with users. Changing the API breaks their code. For starsight, which will evolve through many versions before reaching 1.0, API design decisions made early have long-lasting consequences.

The most important technique is the non-exhaustive attribute. When you put non-exhaustive on an enum, downstream code must include a wildcard arm in match statements. This means you can add new enum variants in future versions without breaking existing code. When you put non-exhaustive on a struct, downstream code cannot construct it with struct literal syntax. This means you can add new fields in future versions without breaking existing code.

Apply non-exhaustive to: error enums (you will add new error variants), configuration structs (you will add new options), and any enum of chart types or scale types. Do not apply it to pure mathematical types like Point, Vec2, Rect, and Color, where the fields are the complete definition.

The builder pattern is the second key technique. Instead of constructors with many parameters, provide a struct with a new method (taking only required parameters) and chainable setter methods for optional parameters. Adding a new setter is never a breaking change. The builder methods take ampersand mut self and return ampersand mut Self, enabling chaining: figure dot title of "Chart" dot x-label of "Time" dot size of 800, 600.

Accept the most general input types. For string parameters, accept impl Into of String (which lets callers pass string literals, owned Strings, or anything that converts to String). For slice parameters, accept ampersand slice of T instead of Vec of T (which avoids requiring callers to allocate). For color parameters, accept impl Into of Color (which lets callers pass tuples, hex values, or Color values directly).

Never expose dependency types in the public API. If your DrawBackend trait takes a tiny-skia Point, your API is coupled to tiny-skia's versioning. When tiny-skia releases a breaking change, your API breaks too. Wrap external types in your own types and convert internally.

Derive Debug and Clone on every public type. Debug is needed for useful test failure messages. Clone is needed by users who want to modify copies of configurations.

Use the question mark operator to propagate errors. Never use unwrap or expect in library code (they panic, crashing the user's program). Never use println or eprintln (they produce unwanted output). Use the log crate for diagnostic messages that the user can opt into.

Every public item should have a documentation comment. Every function should document what it does, what the parameters mean (including units), and what errors it can return. Documentation comments are also tests: code blocks in doc comments are compiled and run by cargo test, so they must be correct.

## How to test a visualization library

Testing a visualization library is different from testing a data processing library because the output is visual. You cannot assert that a chart "looks correct" with a numerical comparison. You need different testing strategies for different kinds of bugs.

Unit tests verify individual computations. Test that a linear scale maps the domain minimum to the range minimum and the domain maximum to the range maximum. Test that the tick algorithm produces monotonically increasing values. Test that color conversion round-trips correctly. These tests are fast, deterministic, and catch logic errors in the mathematical foundations.

Snapshot tests verify visual output. Render a chart to PNG bytes using the deterministic CPU backend, pass the bytes to insta's binary snapshot assertion, and store the result as a reference file. When the code changes, any visual difference causes a test failure. A human reviews the old and new images to decide whether the change is intentional. This catches layout regressions, color changes, text positioning errors, and rendering bugs.

Property tests verify mathematical invariants across random inputs. Use the proptest library to generate random data and check that properties hold: scale forward then inverse returns the original value (roundtrip), tick positions are strictly increasing (monotonicity), the plot area is always smaller than the figure dimensions (containment), and rendering never panics regardless of input (robustness). Property tests catch edge cases that hand-written test cases miss.

Integration tests verify the full pipeline. Create data, build a Figure, render to a file, and check that the file exists and has reasonable content. These tests exercise the user-facing API and catch integration issues between layers.

For starsight, snapshot tests are the most important category. Every chart type should have at least one snapshot test. The snapshot files are committed to version control and reviewed as part of every pull request that changes visual output.

## How floating-point arithmetic affects chart rendering

Floating-point numbers (f32 and f64) are not exact. The value 0.1 cannot be represented exactly in binary floating point, just as one-third cannot be represented exactly in decimal. This means computations that you expect to produce clean round numbers sometimes produce values like 0.30000000000000004 instead of 0.3.

For starsight, this affects three areas. First, scale mapping: mapping a value through a linear scale and back through the inverse should return the original value, but floating-point rounding may introduce tiny errors. Tests should use approximate comparison (check that the difference is less than a small tolerance) rather than exact equality.

Second, tick label formatting: the tick algorithm might produce the value 0.30000000000000004 instead of 0.3. The label formatter must handle this by rounding to a reasonable number of decimal places based on the tick step size.

Third, coordinate conversion: accumulated rounding errors through the scale, axis, and coordinate mapping pipeline can cause pixels to be off by one from the expected position. This is usually invisible but can cause subtle alignment issues in grid lines and axis tick marks.

The special float values NaN (Not a Number) and infinity require careful handling. NaN propagates through arithmetic: any operation involving NaN produces NaN. NaN does not equal itself, which means standard comparisons fail. Infinity represents overflow. For starsight, NaN in input data should produce gaps in line charts (not crashes or garbled output), and infinities should be treated as invalid data.

## How the entire build and development workflow fits together

When you start working on starsight, the workflow is: edit code in your editor, save the file, run cargo check to verify it compiles (this is fast because it skips code generation), run cargo test to verify correctness (this runs all tests including snapshot tests), and commit when everything passes.

For rapid feedback, run bacon (a terminal UI tool) in one terminal. It watches for file changes and automatically runs cargo check or cargo clippy, showing errors and warnings instantly as you type.

When preparing a pull request, run the full CI check locally: cargo fmt to format code, cargo clippy to check for lint issues, cargo test to run all tests, and cargo deny check to verify dependencies.

The CI pipeline runs automatically on every push and pull request. It checks formatting, linting, compilation on multiple platforms, tests on multiple Rust versions, dependency compliance, and snapshot integrity. If all checks pass, the pull request is ready for review.

For releases, cargo-release handles the entire workflow: bumping version numbers in all nine Cargo.toml files, creating a git commit and tag, and publishing all crates to crates.io in the correct dependency order. git-cliff generates the changelog from commit messages. The GitHub Actions release workflow creates a GitHub release with the generated notes.

## What to implement first

The exit criteria for version 0.1.0 is: calling the plot macro with two arrays and then calling save with a PNG file path produces a correct line chart image. This is the minimum vertical slice that proves the architecture works.

To get there, you need: the primitive types (Point, Vec2, Rect, Size, Color, Transform) in layer one, the tiny-skia backend (creating a Pixmap, filling rectangles, drawing paths, rendering text, encoding PNG) in layer one, a linear scale in layer two, the Wilkinson tick algorithm in layer two, a Cartesian coordinate system in layer two, axis rendering (tick lines, tick labels, axis labels) using layers one and two, a line mark in layer three, the Figure builder in layer five, the plot macro in layer five, and snapshot tests proving it all works.

You do not need: log scales, bar charts, histograms, faceting, legends, GPU rendering, terminal rendering, interactivity, streaming data, PDF export, WASM, Polars integration, or any chart type beyond basic lines and points. These come in later milestones.

Resist the temptation to add features before the vertical slice is complete. A library that renders one chart type correctly and has tests is more valuable than a library with stubs for sixty chart types that renders nothing.

Start with the primitive types. Add Vec2 with semantic arithmetic: Point minus Point gives Vec2, Point plus Vec2 gives Point, Point plus Point does not compile. Add Rect accessors and constructors. Add Color with hex parsing, luminance, contrast ratio, and lerp. Write tests for everything.

Then implement the tiny-skia backend. Create the SkiaBackend struct wrapping a Pixmap. Implement fill-rect, draw-path, save-png. Set up snapshot testing with insta: render a blue rectangle on a white background and verify the PNG bytes match.

Then implement the linear scale and tick algorithm. Then the Cartesian coordinate system. Then the line mark. Then the Figure builder and plot macro. Then the integration test that calls plot, save, and checks the output.

Each step is a commit. Each commit leaves the codebase compiling and tests passing. The git history tells the story of steady forward progress. And when the first PNG file appears on disk with a recognizable line chart, you have proven that seven crates, a rendering backend, a text engine, a scale system, and a macro all work together correctly. Everything after that is adding chart types, backends, and features to a foundation you know is solid.

## How to stay motivated over a multi-year project

Building a comprehensive visualization library takes years, not weeks. The scope is enormous. Sixty-six chart types, five rendering backends, GPU acceleration, terminal rendering, WASM, interactivity, streaming, 3D, animation. This scope is achievable but it requires sustained effort over a long timeline.

The key is momentum. Commit every day, even if it is just a small refactor or a single test. Each commit is visible progress. Focus on the vertical slice first: get plot-save to produce a PNG. Then add a second chart type. Then add axis labels. Then add colors. Each addition is a visible improvement you can share.

Do not worry about later milestones until the foundation is solid. The layer architecture ensures later work adds to the codebase without restructuring it. You can add GPU rendering at version 0.6.0 without touching the marks, scales, or figure code that has been stable since 0.1.0.

Share your progress publicly. Post screenshots of rendered charts. Write about the tick algorithm or the tiny-skia integration. Show benchmark comparisons. Public visibility attracts contributors, creates accountability, and sustains motivation across the long timeline.

Accept that the first version will not be perfect. The 0.1.0 line chart will not have perfect text positioning, optimal margin computation, or pixel-perfect anti-aliasing. It needs to produce a recognizable line chart from user data and save it to a file. Perfection comes through iteration, not through getting everything right the first time.

The Rust visualization ecosystem has seen many abandoned efforts: crates that published a 0.1.0, received attention, and went silent. The antidote is sustainable pace. One meaningful commit per day, one release per month, one blog post per quarter. Consistent pace matters more than sprint speed.

The first PNG with a recognizable chart is the proof that the architecture works. Everything after that is incremental improvement. And the Rust ecosystem is waiting for a library exactly like this one. The first comprehensive Rust visualization library to reach maturity will become the default choice, just as matplotlib became the default for Python. starsight aims to be that library.

## How Cargo workspaces organize a multi-crate project

A Cargo workspace is a collection of crates that share a single Cargo.lock file and a single output directory. The workspace is defined by a root Cargo.toml that lists the member crates. All members are compiled together, share dependency versions, and can depend on each other.

For starsight, the workspace has nine members: the starsight facade crate, starsight-layer-1 through starsight-layer-7, and the xtask binary crate. The root Cargo.toml defines workspace-level settings that all members inherit.

Workspace inheritance lets you define common metadata once. The version, edition, license, authors, repository, and other fields are defined in the root Cargo.toml under the workspace.package section. Each member crate's Cargo.toml can inherit these fields by writing "version.workspace equals true" instead of a literal version string. This ensures all crates share the same version number, which simplifies releases.

Dependency versions are also centralized. The root Cargo.toml has a workspace.dependencies section that lists every external dependency with its version. Member crates reference these with "tiny-skia.workspace equals true" instead of specifying the version directly. This prevents the situation where one crate uses tiny-skia 0.11 and another uses 0.12.

Workspace lints provide consistent coding standards. The root Cargo.toml has a workspace.lints section that configures clippy and rustc lints. Each member crate opts in with "lints.workspace equals true." For starsight, the workspace lints forbid unsafe code and enable clippy pedantic at the warn level.

Profile settings (debug and release build configurations) are always workspace-level. You cannot have different optimization levels for different member crates within the same profile. For starsight, the release profile enables link-time optimization (LTO) and sets codegen-units to 1 for maximum optimization. The dev profile sets opt-level to 1 (instead of the default 0) because tiny-skia performs poorly at opt-level 0.

The xtask crate is a special member that is never published. It is a binary that provides development automation: generating the gallery, running benchmarks, preparing releases. The xtask pattern uses a Cargo alias so that "cargo xtask gallery" runs the xtask binary with the "gallery" argument.

## How the Rust toolchain works: rustup, rustc, and Cargo

rustup is the Rust toolchain manager. It installs Rust compilers (rustc), the package manager (Cargo), the formatter (rustfmt), the linter (clippy), and other tools. It manages multiple toolchain versions (stable, beta, nightly, specific version numbers) and can switch between them.

The rust-toolchain.toml file at the project root tells rustup which toolchain to use. When anyone runs any Cargo command in the project directory, rustup reads this file, checks if the specified toolchain is installed, downloads it if necessary, and uses it. This ensures every developer and every CI runner uses the same Rust version.

For starsight, the rust-toolchain.toml specifies Rust version 1.85 (the first version with edition 2024 support) and the components rustfmt, clippy, and llvm-tools-preview. The llvm-tools-preview component is needed for code coverage measurement.

rustc is the Rust compiler. You rarely invoke it directly. Cargo calls it for you, passing the right flags for the current crate, its dependencies, and the build profile. The compilation pipeline goes through several stages: parsing (text to abstract syntax tree), name resolution and type checking (on the high-level intermediate representation, HIR), borrow checking and optimization (on the mid-level intermediate representation, MIR), and finally code generation (converting MIR to LLVM IR, then to machine code via LLVM).

Cargo reads Cargo.toml and Cargo.lock, resolves all dependencies, and orchestrates the compilation of every crate in the right order. Key commands you will use daily: cargo check (type-checks without generating code, much faster than build), cargo build (compiles the code), cargo test (compiles and runs tests), cargo clippy (runs the linter), cargo fmt (formats the code), and cargo doc (generates documentation).

## How to use clippy effectively

clippy is the official Rust linter. It has over 700 lint rules organized into groups: correctness (catches definite bugs, default deny), suspicious (catches code that is likely wrong, default warn), style (enforces idiomatic Rust style, default warn), complexity (suggests simplifications, default warn), perf (suggests performance improvements, default warn), pedantic (enforces stricter style rules, off by default), nursery (experimental lints that may have false positives, off by default), and restriction (opinionated lints that are too strict for most projects, off by default).

starsight enables the pedantic group at the warn level. This catches many issues that the default groups miss: missing documentation, overly complex expressions, unnecessary closures, functions that should return references instead of clones. However, pedantic is noisy. Some of its lints fire frequently and are not useful for your specific codebase. The way to handle this is to allow specific lints either in the workspace lint configuration (for lints that are universally noisy) or with per-item attributes (for specific exceptions).

Common pedantic lints to allow: module_name_repetitions (fires when a type name contains the module name, like scale::LinearScale), must_use_candidate (fires on every function that returns a value, suggesting you add the must_use attribute), and missing_errors_doc (fires on every function returning Result, requesting documentation of the error conditions). These are reasonable lints in principle but produce too many warnings to be useful.

Run clippy with "cargo clippy minus minus workspace" to check all crates. In CI, add "minus D warnings" to treat all warnings as errors, ensuring the CI fails if any lint fires.

## How to use cargo-deny for dependency governance

cargo-deny checks your dependencies against four categories of rules: licenses, security advisories, crate bans, and source registries.

For a GPL-3.0 project like starsight, license checking is critical. Every dependency must have a license that is compatible with GPL-3.0. The compatible licenses include: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0-1.0, Unlicense, MPL-2.0, LGPL variants, and GPL variants. Incompatible licenses include proprietary licenses, SSPL (Server Side Public License), and AGPL-3.0 (which has an additional network clause that is more restrictive than GPL).

The configuration lives in deny.toml at the workspace root. The licenses section has an allow list of SPDX license identifiers. A common gotcha: the unicode-ident crate (a transitive dependency of nearly every Rust project) uses the Unicode-DFS-2016 license, which must be explicitly added to the allow list. Another gotcha: the ring cryptography crate has a complex license file mixing ISC, MIT, and OpenSSL that cargo-deny cannot parse automatically. You need a clarify block that manually specifies the license.

The advisories check queries the RustSec advisory database for known vulnerabilities in your dependency tree. You can ignore specific advisories that do not affect your use case (with documented reasons).

The bans check prevents specific crates from entering your dependency tree. You can also configure the multiple-versions policy to deny having two different versions of the same crate (which increases binary size).

In CI, the recommended pattern separates the advisory check from the other checks because new advisories can appear at any time and break unrelated PRs. Use a matrix strategy with continue-on-error on the advisory job.

## How snapshot testing works with cargo-insta

Snapshot testing is the primary mechanism for catching visual regressions in starsight. You render a chart, convert it to bytes (PNG) or a string (SVG), and compare it against a stored reference.

The insta crate provides assertion macros for snapshot testing. The binary snapshot macro takes an extension (like ".png") and a Vec of bytes. On the first run, it creates a reference file. On subsequent runs, it compares the new output against the reference byte-by-byte. If anything changed (even one pixel), the test fails.

The workflow has three commands. cargo insta test runs all tests and creates pending files (with a ".snap.new" extension) for any mismatches. cargo insta review opens an interactive terminal interface where you see the old and new outputs and can accept or reject each change. cargo insta accept bulk-accepts all pending changes.

In CI, you run cargo insta test with the check flag, which fails immediately on any mismatch (no pending files). This catches regressions automatically.

For starsight, snapshot tests are deterministic because the tiny-skia backend is CPU-only: the same inputs always produce the same pixels. There is no GPU floating-point variance, no driver-dependent rounding. If a snapshot test fails, something in the code actually changed.

Set up snapshot testing before writing any chart code. The very first test should render a simple rectangle and take a snapshot. This validates the entire pipeline from backend creation to PNG encoding.

## How cargo-semver-checks catches API breakage

When you publish a new version of a crate, you promise that users depending on the old version can upgrade without their code breaking (for minor version bumps) or that they accept some breakage (for major version bumps). Accidentally breaking the API in a minor version is one of the most embarrassing mistakes a library author can make.

cargo-semver-checks automatically detects breaking changes by comparing your current public API against the last published version. It works by analyzing rustdoc JSON output using a graph query language. It can detect over 120 categories of breakage: removed public items, changed function signatures, removed trait implementations, added required trait methods, and many more.

Run it with cargo semver-checks on every PR, not just before publishing. The earlier you catch an accidental break, the easier it is to fix.

What it does not catch: changes to parameter types (changing u32 to u64), changes to generic parameters, behavioral changes (a function that returns different values for the same inputs), and visual output changes (a chart that looks different with the same data). These limitations are why snapshot testing and manual review exist alongside cargo-semver-checks.

## How cargo-llvm-cov measures code coverage

Code coverage answers the question: which lines of code are executed by your tests? For starsight, this reveals which rendering paths, scale computations, and error handling branches are exercised and which are not.

cargo-llvm-cov uses LLVM's source-based instrumentation. When you run it, it adds hidden counters to every branch point and expression in the compiled code. When tests run, the counters record which code was executed and how many times. After the tests, the tool generates a report mapping the counter data back to source lines.

Run it with cargo llvm-cov for the workspace. The output can be in LCOV format (for uploading to Codecov or Coveralls), HTML format (for browsing locally), or text format (for the terminal). Test code is excluded by default.

For starsight, coverage is most useful for identifying untested error handling paths and unused backend code. If the SVG backend has zero coverage, that means no tests exercise SVG output, and bugs there will go unnoticed.

## How to use proptest for property-based testing

Regular unit tests check specific examples: "the linear scale maps 50 to 400 pixels." Property-based tests check general properties: "for any input value within the domain, mapping through the scale and then through the inverse scale returns the original value." proptest generates random inputs, checks the property, and when it finds a failure, automatically shrinks the input to the smallest case that still fails.

For starsight, the most valuable property tests cover mathematical invariants. Scale roundtrip: for any value, forward then inverse mapping returns the original value within floating-point tolerance. Tick monotonicity: for any data range and target count, the tick positions are strictly increasing. Coordinate mapping endpoints: the domain minimum maps to the plot area left edge, and the domain maximum maps to the right edge.

A practical detail: avoid using the default f64 generator, which includes NaN, infinity, and subnormal values. These produce nonsensical results for scales and coordinates. Use bounded ranges like minus one million to one million instead.

When proptest finds a failing input, it stores it in a regression file. Commit these files to version control so that the failing case is always re-tested, even if proptest's random seed changes.

## How to use criterion for benchmarking

criterion is a statistical benchmarking framework that measures execution time, performs statistical analysis to detect regressions, and generates HTML reports.

For starsight, the critical benchmarks measure rendering performance at different data scales. How long does it take to render a line chart with 100 points? 1,000? 10,000? 100,000? This scaling behavior determines whether starsight is practical for real-world datasets.

Set up benchmarks by creating a bench file with criterion's main macro, then define benchmark functions that call your rendering code inside criterion's iter method. The iter method runs the code many times and collects timing data. criterion handles warm-up, measurement, statistical analysis, and comparison against previous runs.

Use the throughput feature to report per-element cost. Declaring the throughput as the number of data points lets criterion report "5 microseconds per point" instead of just "50 milliseconds total," making scaling behavior explicit.

## How to use cargo-flamegraph to find performance bottlenecks

A flamegraph is a visualization of where your program spends its CPU time. The horizontal axis represents proportion of total time. The vertical axis represents the call stack. Wide boxes at the top consume the most CPU time directly.

Install cargo-flamegraph and enable debug symbols in the release profile ("debug equals true"). Create a profiling example that generates a realistic workload (like rendering a scatter plot with 100,000 points), then run cargo flamegraph with the example.

For tiny-skia rendering, typical hotspots are the fill_path and stroke_path functions (path rasterization), the alpha blending pipeline (compositing), and text rendering (glyph rasterization). If fill_path dominates, simplify paths by reducing the number of segments. If blending dominates, reduce the number of overlapping semi-transparent elements.

## What every file in the starsight workspace does

The workspace root contains: Cargo.toml (workspace definition, shared metadata, shared dependencies, profiles), deny.toml (cargo-deny configuration), .clippy.toml (clippy-specific settings), .rustfmt.toml (formatting rules), LICENSE (GPL-3.0 text), README.md (public-facing description), CONTRIBUTING.md (contributor guide), CHANGELOG.md (version history), and CODE_OF_CONDUCT.md.

The .github directory contains: CI workflows (ci.yml, release.yml, coverage.yml, snapshots.yml, gallery.yml), issue templates (bug report, feature request), a pull request template, and a funding configuration.

The starsight directory is the facade crate: lib.rs re-exports types from all layer crates, and prelude.rs provides convenient imports for common usage.

starsight-layer-1 is the largest crate. Its src directory contains: lib.rs (module declarations), primitives.rs (Point, Vec2, Rect, Size, Color, Transform), error.rs (StarsightError enum), and the backend directory with sub-modules for skia, svg, pdf, wgpu, and terminal backends.

starsight-layer-2 through starsight-layer-7 are currently mostly empty, with just lib.rs files. They will be filled in as each milestone is implemented.

The xtask directory contains the development automation binary.

The examples directory contains empty example files that will be filled in as chart types are implemented: quickstart.rs, scatter.rs, statistical.rs, terminal.rs, interactive.rs, and others.

## Things you must do when writing starsight code

Derive Debug on every public type. When an assertion fails, the failure message includes the Debug output of the compared values. Without Debug, the message is useless.

Derive Clone on every public type. Users will want to create a chart configuration, modify it slightly, and render both versions. Clone enables this.

Use the question mark operator for all error propagation. Never use unwrap or expect in library code. If an operation can fail, return a Result and let the caller decide what to do with the error.

Write doc comments on every public item. Use the warn missing_docs lint to enforce this. Doc comments are also tests: code examples in doc comments are compiled and run as part of cargo test.

Use the non_exhaustive attribute on every public enum and configuration struct. This lets you add new variants and fields in future versions without breaking downstream code.

Accept impl Into of String for all string parameters. This lets callers pass string literals without calling to_string.

Accept impl Into of Color for all color parameters. This lets callers pass colors from sister crates (chromata, prismatica) without explicit conversion.

Keep DrawBackend object-safe. Never add a Sized bound, never add generic type parameters to trait methods, never return Self from trait methods.

Test every chart type with a snapshot test before considering the implementation complete.

Use feature flags for every optional dependency. The default build should compile quickly with minimal dependencies.

## Things you must not do when writing starsight code

Do not use unwrap or expect in library code. These panic on failure, crashing the caller's program. Use the question mark operator instead.

Do not use println or eprintln in library code. Library code should be silent by default. Use the log crate for diagnostic messages.

Do not expose dependency types in the public API. Wrap tiny-skia, cosmic-text, and other dependency types in your own types. This prevents dependency version changes from breaking your public API.

Do not use unsafe code in layers three through seven. Only layer one (the rendering abstraction) might need unsafe for performance-critical pixel manipulation, and even there it should be avoided.

Do not use async in the public API. starsight is CPU-bound, not I/O-bound. Async adds complexity without benefit.

Do not add JavaScript or C dependencies in the default feature set. One of starsight's key differentiators is being pure Rust with no foreign dependencies.

Do not add nightly-only features as requirements. starsight must compile on stable Rust.

Do not create deep module nesting. Three levels is the maximum (crate, module, sub-module). Deeper nesting makes code hard to navigate.

Do not use global re-exports with "pub use crate star." List every re-exported type explicitly. Glob re-exports hide the origin of types and cause namespace pollution.

Do not derive Default on types where the default is not useful. A default Figure with no data produces an empty chart, which is never what anyone wants. Force users to call a constructor that requires meaningful parameters.

## How to write good error messages

Error messages are documentation. When something goes wrong, the error message is often the only information the user has.

A bad error message: "rendering failed." This tells the user nothing about what happened or how to fix it.

A good error message: "failed to create pixmap at 800 by 600: out of memory. Try reducing the chart dimensions or the DPI setting." This tells the user what happened (pixmap creation failed), why (out of memory), and what to do about it (reduce dimensions or DPI).

Write error messages in lowercase, without trailing periods, and with enough context to identify the failed operation. Include variable values when helpful: "scale domain is empty: min and max are both 5.0" is much more informative than "invalid scale domain."

## How to handle NaN values throughout the pipeline

NaN (Not a Number) is a floating-point value that represents undefined results like zero divided by zero or the square root of a negative number. NaN has a unique property: it is not equal to itself. The expression NaN double-equals NaN is false. NaN also propagates through arithmetic: any operation involving NaN produces NaN.

starsight must handle NaN gracefully because real-world data often contains missing values represented as NaN. Sensor readings with gaps, database columns with nulls converted to floating-point, and computed values that produce division by zero all result in NaN.

The design principle is: NaN should never cause a panic, a garbled chart, or an infinite loop. It should produce a visible gap in the chart and optionally a warning.

For scales, NaN input should produce NaN output. The map method should not crash on NaN. The domain computation (finding the minimum and maximum) should skip NaN values.

For line marks, NaN should produce a gap. When the LineMark encounters a NaN value, it starts a new sub-path (a new MoveTo) at the next non-NaN value. This breaks the line at the gap.

For scatter marks, NaN should skip the point. No circle is drawn for a data point with NaN coordinates.

For the tick algorithm, NaN in the data range should not cause an infinite loop. The scoring function compares values, and NaN comparisons always return false, which can prevent the loop from terminating if not guarded.

Test NaN handling explicitly in every component. A property test that feeds NaN to every public function and asserts that no panic occurs is one of the highest-value tests in the suite.

## How to handle empty data

Empty data (zero-length arrays) is valid input. A user might create an empty Figure, or filter data down to zero rows. Every component must handle this gracefully.

Scales with empty data should use a default domain (like 0 to 1). Marks with empty data should render nothing (return Ok without drawing). The tick algorithm with a zero-range domain (min equals max) should return a pair of ticks at sensible positions. An empty Figure should produce a chart with axes but no data content.

## How to think about the 0.1.0 milestone

The exit criteria for version 0.1.0 is: calling plot with two arrays and then save with a PNG file path produces a correct line chart image. This is a narrow vertical slice that proves the architecture works.

To get there, you need: Point, Vec2, Rect, Color, Transform types (layer one); SkiaBackend with fill, fill_rect, draw_path, draw_text, save_png (layer one); SvgBackend with fill_rect, save_svg (layer one); LinearScale with map and inverse (layer two); the Wilkinson Extended tick algorithm (layer two); Axis combining a scale with ticks and labels (layer two); CartesianCoord mapping data to pixels with Y inversion (layer two); the Mark trait (layer three); LineMark with NaN gap handling (layer three); PointMark with batched circles (layer three); Figure builder with title, x_label, y_label, size, add, save (layer five); the plot macro (layer five); and snapshot tests.

You do not need: log scales, categorical scales, bar charts, histograms, box plots, faceting, legends, GPU rendering, terminal rendering, interactivity, streaming, PDF, WASM, Polars, ndarray, Arrow, 3D, or any of the 66 chart types beyond lines and points.

Resist the temptation to add features before the vertical slice is complete. A library that renders one chart type correctly, with tests, is more valuable than a library with stubs for sixty chart types and no working output.

## How to approach the implementation in your first coding session

Open starsight-layer-1/src/primitives.rs. This file already has Color, Point, Rect, and Size. Your first task is to add Vec2.

Write the Vec2 struct with two f32 fields, x and y. Add the Debug, Clone, Copy, PartialEq, and Default derives. Add a new constructor. Add the ZERO, X, and Y constants. Add the length and normalize methods. Write a test. Run cargo test. See the test pass. Commit.

Then add the arithmetic: Point minus Point gives Vec2. Point plus Vec2 gives Point. Vec2 plus Vec2 gives Vec2. Vec2 times f32 gives Vec2. Write a test for each. Commit.

Then move to Rect: add from_xywh, from_center_size, width, height, center, contains, intersection, pad, to_tiny_skia. Write tests. Commit.

Then Color: add constants (BLACK, WHITE, RED, GREEN, BLUE), from_css_hex, to_css_hex, luminance, contrast_ratio, lerp, to_tiny_skia. Write tests. Commit.

Then Transform: a newtype wrapping tiny_skia Transform with identity, translate, scale, rotate_degrees, then, pre_translate methods. Commit.

Then the SkiaBackend: a struct holding a tiny_skia Pixmap, FontSystem, and SwashCache. Implement new, fill, fill_rect, draw_path, save_png. Write a snapshot test. Commit.

This rhythm (one type or method at a time, tests alongside implementation, frequent small commits) is sustainable over months and keeps the codebase in a working state at every point.

## How testing should evolve as the project grows

At 0.1.0: unit tests for every method on primitive types. Unit tests for linear scale. Unit tests for the tick algorithm. One snapshot test for a rendered line chart. One snapshot test for SVG output.

At 0.2.0: snapshot tests for bar, area, histogram, and heatmap charts. Property tests for scale roundtrips.

At 0.3.0: snapshot tests for all statistical chart types. Reference comparisons with matplotlib output for the same data.

At 0.4.0: layout tests verifying faceted charts have the correct panel count. Legend tests. Colorbar tests.

At 0.5.0: property tests for all scale types. Edge case tests for symlog near zero, log with very small values, datetime scale with daylight saving transitions.

At each milestone, coverage should stay above 80 percent. Run cargo-mutants periodically to find code paths where tests exist but do not actually verify behavior.

## How the CI pipeline catches bugs automatically

The CI pipeline runs on every pull request and push to main. It checks:

Formatting: cargo fmt minus minus check verifies that all code is formatted according to the project's rustfmt rules. If any file differs, the check fails.

Linting: cargo clippy with all warnings treated as errors catches style issues, common mistakes, and potential bugs.

Compilation: cargo check verifies that the code compiles. This catches syntax errors and type errors.

Testing: cargo test runs all unit tests, integration tests, and doc tests. The test matrix includes Linux, macOS, and Windows, and two Rust versions (stable and the MSRV 1.85). This catches platform-specific bugs and MSRV violations.

Snapshot testing: cargo insta test with check mode fails if any snapshot does not match. This catches visual regressions.

Dependency auditing: cargo-deny checks licenses, security advisories, banned crates, and source registries.

Coverage: cargo-llvm-cov generates coverage reports and uploads them to Codecov. This tracks coverage trends over time.

All of these checks are already configured in the starsight repository's GitHub Actions workflows.

## How version control and conventional commits work

starsight uses conventional commits for its git history. Every commit message starts with a type, an optional scope, and a description. The type determines how the commit is categorized in the changelog and how version bumps are calculated.

The type "feat" indicates a new feature. "fix" indicates a bug fix. "perf" indicates a performance improvement. "refactor" indicates a code change that neither fixes a bug nor adds a feature. "docs" indicates a documentation change. "test" indicates a test addition or modification. "chore" indicates a maintenance task (like updating dependencies). An exclamation mark after the type (or a "BREAKING CHANGE" footer) indicates a breaking change.

The scope is the area of the codebase affected. For starsight, useful scopes include: layer-1, layer-2, primitives, scale, tick, skia, svg, and ci.

The description is imperative mood, lowercase, no period. "add linear scale support" not "added linear scale support." The description should complete the sentence "this commit will" followed by the description.

git-cliff reads the commit history and generates a changelog by grouping commits by type and formatting them according to a template. The release workflow uses git-cliff to generate release notes.

## How to stay motivated on a multi-year project

Building a comprehensive visualization library takes years, not months. The scope is enormous: 66 chart types, 5 rendering backends, GPU acceleration, terminal rendering, WASM, interactivity, streaming, 3D, animation.

The key is momentum. Commit every day, even if it is small. Each commit is visible progress. The git history tells the story of continuous forward motion.

Focus on the vertical slice first. Get one chart type rendering correctly. Then add a second. Then add axis labels. Then add colors. Each addition is a visible improvement you can share.

Share your progress publicly. Post screenshots of rendered charts. Write about the tick algorithm or the tiny-skia integration. Public visibility attracts contributors and creates accountability.

Do not worry about the later milestones until the foundation is solid. The layer architecture ensures that later work builds on top of early work without restructuring it.

Accept that early code will be imperfect. The 0.1.0 line chart does not need perfect text positioning or optimal margins. It needs to produce a recognizable chart from user data. Perfection comes through iteration.

## How starsight fits into the broader Rust ecosystem

The Rust data science ecosystem is maturing rapidly. Polars rivals pandas. ndarray is stable. Arrow is standardized. statrs provides statistics. The missing piece is visualization. starsight aims to fill that gap.

The closest competitors (plotters, plotly-rs, charming) each have fundamental limitations. plotters' Sized bound prevents dynamic dispatch. plotly-rs requires JavaScript. charming requires JavaScript. textplots is terminal-only.

starsight's differentiators are: no JavaScript, no C dependencies, comprehensive chart coverage, deep Rust stack integration, and multiple output formats from a single API.

The opportunity is timing. The first Rust visualization library that reaches maturity will become the default choice, just as matplotlib became the default for Python. starsight aims to be that library.

## How the resonant-jovian ecosystem works together

prismatica provides 308 scientific colormaps as compile-time lookup tables. When starsight needs a color from the viridis colormap, it calls prismatica viridis eval with a value between 0 and 1 and gets back an RGB color. The colormaps are embedded in the binary at compile time, so there is no runtime file I/O.

chromata provides 1104 editor color themes as compile-time constants. When starsight applies a dark theme, it reads the background, foreground, and accent colors from the theme and derives chart colors from them.

caustic (a Vlasov-Poisson astrophysical simulation solver) and phasma (its terminal UI) are consumers of starsight. They will use starsight to visualize simulation results. This consumer relationship informs API design: the API must work well for large scientific datasets with millions of data points.

All crates in the ecosystem are GPL-3.0 licensed, consistent with the project's commitment to free-as-in-freedom software.

## What happens after 1.0.0

Once starsight reaches 1.0.0, semver becomes strict. Patch versions fix bugs. Minor versions add features without breaking the API. Major versions may break the API.

A well-designed 1.0.0 API rarely needs a 2.0.0. The non_exhaustive attribute, builder pattern, and feature flags all provide extension points that accommodate new functionality without breaking changes. New chart types are new Mark implementations. New backends are new DrawBackend implementations. New scales and colormaps are new types behind feature flags.

Post-1.0, the focus shifts from API design to performance optimization, chart type coverage, and ecosystem integration. The architecture you build now determines whether that shift is smooth or painful.

## Everything you need to know before writing the first line

You now have the complete mental model. The architecture is seven layers with strict dependency direction. The rendering pipeline goes from data to marks to scales to coordinates to path commands to backend to pixels. Color goes from sRGB u8 to tiny-skia premultiplied f32. Text goes from string to shaped glyphs to pixel callback to pixmap fill. Testing is snapshots for visual output, property tests for math, unit tests for everything else.

The tools are: rustfmt, clippy, cargo-deny, cargo-semver-checks, cargo-insta, cargo-llvm-cov, cargo-nextest, git-cliff, taplo, criterion, and cargo-flamegraph.

The rules are: no unsafe in layers 3 through 7, no panics in library code, no println, no async, no JavaScript, no C in defaults, no nightly. Every public type gets Debug and Clone. Every public item gets a doc comment. Every error is a StarsightError. Every feature is behind a flag.

Start with the vertical slice. Get plot save to produce a PNG. Everything else follows from there.


---

## Chapter Nineteen: How starsight is architected

starsight is organized into seven layers, each a separate Rust crate. Each layer depends only on layers below it. This is enforced by the Cargo dependency graph: it is physically impossible for a lower layer to import from a higher layer because the higher layer is not in its dependency list.

### Why layers instead of a single crate

A single crate containing everything would be simpler to set up but harder to maintain. When everything is in one crate, a change to the GPU rendering code recompiles the axis rendering code, the text layout code, and the statistical transforms. In a layered workspace, a change to layer 6 (interactivity) only recompiles layers 6 and 7 and the facade. Layers 1 through 5 are untouched.

The layer structure also enforces separation of concerns. The tick algorithm in layer 2 cannot accidentally depend on the Figure builder in layer 5 because layer 2 cannot see layer 5. This prevents the kind of tangled dependency graph where everything depends on everything, which makes code impossible to reason about.

### Layer 1: Rendering and primitives

Layer 1 is the foundation. It contains the basic types (Point, Vec2, Rect, Size, Color, Transform), the error types (StarsightError, Result), the DrawBackend trait, and the rendering backend implementations (tiny-skia, SVG, PDF, wgpu, terminal).

Every other layer depends on layer 1. The types defined here are used everywhere: Point appears in coordinate mapping, drawing commands, text positioning, and layout calculations. Color appears in marks, themes, scales, and backends. StarsightError appears in every function that can fail.

The DrawBackend trait is the contract between the chart-building code (which describes what to draw) and the rendering code (which actually draws it). The trait defines methods for drawing paths, filling rectangles, drawing text, and saving output. Each backend implements these methods differently: the tiny-skia backend rasterizes to pixels, the SVG backend writes XML elements, the terminal backend converts to escape sequences.

### Layer 2: Scales, axes, and coordinates

Layer 2 builds on layer 1 to provide the mathematical machinery for mapping data to visual space. The Scale trait defines the map method (data to normalized zero-to-one range) and the inverse method (normalized back to data). LinearScale implements this with the linear formula. LogScale applies a logarithm first. SymlogScale handles zero-crossing data.

The tick module implements the Wilkinson Extended algorithm. It takes a data range and a target tick count and produces a Vec of tick positions.

The Axis struct bundles a scale, tick positions, and tick labels. The auto_from_data constructor creates an axis automatically from a data array: it computes the range, generates ticks, and formats labels.

The CartesianCoord struct bundles an x axis and a y axis with a plot area rectangle. Its data_to_pixel method takes data coordinates and returns a screen Point, handling the Y axis inversion.

### Layer 3: Marks, stats, and aesthetics

Layer 3 contains the visual vocabulary. The Mark trait has a render method that takes a coordinate system and a backend. Each concrete mark type (LineMark, PointMark, BarMark, AreaMark, and so on) implements this trait.

The stat module contains data transformations: Bin for histograms, KDE for density estimation, Boxplot for summary statistics, Regression for fitted lines. Each stat takes input data and produces output data in a form that a mark can consume.

The aesthetics module defines how data columns map to visual properties. This is the grammar of graphics layer: the user says "color maps to species" and the aesthetics system resolves this to specific color values for each data point.

### Layer 4: Layout and composition

Layer 4 handles the arrangement of multiple charts. GridLayout places charts in a row-column grid. FacetWrap and FacetGrid split data into small multiples. Legend generates a key mapping visual properties back to data labels. Colorbar shows a continuous color scale.

This layer does not render anything itself. It computes the position and size of each chart panel and creates the appropriate coordinate systems. The actual rendering is delegated to the marks and backends in lower layers.

### Layer 5: The high-level API

Layer 5 is where the user interacts with the library. The Figure struct is a builder: you create it, add marks, set labels and title, and call save or show. The plot macro provides a one-liner: plot of x, y creates a Figure with a LineMark and returns it.

This layer also handles data acceptance: converting Polars DataFrames, ndarray arrays, and plain slices into the internal data representation that marks consume. The conversions are behind feature flags so that users who do not use Polars do not pay the compile-time cost.

### Layer 6: Interactivity

Layer 6 adds mouse and keyboard interaction: hover tooltips, zoom (box zoom and wheel zoom), pan (click-and-drag), and selection (lasso and box select). It wraps a Figure and manages mutable state (current zoom level, pan offset, selected points).

This layer depends on a windowing system (winit for native, web-sys for browser) and is entirely optional. Static chart rendering (save to PNG or SVG) does not use layer 6 at all.

### Layer 7: Animation and export

Layer 7 handles animated charts (GIF and MP4 output), terminal inline display, PDF export, and interactive HTML export. It builds on all lower layers to provide the final output formats.

### The facade crate

The starsight crate is a thin shell that re-exports types from all layers. It is the only crate that users depend on. Its lib dot rs file uses "pub use" to make layer types available through a flat namespace. Its prelude module re-exports the most commonly used types.

---

## Chapter Twenty: The data-to-pixels pipeline

Understanding the end-to-end pipeline from raw data to rendered pixels is the most important mental model for building starsight. Every bug you encounter will be at a boundary between two stages of this pipeline.

### Stage 1: Data acceptance

The user provides data. This might be two plain arrays of f64 values, a Polars DataFrame with named columns, an ndarray matrix, or an Arrow RecordBatch. Layer 5 converts all of these into a common internal representation: pairs of f64 slices for x and y data. This conversion happens once, at the boundary between user code and library code.

### Stage 2: Scale computation

The data ranges determine the scale domains. The x domain is the minimum and maximum of the x data. The y domain is the minimum and maximum of the y data. But the scale domain is not exactly the data range: it is expanded to "nice" bounds using the tick algorithm. If the data goes from 3.7 to 97.2, the ticks might be 0, 20, 40, 60, 80, 100, and the domain becomes 0 to 100.

### Stage 3: Layout computation

The chart has a total size (for example, 800 by 600 pixels). Not all of this space is available for the plot area. The top needs space for the title. The left needs space for the Y axis tick labels and label. The bottom needs space for the X axis tick labels and label. The right needs a small margin.

Computing these margins requires knowing the width of the tick labels, which requires shaping the text. This is the first time the font system is involved. The tick labels are shaped using cosmic-text, their widths are measured, and the margins are computed.

The plot area is the rectangle that remains after subtracting all margins from the total size. This rectangle is where the data is drawn.

### Stage 4: Coordinate system creation

With the scale domains and the plot area established, the CartesianCoord is created. It knows how to convert any data coordinate pair to a pixel position within the plot area, including the Y axis inversion.

### Stage 5: Axis rendering

The axes are drawn onto the pixel buffer. For each axis: draw the axis line along the edge of the plot area. For each tick position: compute the pixel position using the scale, draw a short tick mark perpendicular to the axis, and draw the tick label text. Draw the axis label (for example, "Temperature" or "Time") centered along the axis.

### Stage 6: Mark rendering

Each mark renders itself by iterating its data, converting each data point to pixel coordinates using the CartesianCoord, and producing drawing commands. A LineMark produces move-to and line-to path commands. A PointMark produces filled circles. A BarMark produces filled rectangles.

The drawing commands are sent to the backend, which executes them. The tiny-skia backend converts path commands to tiny-skia Path objects and calls fill_path or stroke_path. The SVG backend converts path commands to SVG path data strings and appends them to the document.

### Stage 7: Output

The backend produces the final output. The tiny-skia backend encodes the Pixmap to PNG bytes and writes them to a file. The SVG backend serializes the document to an XML string and writes it to a file. The file extension determines which backend is used: a dot png extension triggers the tiny-skia backend, a dot svg extension triggers the SVG backend.

### Why this pipeline matters

Each stage has a clear input and output. When something goes wrong, you can isolate the bug by checking the output of each stage. If the data is wrong, the chart is wrong. If the scales are wrong, everything downstream is wrong. If the margins are wrong, the plot area is the wrong size. If the coordinate mapping is wrong, the marks are in the wrong position. If the path commands are wrong, the shapes are garbled. If the backend is wrong, the output file is corrupt.

Snapshot tests capture the final output of the entire pipeline. When a snapshot fails, you work backward through the stages to find which one changed. This systematic debugging approach is much more efficient than staring at a garbled chart and guessing.

---

## Chapter Twenty-One: The SVG rendering backend

SVG (Scalable Vector Graphics) is an XML-based format for describing vector graphics. Unlike PNG (which stores pixels), SVG stores shapes, text, and coordinates as text. This means SVG files are resolution-independent: they look sharp at any zoom level.

### How SVG works

An SVG document starts with an svg element that defines the coordinate system via the viewBox attribute. Inside, you add shape elements: rect for rectangles, circle for circles, line for line segments, path for complex shapes, text for text labels, and g for grouping elements with shared attributes or transforms.

Each element has attributes that control its appearance. The fill attribute sets the interior color. The stroke attribute sets the outline color. The stroke-width attribute sets the outline thickness. The transform attribute applies geometric transforms (translate, scale, rotate).

### Building SVG in starsight

starsight uses the svg Rust crate, which provides a builder API. You create a Document, add elements to it, and serialize it to a string. The SvgBackend struct holds the Document and the chart dimensions.

The fill_rect method creates an SVG rect element with x, y, width, height, and fill attributes. The draw_path method converts PathCommand sequences to SVG path data using the d attribute. The draw_text method creates a text element with x, y, font-size, and text-anchor attributes.

### The text measurement problem in SVG

SVG cannot measure text width. When you place a text element in an SVG, the width of the rendered text depends on the font, the font size, the kerning, and the rendering engine. Different browsers and SVG viewers use different fonts and render text differently.

starsight works around this by estimating: each digit is approximately 0.55 times the font size wide, and each average character is approximately 0.6 times the font size wide. This estimate is used for margin computation when the SVG backend is selected. For the tiny-skia backend, cosmic-text provides exact text measurement.

The more robust solution (planned for later versions) is to embed the font in the SVG using a style element with a base64-encoded font data URI. This ensures every SVG viewer uses the same font, eliminating the measurement problem.

### SVG text positioning

Text positioning in SVG uses the baseline as the reference point, not the bounding box. The x and y attributes set where the text baseline starts. To center text horizontally, set the text-anchor attribute to "middle." To center text vertically, set the dominant-baseline attribute to "central."

To rotate text (for example, a Y axis label that reads vertically), apply a transform: first translate to the label position, then rotate by negative 90 degrees. This positions the text at the correct location and then rotates it in place.

---

## Chapter Twenty-Two: Design patterns for library code

Building a library is different from building an application. A library must be used by many people with different needs, so its API must be flexible, consistent, and hard to misuse.

### The builder pattern

When a type has many optional configuration parameters, the builder pattern provides a clean API. Instead of a constructor with ten parameters (most of which are usually default), you create a builder with sensible defaults and let the user override only the parameters they care about.

The Figure type uses a builder pattern. You call Figure new to get a default figure (800 by 600, no title, no marks), then chain method calls: figure title "Temperature Over Time" dot x_label "Month" dot size 1024, 768. Each method takes a mutable reference to self and returns a mutable reference to self, enabling chaining.

The alternative pattern (consuming self and returning self) also works and is preferred by some Rust developers because it prevents accidentally using the builder after calling a terminal method. starsight uses mutable references because they are more ergonomic for conditional configuration: you can write "if show_legend then figure dot legend true" without rebinding the variable.

### The newtype pattern for type safety

When two parameters have the same underlying type but different meanings, the newtype pattern wraps the type to create a distinct type. Without newtypes, a function that takes two f32 parameters (font size and line height) can be called with the arguments swapped, and the compiler will not catch it. With newtypes (FontSize wrapping f32 and LineHeight wrapping f32), swapping them is a type error.

For starsight, the most important type distinction is between Point and Vec2. Both contain two f32 values, but Point represents a position and Vec2 represents a displacement. The type system enforces that you cannot add two positions (meaningless) but you can add a position and a displacement (meaningful). This catches real bugs in coordinate transformation code.

### The non_exhaustive attribute

This attribute on a public enum or struct prevents downstream code from exhaustively matching the enum or constructing the struct with literal syntax. This means you can add new variants or fields in future versions without breaking existing code.

For starsight, non_exhaustive should be on the StarsightError enum (new error variants will be added), on any configuration struct (new options will be added), and on any enum of chart types or scale types. It should not be on pure mathematical types like Point, Vec2, Color, and Rect, where the fields are the complete definition that will never change.

### The facade pattern for multi-crate libraries

The user depends on one crate (starsight) and interacts with a flat namespace. The internal organization into seven layers is hidden behind re-exports. This gives you freedom to restructure the internals without breaking the user's code, as long as the re-exported types stay the same.

### Accepting generic input with impl Into

Functions that take user-provided values should accept "impl Into" rather than specific types. A function that takes "title colon impl Into of String" accepts both string literals (which convert via From str for String) and owned Strings. A function that takes "color colon impl Into of Color" accepts starsight Colors, chromata Colors, and prismatica Colors.

This pattern makes the API feel effortless: the user passes what they have, and the library converts it automatically. The cost is a slight increase in compile time due to monomorphization, but it is negligible for API entry points.

---

## Chapter Twenty-Three: Testing a visualization library

Testing visual output is different from testing logic. You cannot assert that a chart "looks right." You can assert that its numerical properties are correct (unit tests), that its output matches a known reference (snapshot tests), and that its mathematical invariants hold (property tests).

### Unit tests

Unit tests verify individual functions with known inputs and expected outputs. For starsight, unit tests cover: the scale mapping formula (input 50 with domain 0 to 100 and range 0 to 800 should output 400), the tick algorithm (input range 0 to 100 with target 5 should produce round numbers), the color conversion (from_hex of 0xFF8000 should produce r equals 255, g equals 128, b equals 0), and the geometry operations (Rect width should equal right minus left).

Unit tests are fast, deterministic, and easy to write. They catch logic errors in the mathematical foundations but they cannot catch rendering bugs.

### Snapshot tests

Snapshot tests render a chart and compare the output to a stored reference. If anything changes (a line moves by one pixel, a color shifts slightly, a label moves), the test fails.

For starsight, snapshot tests use the insta crate. You render a chart to PNG bytes using the tiny-skia backend, pass the bytes to the binary snapshot assertion, and insta stores the PNG alongside the test. On subsequent runs, insta compares the new output byte-by-byte.

The workflow is: run cargo insta test (which runs all tests and creates pending files for any differences), then run cargo insta review (which shows an interactive comparison), then accept or reject each change. In CI, the check flag fails immediately on any difference.

Snapshot tests require determinism. Use the tiny-skia backend (which is CPU-based and deterministic), fixed chart dimensions, and embedded fonts (not system fonts). If the rendering produces even slightly different output on different machines (due to different font fallback or different floating-point rounding), snapshots will flake.

### Property tests

Property tests generate random inputs and verify that mathematical invariants hold. The proptest crate is the standard tool.

For starsight, the most valuable property tests are: scale roundtrip (for any value in the domain, mapping through the scale and back should return the original value within floating-point tolerance), tick monotonicity (for any data range and target count, the tick positions should be strictly increasing), color lerp bounds (for any two colors and any t between 0 and 1, each channel of the result should be between the channels of the inputs), and rendering safety (for any reasonable chart dimensions and data, rendering should not panic).

Property tests find edge cases that unit tests miss: the degenerate case where the data range is zero, the extreme case where the data has a billion elements, the NaN case where some values are missing.

### Testing against reference implementations

For some computations, you can verify starsight's output against a known-correct reference. Render the same data in starsight and matplotlib, and compare the results. This is manual and infrequent but catches systematic errors: if all Y axes are inverted, or if the tick algorithm is off by one.

---

## Chapter Twenty-Four: The Rust development toolchain

Building starsight requires more than just rustc. You need a suite of tools for formatting, linting, testing, benchmarking, auditing, and publishing.

### Cargo and its role

Cargo is Rust's build system and package manager. It reads Cargo.toml files, resolves dependency versions, compiles code, runs tests, and generates documentation. The most important commands are:

cargo check: type-checks the code without generating machine code. This is significantly faster than cargo build and catches most errors. Use it during development for rapid feedback.

cargo build: compiles the code and generates executables or libraries. The default profile is dev (with debug info and minimal optimization). The release flag enables the release profile (with full optimization and no debug info).

cargo test: compiles and runs tests. This includes unit tests, integration tests, and doc tests. The test matrix for starsight runs across multiple operating systems (Linux, macOS, Windows) and Rust versions (stable, beta, and the MSRV).

cargo doc: generates HTML documentation from doc comments. The no-deps flag skips dependency documentation, producing only starsight's docs.

### Clippy for linting

Clippy is a lint tool that catches common mistakes, suggests idiomatic alternatives, and enforces style conventions. starsight enables clippy pedantic at the warn level, which is more aggressive than the default.

Important pedantic lints for starsight: missing documentation warnings on public items, must-use suggestions on functions that return values, and unnecessary closure warnings where a function reference would suffice.

Some pedantic lints are too noisy and should be selectively allowed: module name repetitions (where a type name contains the module name), must-use candidate (which fires on nearly every function), and cast possible truncation (which fires on f64-to-f32 casts that are intentional in visualization code).

### Rustfmt for formatting

Rustfmt enforces consistent code formatting. Every contributor must run rustfmt before committing. CI checks formatting with cargo fmt check and fails if any file is not formatted.

The configuration lives in a rustfmt dot toml file at the workspace root. Important settings for starsight include: maximum line width (100 characters), whether to use trailing commas (always), and whether to reorder imports (yes).

### Cargo-deny for dependency governance

Cargo-deny checks dependencies for license compliance, security vulnerabilities, banned crates, and untrusted sources. The configuration lives in deny dot toml.

For starsight's GPL-3.0 license, the allow list must include every GPL-compatible license: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Zlib, CC0, Unlicense, MPL-2.0, LGPL variants, and GPL variants. The most common gotcha is forgetting the Unicode-DFS-2016 license, which is used by the unicode-ident crate that nearly every Rust project depends on.

### Cargo-insta for snapshot testing

As described in the testing chapter, insta stores reference outputs and fails when they change. The CLI tool (cargo insta) provides interactive review and acceptance of changes.

### Cargo-semver-checks for API compatibility

This tool compares your current public API against the last published version and reports any breaking changes. It catches removed functions, changed signatures, removed trait implementations, and other API breakage. Run it on every pull request to catch accidental breaks.

### Cargo-llvm-cov for code coverage

This tool measures which lines of code are executed by your tests. It uses LLVM's instrumentation to produce precise per-line coverage data. The output can be uploaded to Codecov for visual reporting.

For starsight, coverage should stay above 80 percent on library code. Low coverage on a specific module tells you where to add tests.

### Cargo-hack for feature flag testing

This tool tests your project with different feature flag combinations. The each-feature mode tests each feature individually, catching cases where a feature depends on code from another feature without declaring the dependency. The feature-powerset mode tests combinations of features, catching interaction bugs.

### Git-cliff for changelog generation

Git-cliff reads your commit history (which follows the Conventional Commits format) and generates a changelog. It groups commits by type (features, bug fixes, performance improvements) and formats them under the appropriate headings.

### Taplo for TOML formatting

Taplo formats Cargo.toml and other TOML configuration files. It enforces consistent key ordering, indentation, and style.

---

## Chapter Twenty-Five: Continuous integration with GitHub Actions

Every change to starsight runs through automated checks before it can be merged. This catches bugs early and ensures that the codebase stays healthy.

### The CI pipeline

The CI pipeline runs on every pull request and every push to the main branch. It consists of several parallel jobs:

Formatting check: runs cargo fmt check. Fails if any file is not formatted correctly.

Lint check: runs cargo clippy with warnings treated as errors. Catches code quality issues and potential bugs.

Build check: runs cargo check to verify the code compiles. This is faster than a full build.

Test matrix: runs cargo test on three platforms (Linux, macOS, Windows) and three Rust versions (stable, beta, and 1.85, the MSRV). This catches platform-specific bugs and version-specific regressions.

Feature testing: runs cargo test with all features enabled, with no default features, and with minimal features. This catches feature flag issues.

Snapshot tests: runs cargo insta test with the check flag. Fails if any visual output has changed.

Dependency audit: runs cargo-deny to check licenses and security advisories. The advisory check uses continue-on-error because new advisories can appear at any time.

Documentation build: runs cargo doc with no-deps to verify that documentation compiles without errors or broken links.

### Caching for speed

The Swatinem rust-cache action caches the compiled dependencies between CI runs. This reduces compile time from about 90 seconds (clean build) to about 20 seconds (cached incremental build). The cache key includes the Rust version, the Cargo.lock hash, and the operating system.

### The release pipeline

When a new version is ready, the release pipeline creates a git tag, publishes all crates to crates.io in dependency order, generates a changelog, and creates a GitHub release with the changelog as the release notes.

---

## Chapter Twenty-Six: The workspace structure and how crates connect

starsight is a Cargo workspace: a collection of crates that share a single Cargo.lock file and can be built, tested, and published together.

### The workspace Cargo.toml

The root Cargo.toml defines the workspace. It lists all member crates (starsight, starsight-layer-1 through starsight-layer-7, and xtask). It defines shared settings: the version (all crates share the same version), the edition (2024), and common metadata (license, authors, repository).

It also defines shared dependencies. The workspace dependencies section lists every external crate that any member uses, with its exact version. Member crates reference these with the workspace equals true syntax. This ensures all crates use the same version of each dependency.

### How crate dependencies flow

The dependency chain is strictly layered. Layer 1 depends on no internal crates. Layer 2 depends on layer 1. Layer 3 depends on layers 1 and 2. Layer 4 depends on layers 1, 2, and 3. And so on, with each layer depending on all layers below it. The facade crate depends on all seven layers.

The xtask crate is not published and does not follow the layer structure. It can depend on any crate in the workspace and is used for development automation.

### Feature flag propagation

When a user enables a feature on the starsight facade crate (for example, the gpu feature), the facade crate forwards it to the appropriate layer crate. The gpu feature on starsight enables the gpu feature on starsight-layer-1, which activates the wgpu dependency and compiles the GPU backend code.

Feature flags must be additive: enabling a feature adds functionality without removing anything. Two features should never be mutually exclusive.

### Workspace lints and profiles

The workspace lints section in the root Cargo.toml defines lint levels that all member crates share: unsafe code is forbidden, clippy pedantic is warned. Each crate opts in by adding lints workspace equals true to its own Cargo.toml.

The profiles (dev and release) are always workspace-wide. The dev profile uses optimization level 1 (gives tiny-skia reasonable performance during development). The release profile uses LTO (link-time optimization) and single codegen unit for maximum performance.

---

## Chapter Twenty-Seven: Publishing to crates.io

When starsight is ready for users, it will be published to crates.io, Rust's package registry.

### What crates.io requires

Each published crate must have: a name (unique on crates.io), a version (following semver), a license (an SPDX identifier), and a description (a short sentence explaining what the crate does). The description is searchable and appears in crates.io listings, so it should be clear and specific.

### Publishing order for a workspace

In a workspace with interdependent crates, the order matters. You must publish leaf crates first (those with no internal dependencies), then work upward. For starsight: layer 1 first, then layer 2, then 3, 4, 5, 6, 7, and finally the facade.

Between each publish, crates.io needs a few seconds to update its index. If you publish layer 2 before the index reflects layer 1's new version, the publish will fail because layer 2's dependency on layer 1 cannot be resolved.

The cargo-release tool automates this. It computes the topological order, publishes each crate with appropriate delays, and creates git tags and commits.

### Semver for pre-1.0 crates

In semver, the rules for versions starting with 0 are different. The minor version acts as the major version: going from 0.1.0 to 0.2.0 allows breaking changes. Going from 0.1.0 to 0.1.1 should not break anything.

This means: be very careful with patch releases. Use cargo-semver-checks before every publish, including patches. A single accidental breaking change in a patch release breaks every downstream user without their opt-in.

### Yanking and the permanence of publishing

Publishing to crates.io is permanent. You cannot delete a version or overwrite it. If you publish a version with a bug, you can yank it (which prevents new projects from depending on it) but existing projects with it in their lock file continue to work.

If you accidentally publish secrets (API keys, passwords), you must rotate them immediately. Yanking does not remove the code from crates.io's storage.

---

## Chapter Twenty-Eight: Building starsight step by step

This chapter describes the actual implementation order: what to build first, what to build next, and why.

### The first milestone: a line chart PNG

The goal for version 0.1.0 is a single working chart: plot of [1, 2, 3, 4], [10, 20, 15, 25] dot save "chart.png" produces a PNG file showing a line chart with axes, ticks, and labels.

This requires implementing a vertical slice through all the layers: primitives (Point, Vec2, Rect, Color) in layer 1, the tiny-skia backend in layer 1, LinearScale and the tick algorithm in layer 2, Axis and CartesianCoord in layer 2, the Mark trait and LineMark in layer 3, the Figure builder in layer 5, and the plot macro in layer 5.

### Start with the primitive types

The first code you write is the Point struct with two f32 fields. Add Vec2. Add the arithmetic: Point minus Point gives Vec2, Point plus Vec2 gives Point. Add Rect with constructors and accessors. Add Color with from_hex and to_hex. Add Transform wrapping tiny-skia's Transform.

These types are small, self-contained, and easy to test. Each type gets unit tests immediately. This gives you a foundation of proven code to build on.

### Then the tiny-skia backend

Create the SkiaBackend struct wrapping a Pixmap. Implement the constructor, the fill method, fill_rect, and save_png. Write a snapshot test that creates a 200-by-100 backend, fills it white, draws a blue rectangle, and saves to PNG. This validates the entire rendering pipeline: backend creation, drawing, encoding, and snapshot testing.

### Then scales and ticks

Implement LinearScale with the map and inverse formulas. Write unit tests for both. Implement the Wilkinson Extended tick algorithm. Write tests for various data ranges. This is mathematically intensive code that benefits from thorough testing.

### Then the coordinate system

Implement CartesianCoord with the data_to_pixel method. Write tests that verify known data values map to expected pixel positions, including the Y axis inversion.

### Then axis rendering

Implement axis drawing: the axis line, tick marks, tick labels, and axis label. This is the first code that uses both the coordinate system and the rendering backend together. Test with snapshots.

### Then the line mark

Implement LineMark with the render method that converts data points to pixel positions and produces path commands. Handle NaN values by breaking the path. Test with snapshots.

### Then the figure and the macro

Implement Figure with the builder API and the render method that orchestrates the entire pipeline. Implement the plot macro that creates a Figure with a LineMark. Write the integration test that calls plot, save, and verifies the output.

### After 0.1.0

With the first chart working, everything after is incremental. Add PointMark for scatter plots. Add BarMark for bar charts. Add more scale types. Add faceting. Add GPU rendering. Add terminal rendering. Each addition builds on the existing foundation without restructuring it.

The key is discipline: do not skip ahead. Do not implement GPU rendering before the CPU rendering works. Do not add interactivity before the static charts are correct. Each layer must be solid before building the next layer on top.

---

## Chapter Twenty-Nine: How to think about performance

Performance matters for a visualization library because users may have large datasets (millions of points) and expect charts to render in under a second.

### Where time goes

The rendering pipeline has several stages, each with its own performance characteristics. Data processing (iterating arrays, computing scales) is fast: modern CPUs process billions of arithmetic operations per second. Text shaping is moderately expensive: shaping and rasterizing each unique text string takes microseconds, but there are typically only dozens of unique strings (tick labels, axis labels, title). Path rendering is the bottleneck: filling and stroking complex paths with anti-aliasing requires per-pixel computation, and there may be millions of pixels.

### Do not optimize prematurely

Write correct code first. Test it. Then benchmark it. Then optimize only the hotspots. A function that takes 1 percent of the total time does not benefit from optimization, no matter how elegant the optimization would be.

Use criterion for benchmarks and cargo-flamegraph for profiling. criterion tells you how long things take. Flamegraphs tell you where the time goes. Together, they identify the functions that would benefit most from optimization.

### Common optimizations for rendering

Batch similar draw calls. Instead of calling fill_path once for each scatter point, accumulate all circles into a single path and call fill_path once. This reduces the overhead of path setup and rasterizer initialization.

Pre-cull off-screen geometry. If a data point maps to a position outside the plot area, do not generate path commands for it. The mask will clip it anyway, but generating and rasterizing invisible geometry wastes time.

Reuse buffers. The PathBuilder allocates memory for path commands. Instead of creating a new PathBuilder for each mark, clear and reuse the same one.

Disable anti-aliasing for axis-aligned elements. Anti-aliasing is expensive (it computes sub-pixel coverage) and unnecessary for horizontal and vertical lines at integer coordinates.

### Scaling to large datasets

For datasets larger than about 10,000 points, the number of pixels on screen is much less than the number of data points. Many data points map to the same pixel column. Drawing a line segment shorter than a pixel is invisible and wastes time.

Data decimation reduces the point count before rendering. The simplest algorithm divides the x axis into pixel-width bins and keeps only the minimum and maximum y value in each bin. This preserves the visual envelope of the data while reducing the point count to at most twice the pixel width.

The Largest Triangle Three Buckets algorithm is a more sophisticated alternative that preserves the overall shape of the line more faithfully.

Decimation should happen transparently: the user passes all their data, and starsight internally decimates before rendering. The original data is preserved for interactive features (hover should show the exact value, not the decimated value).

---

## Chapter Thirty: How to handle edge cases gracefully

Real data is messy. A robust visualization library must handle missing values, empty datasets, extreme ranges, and other edge cases without crashing.

### NaN values

NaN (Not a Number) is a floating-point value that represents undefined or missing data. It propagates through arithmetic: any computation involving NaN produces NaN. It also has the unusual property that NaN does not equal itself.

In starsight, NaN in the data should produce a visible gap in the chart, not a crash. The LineMark handles this by breaking the path: when it encounters a NaN value, it starts a new path segment at the next valid value. The PointMark skips NaN values entirely. The scale mapping function should pass NaN through without attempting division.

### Empty data

If the user passes empty arrays, the chart should render with axes and labels but no data marks. This is more useful than an error because it shows the chart skeleton (title, labels, axis structure) even when no data is available.

The tick algorithm and scale computation must handle empty data: the domain is undefined, so use a default domain of 0 to 1. The axis renders with ticks at 0 and 1.

### Zero-range data

If all data values are the same (for example, all y values are 42), the domain minimum equals the domain maximum. The scale formula divides by zero, producing infinity.

The fix is to detect this case and expand the domain. If min equals max, use min minus one to max plus one as the domain. This produces a sensible chart that shows all data points at the center of the axis.

### Extremely large or small values

If the data values are in the billions or the trillionths, the tick labels become very long. The tick formatting should use appropriate precision and potentially scientific notation for extreme values.

### Negative dimensions

If the user passes zero or negative chart dimensions, the Pixmap creation fails. The backend constructor should check for this and return a descriptive error rather than propagating the None from tiny-skia.

---

## Chapter Thirty-One: How to write good documentation

Documentation is not optional for a library. Users who cannot figure out how to use the library will not use it, no matter how good the code is.

### Doc comments on every public item

Every public function, method, struct, enum, trait, and constant needs a doc comment. The doc comment should explain what the item does, not how it is implemented. It should include the semantics: what the inputs mean, what the output means, what happens in edge cases.

Bad doc comment: "creates a new Point." Good doc comment: "creates a point at the given screen coordinates, where x increases rightward and y increases downward. Values are in logical pixels, not physical pixels."

### Code examples in doc comments

Doc comments can contain code examples that are automatically compiled and run as tests. This means your examples are always up to date: if the API changes and the example breaks, the test fails.

For starsight, the most important doc examples are: the one-liner on the plot macro (showing the simplest possible chart), the Figure builder example (showing how to compose marks), and the Color example (showing how to create and convert colors).

### The README as a landing page

The README is the first thing a potential user sees. It should answer three questions in the first screen: what is this (a scientific visualization library for Rust), what does it look like (a screenshot of a chart), and how do I get started (a three-line code example).

### Organize for discovery

Users look for documentation in predictable places. The crate root documentation (the doc comment on lib dot rs) is the landing page on docs.rs. The prelude module documentation lists the most commonly used types. Each type's documentation explains its purpose and links to related types.

Do not make users hunt through deeply nested modules to find what they need. Re-export important types at a shallow level and cross-reference them in the documentation.

---

## Chapter Thirty-Two: The resonant-jovian ecosystem

starsight does not exist in isolation. It is part of a family of crates published under the resonant-jovian organization on GitHub.

### prismatica: colormaps

prismatica provides 308 scientific colormaps from sources including Crameri, CET, matplotlib, CMOcean, and CartoColors. Each colormap is a compile-time lookup table: 256 RGB entries baked into the binary with zero runtime cost.

The primary method is eval, which takes a float between 0 and 1 and returns a Color. For a sequential colormap, 0 maps to the low end (dark or cold) and 1 maps to the high end (bright or warm). For a diverging colormap, 0 and 1 are the extremes and 0.5 is the neutral center.

starsight uses prismatica for all color scales. When a user maps data values to colors on a heatmap, scatter plot, or contour chart, the scale internally calls eval on a prismatica colormap.

### chromata: themes

chromata provides 1,104 editor and terminal color themes as compile-time Rust constants. Each theme is a struct with 29 color fields (background, foreground, keywords, strings, comments, and so on) plus metadata.

starsight uses chromata for the theming system. When a user applies a theme (for example, Catppuccin Mocha or Gruvbox Dark), starsight reads the theme's background, foreground, and accent colors and derives a chart color scheme. The background becomes the chart background. The foreground becomes the axis and text color. The accent colors become the data series color cycle.

### caustic and phasma

caustic is a Vlasov-Poisson solver for astrophysical simulation. phasma is its terminal UI. These are consumers of starsight: they will use it to visualize simulation results. This consumer relationship informs starsight's design: the API must handle large scientific datasets efficiently.

### Naming convention

All crates in the resonant-jovian organization use single Latin or Greek scientific words: chromata (colors), prismatica (prisms), caustic (a surface of concentrated light), phasma (phantom/spectrum), starsight (astronomical vision). This naming convention creates a recognizable brand and suggests the scientific register of the tools.

---

## Chapter Thirty-Three: Long-term maintenance and sustainability

Building starsight is a multi-year commitment. The codebase will grow, the dependency tree will evolve, and the user base (hopefully) will expand. Thinking about sustainability from the start prevents burnout and abandonment.

### Commit every day

Consistent progress is more important than sprinting. One meaningful commit per day (even if it is just a small refactor or a single test) maintains momentum and shows the project is active. The git history tells the story of steady forward motion.

### Accept contributions carefully

Contributions are welcome but each one creates a maintenance obligation. A contributed feature without tests and documentation is a liability. The CONTRIBUTING file should set clear expectations: tests, documentation, clippy-clean code, and snapshot updates for any visual changes.

### Monitor dependencies

Dependencies change over time. New versions may fix bugs, improve performance, or introduce breaking changes. Use Dependabot or Renovate to create automatic pull requests for dependency updates. Review major version bumps carefully.

### Plan for breaking changes

Before 1.0, breaking changes are allowed in minor versions (0.1 to 0.2). After 1.0, they require a major version bump (1.0 to 2.0). Use the non_exhaustive attribute, the builder pattern, and feature flags to minimize the need for breaking changes. Get the core traits (DrawBackend, Scale, Mark) right before 1.0 because changing them later is expensive.

### Share progress publicly

Post screenshots of rendered charts. Write blog posts about interesting technical challenges. Publish benchmarks comparing starsight to plotters. Public visibility attracts contributors and creates accountability.

---

## Chapter Thirty-Four: What makes a chart effective versus misleading

A visualization library has an ethical responsibility: the default behavior should produce charts that communicate data truthfully. Misleading charts are often not intentionally deceptive; they result from poor defaults.

### Start bar chart axes at zero

Bar charts encode value through length. If the y axis starts at 50 instead of 0, a bar that goes from 50 to 60 looks the same size as one that goes from 50 to 100, but the actual values differ by a factor of 5. This is one of the most common forms of chart deception. starsight defaults to starting bar chart y axes at zero.

For line charts, a non-zero baseline is acceptable because lines encode trends (slopes), not absolute magnitudes.

### Use perceptually uniform colormaps

Rainbow colormaps create false boundaries where none exist in the data. starsight defaults to viridis, which is perceptually uniform, colorblind-safe, and prints well in grayscale.

### Avoid 3D when 2D suffices

3D bar charts, 3D pie charts, and 3D scatter plots with no meaningful Z axis obscure data through perspective distortion. Bars further from the viewer appear smaller. Pie slices are foreshortened. starsight does not offer 3D versions of inherently 2D charts.

### Aspect ratio matters

The aspect ratio of a chart affects how slopes are perceived. A line chart that is very wide and short makes trends look flat. A chart that is very tall and narrow makes trends look steep. The mathematical recommendation (from Cleveland 1988) is to choose the aspect ratio so that the average slope of lines is about 45 degrees, which maximizes slope discriminability.

### Label everything

Every chart should have: a title (what the chart shows), axis labels (what each axis measures, with units), and a legend (if multiple series are present). Unlabeled charts are ambiguous and unprofessional. starsight's default theme includes space for these labels, and the Figure builder makes it easy to set them.

---

## Chapter Thirty-Five: Accessibility in visualization

About 8 percent of men and 0.5 percent of women have some form of color vision deficiency. Charts that rely solely on color to distinguish categories are inaccessible to these users.

### Colorblind-safe palettes

The default color cycle should use colors that are distinguishable by people with all common forms of color vision deficiency (deuteranopia, protanopia, tritanopia). Colors that differ in both hue and lightness are safer than colors that differ only in hue. prismatica provides several colorblind-safe palettes, and starsight should default to one of them.

### Redundant encoding

In addition to color, use shape (circles versus triangles versus squares) and line style (solid versus dashed versus dotted) to distinguish series. This provides a secondary channel for colorblind users and also helps when charts are printed in grayscale.

### Sufficient contrast

Text and data marks should have sufficient contrast against the background. The WCAG recommends a contrast ratio of at least 4.5 to 1 for normal text and 3 to 1 for large text. starsight's default dark-gray-on-white color scheme exceeds this threshold.

### Text alternatives

For interactive HTML output, include text descriptions of the data (accessible via screen readers). For static images, the alt text should summarize the chart's key findings.

---

## Chapter Thirty-Six: Everything comes together

You now have the complete mental model. Let us trace through what happens when a user writes the one-liner to create a chart.

The user writes: use starsight prelude star. Then: plot of [1, 2, 3, 4, 5], [23, 41, 38, 57, 49] dot save "chart.png."

The plot macro creates a Figure with default size (800 by 600) and a LineMark holding the x and y data.

The save method checks the file extension (png), creates a SkiaBackend at 800 by 600, and fills it with white.

The Figure compute the scales: x domain from the ticks (say 0 to 6), y domain from the ticks (say 20 to 60). The tick algorithm finds nice positions.

The Figure computes margins by shaping the tick labels with cosmic-text and measuring their widths. The plot area is the remaining rectangle.

A CartesianCoord is created with the x axis, y axis, and plot area.

The axes are rendered: axis lines, tick marks, tick labels. Each tick label is drawn with cosmic-text's draw callback painting pixels onto the Pixmap.

The LineMark renders: it iterates the five data points, converts each to pixel coordinates using the CartesianCoord, and produces path commands. The path is stroked onto the Pixmap with a blue color and a 2-pixel line width.

The Pixmap is encoded to PNG and written to the file.

The user opens chart.png and sees a clean line chart with axes, ticks, labels, and data. The entire process took milliseconds.

That is what we are building. You have the knowledge to make it happen. Start with Point and Vec2. The rest follows from there.


---

## Appendix: The complete journey from blank file to first chart

Let this serve as a summary and a roadmap. The path from an empty project to a working visualization library is long but each step is clear.

You begin by understanding ownership: every value has one owner, borrowing gives temporary access, and the compiler prevents data races at compile time. This matters because rendering code passes large pixel buffers through many functions, and Rust's ownership rules ensure these buffers are used correctly.

You learn traits: contracts that types can implement, enabling polymorphism through both static dispatch (generics) and dynamic dispatch (trait objects). This matters because the DrawBackend trait must be a trait object to allow runtime selection of rendering backends.

You learn error handling: Result for fallible operations, the question mark operator for propagation, thiserror for defining error types. This matters because every rendering operation can fail (invalid dimensions, missing fonts, disk full), and the library must handle these failures gracefully.

You learn iterators: lazy, chainable transformations on sequences of data. This matters because data processing (scaling, filtering, pairing) is the first stage of the rendering pipeline, and iterators make it both efficient and readable.

You learn computer graphics: pixels are point samples in a grid, framebuffers store them in memory, Bresenham's algorithm draws lines, Bezier curves describe smooth shapes, paths combine curves and lines, fill rules determine what is inside a shape, and strokes give lines visible width. This matters because the entire rendering backend is built on these concepts.

You learn color: the RGB model, the sRGB encoding, gamma curves, premultiplied alpha for compositing, WCAG contrast for accessibility. This matters because every pixel in every chart is a color, and getting colors wrong makes charts misleading.

You learn text rendering: font files, shaping engines, layout algorithms, rasterization callbacks. This matters because every chart has labels, and text is the hardest part of 2D graphics.

You learn the grammar of graphics: marks, aesthetics, scales, stats, coordinates, facets. This matters because it provides the theoretical foundation for a library that can produce any chart type from a small set of composable components.

You learn the Wilkinson Extended tick algorithm: a search over step bases and skip factors with a scoring function that optimizes simplicity, coverage, density, and legibility. This matters because good tick positions are the difference between a professional chart and an amateur one.

You learn the tiny-skia API: Pixmap for pixel storage, PathBuilder for constructing shapes, Paint for specifying colors, Stroke for line properties, Transform for geometric transformations, Mask for clipping, and the rendering methods that bring them all together. This matters because tiny-skia is the primary rendering engine.

You learn the cosmic-text API: FontSystem for font discovery, SwashCache for glyph caching, Buffer for text layout, Metrics for font sizing, Attrs for text styling, and the draw callback for pixel-level glyph rendering. This matters because chart text must be correctly shaped, measured, and rendered.

You learn the architecture: seven layers from primitives to animation, each in its own crate, each depending only on layers below it. This matters because the layer structure enforces separation of concerns and enables incremental development.

You learn the pipeline: data acceptance, scale computation, layout computation, coordinate creation, axis rendering, mark rendering, output encoding. This matters because every bug lives at a boundary between stages, and understanding the pipeline lets you isolate bugs systematically.

You learn the tooling: cargo for building, clippy for linting, rustfmt for formatting, cargo-deny for dependency governance, cargo-insta for snapshot testing, cargo-semver-checks for API compatibility, cargo-llvm-cov for coverage, cargo-hack for feature testing, git-cliff for changelogs, and GitHub Actions for continuous integration. This matters because professional software requires professional tooling.

And you learn the principles: start bar charts at zero, use perceptually uniform colormaps, provide redundant encoding for accessibility, handle NaN and empty data gracefully, write doc comments on every public item, never panic in library code, and test everything.

This is the complete set of knowledge needed to build starsight. The first line of code is the Point struct. The last line of code, many months later, is the 1.0.0 release. Every step in between is an application of the principles in this document.

Start with Point and Vec2. The rest follows from there.



## How affine transforms compose and why order matters

An affine transform is a mathematical operation that combines translation (shifting), scaling (stretching), rotation (spinning), and skewing (shearing) into a single operation. It is represented by six numbers arranged conceptually as a matrix. Every drawing operation in tiny-skia can have a transform applied, which repositions the output without modifying the source path or shape.

The crucial thing about transforms is that the order of composition matters. Translating by 100 pixels and then rotating 45 degrees produces a different result than rotating 45 degrees and then translating 100 pixels. In the first case, you move to a new position and then rotate around the new position. In the second case, you rotate around the origin and then translate, which moves you in a rotated direction.

tiny-skia provides two sets of composition methods: pre-methods and post-methods. A pre-method applies the new operation before the existing transform. This means the new operation acts on the input points first, and then the existing transform acts on the result. A post-method applies the new operation after the existing transform. The existing transform acts first, and then the new operation acts on its output.

For starsight, transforms are used in two main places. First, for DPI scaling: the entire chart is scaled by a factor (like 2 for retina displays) using a single transform on every draw call. Second, for rotated text: Y-axis labels are typically rotated 90 degrees counterclockwise, which requires translating to the label position, then rotating. Getting the order wrong produces labels that appear in the wrong location or at the wrong angle.

The Transform identity is the do-nothing transform. Composing any transform with identity produces the original transform unchanged. This is what you pass when you do not want any transformation.

## How PNG compression works at a high level

When starsight saves a chart as a PNG file, the pixel data goes through a two-stage compression pipeline.

Stage one is filtering. Each row of pixels (called a scanline) is independently filtered to reduce redundancy. There are five filter types: None passes the bytes through unchanged. Sub stores the difference between each byte and the corresponding byte in the pixel to the left. Up stores the difference from the pixel directly above. Average uses the average of the left and above pixels. Paeth uses a more complex predictor that selects whichever of left, above, or upper-left is closest to a computed value. The filter is chosen per row to minimize the resulting byte values (values near zero compress better).

Stage two is Deflate compression, the same algorithm used in gzip and zip files. It combines two techniques: LZ77 (finding repeated byte sequences within a sliding window and replacing them with back-references to earlier occurrences) and Huffman coding (replacing fixed-length symbols with variable-length codes based on frequency, so common symbols get short codes).

The filtering stage is what makes PNG compression effective for chart images. Chart images typically have large areas of solid color (backgrounds, bars, filled regions) where adjacent pixels are identical. The Sub or Up filter reduces these areas to runs of zeros, which Deflate compresses extremely well. This is why a chart PNG with a white background and a few colored lines is much smaller than a photograph of the same pixel dimensions.

For starsight, PNG encoding is handled by tiny-skia's encode-png method. You do not interact with the filtering or compression directly. But understanding the process explains why different chart designs produce different file sizes and why adding unnecessary gradients or textures to charts bloats the file.

## How SVG differs from raster graphics

A raster image (like PNG) stores individual pixel values. An SVG (Scalable Vector Graphics) document stores a description of shapes: draw a line from here to there, fill this rectangle with this color, place this text at these coordinates. The SVG viewer (a web browser, an image viewer, or a rasterizer library) interprets the description and renders it at whatever resolution is needed.

The advantage of SVG for charts is resolution independence. A PNG chart at 800 by 600 pixels looks blurry when zoomed in. An SVG chart at any nominal size looks sharp at any zoom level because the viewer re-rasterizes the shapes at the target resolution. This makes SVG ideal for web embedding and print publication.

SVG is an XML format. Each visual element is an XML element: rect for rectangles, path for arbitrary shapes, text for text, circle for circles, g for groups. Attributes control appearance: x and y for position, fill for interior color, stroke for outline color, stroke-width for outline thickness, font-size for text size.

The SVG coordinate system uses the same convention as screen graphics: origin at the top-left, x increasing right, y increasing down. The viewBox attribute defines the coordinate system: viewBox equals "0 0 800 600" means the internal coordinates span from (0,0) to (800,600). The actual display size is set by the width and height attributes on the root svg element.

For starsight, the SvgBackend generates SVG documents using the svg crate. Instead of rasterizing shapes onto pixels, it creates XML elements. A fill-rect call adds a rect element. A draw-path call adds a path element with d attribute containing the path data (M for MoveTo, L for LineTo, C for CubicTo, Z for Close). A draw-text call adds a text element.

The major limitation of SVG for charts is text measurement. When you generate an SVG, you need to know how wide a text label is in pixels (to compute margins and avoid overlaps). But the width depends on the font, which depends on what fonts are installed on the viewer's system. A browser in Japan might use a different font than a browser in Sweden, producing different text widths. starsight works around this by estimating text width (digits are about 0.55 times the font size, average characters about 0.6 times) and accepting the approximation. For precise measurement, use the PNG backend with cosmic-text, which shaping engine provides exact glyph widths.

## How the Rust type system prevents unit confusion

One of the most common bugs in graphics code is mixing up coordinate systems: passing a pixel value where a data value is expected, or using screen Y (increasing downward) where mathematical Y (increasing upward) is expected. These bugs do not cause compiler errors with plain f32 values because the compiler sees all f32s as interchangeable.

The newtype pattern solves this. You define a struct with a single field, like: struct Pixels wrapping f32, and struct DataUnits wrapping f32. These are distinct types. A function that expects Pixels will not accept DataUnits, and vice versa. The compiler catches the mismatch at compile time, before the code ever runs. The newtype has zero runtime overhead: the wrapping struct compiles to exactly the same machine code as a bare f32.

For starsight, the Point and Vec2 distinction is a practical example of this principle. Point represents a position in space. Vec2 represents a displacement (a direction and magnitude). They are both two f32 fields, but they support different operations. Point minus Point gives Vec2 (the distance between two positions). Point plus Vec2 gives Point (shifting a position). Point plus Point is a compile error (adding two positions is meaningless). Vec2 times f32 gives Vec2 (scaling a displacement). Point times f32 is a compile error (scaling a position is meaningless).

This catches real bugs. In layout code, you deal with positions (where to place a label) and offsets (how much margin to add). If they were both plain f32 tuples, nothing prevents accidentally adding two positions together and getting garbage coordinates. With separate types, the compiler catches this immediately.

## How the standard library traits interact with each other

The standard traits (Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Display) have specific relationships and rules.

Copy requires Clone. This makes sense: if a type can be trivially bit-copied, it can certainly be cloned. The reverse is not true: many types can be cloned (deeply copied) but not bit-copied because they own heap memory.

Eq requires PartialEq. Eq adds the reflexivity guarantee (a equals a for all a). Types that implement Eq can be used in more contexts, like HashMap keys.

Hash requires Eq (conceptually, though the compiler does not enforce this). The invariant is: if a equals b, then hash of a must equal hash of b. Types that implement PartialEq but not Eq (like f32) should not implement Hash because NaN violates the invariant.

Ord requires PartialOrd and Eq. Ord provides total ordering (every pair of values is comparable). PartialOrd allows incomparable values (like NaN compared to any number). Types that implement Ord can be sorted deterministically and used in BTreeMap.

Display provides human-readable formatting (using the curly-brace format specifier). Debug provides developer-readable formatting (using colon-question-mark). It is common to derive Debug but implement Display manually, because Debug has a reasonable derived form but Display is type-specific (Color should display as a hex string, not a struct dump).

For starsight, the standard trait derivations are: Point, Vec2, Rect, Size, and Color derive Debug, Clone, Copy, and PartialEq. Color also derives Eq and Hash (u8 channels support both). Transform derives Debug, Clone, Copy, and PartialEq. StarsightError derives Debug (through thiserror, which also generates Display and Error). Figure derives Debug and Clone. Mark implementations derive Debug and Clone.

## How Result and Option combine with iterators

One of Rust's most powerful patterns is collecting an iterator of Results into a Result of a collection. If you have an iterator where each element is Result of T comma E, you can collect into Result of Vec of T comma E. This gathers all the Ok values into a Vec, or returns the first Err if any element fails. This pattern is extremely useful for processing data where each element might be invalid.

For starsight, this pattern appears when parsing data. If you have a column of strings that should be parsed as floating-point numbers, you iterate over the strings, call parse on each one (which returns Result), and collect into Result of Vec of f64. If all strings parse successfully, you get Ok of a Vec of f64. If any string is invalid, you get Err with the parse error, and none of the valid values are wasted on a partial result.

Similarly, Option works with iterators through filter-map. If you have an iterator and a function that returns Option (Some for items you want, None for items you want to skip), filter-map applies the function and yields only the Some values. This is perfect for processing data with missing values: map NaN to None, filter-map to remove them, and process the remaining valid values.

The flatten method unwraps one level of nesting: an iterator of Option of T becomes an iterator of T (skipping Nones), and an iterator of Vec of T becomes an iterator of T (concatenating the Vecs). This is useful for processing nested data structures.

## How Rust's module privacy protects internal implementation

The module system's privacy rules serve a specific purpose in library design: they let you change internal code without breaking external users. If a function or type is private (not pub), external code cannot depend on it, which means you can rename it, change its signature, or delete it without a semver violation.

For starsight, this means: the internal conversion between starsight Color and tiny-skia Color should be a private method or a pub-crate function, not a fully public one. The internal layout computation algorithm should be private. The internal path construction for each mark should be private. Only the types and methods that users need to interact with should be public.

The prelude module (starsight prelude) re-exports the minimal set of types needed for basic usage: Figure, the plot macro, Color, Point, StarsightError, and Result. Everything else lives in the crate's module tree and is imported explicitly by users who need advanced functionality. This keeps the namespace clean for the common case while providing full access for power users.

## How to think about memory allocation in rendering code

Memory allocation (asking the operating system for memory) is relatively expensive compared to computation. In a tight rendering loop that processes thousands of data points, unnecessary allocations can dominate the runtime.

The primary sources of allocation in starsight's rendering pipeline are: the Pixmap (one large allocation per render, around 2 megabytes for an 800 by 600 chart), PathBuilder's internal buffer (one allocation per path, growing as commands are added), Vec of PathCommand (one allocation per mark's render call), cosmic-text's FontSystem (one large allocation on initialization), and the PNG encoder's output buffer (one allocation per encode).

For version 0.1.0, do not optimize allocations. Use straightforward Vec and String allocations everywhere. The code should be clear and correct first. Profile with cargo-flamegraph after correctness is established, and optimize only the actual hotspots.

The most likely optimization targets (after profiling, not before) are: reusing PathBuilder buffers between marks instead of creating new ones for each mark, pre-allocating the path command Vec with an estimated capacity based on the data size, caching shaped text so the same tick label is not shaped repeatedly, and reusing the Pixmap between renders in interactive mode instead of allocating a new one each frame.

## How DPI and resolution affect chart output

DPI (dots per inch) determines the physical size of pixels when a chart is printed or displayed on a high-resolution screen. A standard screen is about 96 DPI. A retina display is about 192 DPI. A laser printer is typically 300 or 600 DPI.

If you render a chart at 800 by 600 pixels and print it at 300 DPI, the physical size is 800 divided by 300 equals 2.67 inches wide by 600 divided by 300 equals 2 inches tall. That is quite small. For a 6-inch-wide printed chart at 300 DPI, you need 1800 pixels wide.

starsight separates logical size from physical size. The user specifies the chart in logical pixels (800 by 600). A scale factor converts to physical pixels. Scale factor 1 gives 800 by 600 for screens. Scale factor 3.75 gives 3000 by 2250 for 300 DPI print. Font sizes, line widths, and point sizes all scale proportionally, so the chart looks the same at every resolution, just sharper at higher DPI.

For SVG output, DPI is irrelevant because SVG is resolution-independent. The viewBox defines the coordinate system, and the viewer rasterizes at whatever resolution is needed.

## How to handle the edge cases that break charts

Edge cases that seem unlikely in isolation appear frequently in real data. Every component must handle them gracefully.

Empty data (zero-length arrays): the scale should produce a degenerate domain (like 0 to 1), and the chart should show axes with default labels but no data content. This is valid output, not an error.

Constant data (all values identical): the scale domain has zero width. Division by zero in the mapping formula must be handled, typically by returning 0.5 (mapping everything to the center).

NaN values (Not a Number): NaN propagates through arithmetic and breaks comparisons. The tick algorithm's loop termination depends on score comparisons, and NaN comparisons always return false, which can cause infinite loops if not guarded. Input NaN should produce gaps in line charts, not crashes.

Infinite values: these represent overflow. They should be treated as invalid data and filtered out before scale computation.

Very large data ranges (like 0 to 10 to the 20th power): the tick algorithm should produce ticks at appropriate orders of magnitude (using scientific notation in labels if necessary).

Very small data ranges (like 1.0000001 to 1.0000002): the tick algorithm should produce ticks at an appropriate precision, and labels should show enough decimal places to distinguish the values.

Negative dimensions (width or height less than or equal to zero): the Pixmap constructor returns None. The Figure should validate dimensions and return a descriptive error.

Very large dimensions (like a million by a million pixels): this would require 4 terabytes of memory. The Pixmap constructor returns None. The Figure should validate dimensions against a reasonable maximum and return a descriptive error.

## How Rust prevents the most common graphics programming bugs

In C and C-plus-plus, graphics code is a minefield of memory safety issues. Buffer overflows (writing past the end of a pixel buffer), use-after-free (using a texture after it has been freed), double-free (freeing a GPU buffer twice), data races (two threads rendering to the same buffer without synchronization), and null pointer dereferences (accessing an uninitialized font handle) are all common bugs that cause crashes, visual corruption, or security vulnerabilities.

Rust prevents all of these at compile time. Buffer overflows: Vec and slice access is bounds-checked (and the checks are elided by the optimizer when it can prove they are unnecessary). Use-after-free: the ownership system ensures values are not used after they are dropped. Double-free: single ownership ensures each value is dropped exactly once. Data races: the borrow checker prevents unsynchronized mutable access, and the Send and Sync traits control cross-thread access. Null pointers: Rust does not have null. Optional values use the Option type, which the compiler forces you to handle.

This means starsight can focus on getting the visualization logic right without worrying about memory corruption. A chart that renders incorrectly will produce wrong pixels, not a segmentation fault. The bugs you encounter are logic bugs (wrong scale formula, incorrect coordinate mapping, misplaced text) rather than memory bugs (buffer overflow, dangling pointer, data race). Logic bugs are much easier to find and fix because they are reproducible and do not depend on memory layout or timing.

The one exception is unsafe code, which opts out of the compiler's safety checks. starsight forbids unsafe in layers three through seven (the higher-level chart and layout code). Layer one may contain unsafe in the rendering backends where it interfaces with the hardware or optimizes hot paths, but this should be minimized and carefully audited.

## What makes Rust uniquely suited for a visualization library

Rust's combination of features makes it arguably the best language for a scientific visualization library.

Performance: Rust compiles to native code with no garbage collection, no runtime overhead, and aggressive optimization. tiny-skia renders at speeds comparable to Skia (written in C plus plus). This means interactive charts are smooth and large datasets render in reasonable time.

Safety: Memory bugs, the most common class of bugs in C and C plus plus graphics code, are prevented at compile time. This means fewer crashes, fewer security vulnerabilities, and more time spent on the actual visualization logic.

Zero-cost abstractions: Iterator chains, trait objects, generic functions, and closures compile to the same machine code as hand-written specialized code. You do not pay a performance penalty for using high-level abstractions.

Package manager: Cargo and crates.io make it easy to depend on and publish libraries. tiny-skia, cosmic-text, prismatica, and all other dependencies are one line in Cargo.toml. Publishing starsight to crates.io makes it available to every Rust project with one command.

Cross-platform: Rust compiles for Linux, macOS, Windows, and WebAssembly from the same source code. starsight works on all platforms without conditional compilation in the mark, scale, or layout code. Platform-specific code is isolated to the backend implementations.

Type system: Newtypes prevent unit confusion. The Point-versus-Vec2 distinction prevents coordinate arithmetic bugs. The trait object system enables pluggable backends. The module system enforces API boundaries.

The main disadvantage of Rust compared to Python is the learning curve. Python's matplotlib can be learned in an afternoon. starsight requires understanding ownership, borrowing, traits, and the other concepts covered in this document. But once understood, these concepts prevent entire categories of bugs and enable performance that Python cannot match. For researchers who already work in Rust, starsight eliminates the need to export data to Python for plotting, keeping the entire workflow in one language.

## How the resonant-jovian ecosystem creates a unified experience

starsight does not exist in isolation. It is part of the resonant-jovian organization on GitHub, alongside prismatica and chromata. These three crates are designed to work together seamlessly.

prismatica provides 308 scientifically validated colormaps as compile-time lookup tables. Each colormap is an array of 256 RGB triplets baked into the binary at compile time. There is no runtime file loading, no parsing, no allocation. When starsight needs to map a data value to a color (for a heatmap, a colored scatter plot, or a contour map), it calls prismatica's eval method with a value between 0 and 1 and gets back an RGB color.

chromata provides 1104 editor and terminal color themes as compile-time constants. Each theme has fields for background, foreground, keyword, string, function, and other semantic roles. When starsight applies a theme to a chart, it reads the theme's background color for the chart background, the foreground for axis lines and text, and the accent colors for the data series color cycle.

The color type is shared across all three crates: three u8 fields for red, green, and blue. Conversion between them is zero-cost because the memory layout is identical. starsight implements From of prismatica Color for starsight Color and From of chromata Color for starsight Color.

This integration means the user gets a consistent visual experience. The colormaps are perceptually uniform and colorblind-safe. The themes are curated from popular editors like Visual Studio Code, Vim, and Emacs. And everything works without network requests, configuration files, or runtime setup.

## What you now know and what comes next

You have now covered every concept needed to build starsight from scratch. You understand Rust's ownership system and how it applies to pixel buffers. You understand traits and how they enable pluggable rendering backends. You understand how pixels get from data to screen through the rendering pipeline. You understand how text is shaped, laid out, and rasterized. You understand how colors work from physics to bytes. You understand the grammar of graphics and how it decomposes charts into composable components. You understand scales, ticks, and coordinate systems. You understand the seven-layer architecture and why each layer exists. You understand the tools for testing, profiling, and publishing. You understand API design principles that keep a library maintainable over years.

The next step is implementation. Open starsight-layer-1 slash src slash primitives.rs and start typing. Add Vec2. Add the arithmetic. Write the tests. Commit. Then Color. Then the tiny-skia backend. Then scales and ticks. Then the line mark. Then the Figure builder and plot macro. Then the first PNG.

Each step is small. Each commit is a milestone. And when the first chart file appears on disk, you have proved that everything in this document works together correctly. Everything after that is expansion: more chart types, more backends, more features. The foundation is what matters, and now you have the knowledge to build it.

---

## Appendix: Glossary of terms you will encounter

This section defines every important term used in this document and in starsight's source code, in plain language. When you encounter a word you do not recognize, check here.

Affine transform is a geometric operation that preserves parallel lines. It combines rotation, scaling, translation, and shearing. In two dimensions, it is represented by six numbers. Tiny-skia's Transform type holds these six numbers.

Alpha compositing is the process of combining a semi-transparent foreground image with a background image. The most common operation is "source over," where the foreground appears on top of the background with transparency blending.

Anti-aliasing is a technique for reducing jagged edges on diagonal lines and curves. It works by partially coloring pixels at the boundary of a shape, based on how much of the pixel area is covered by the shape.

Aesthetic mapping is a connection between a data variable and a visual property. "x equals temperature" maps the temperature column to horizontal position. "color equals species" maps the species column to color.

Backend is a rendering engine that converts drawing commands into visible output. Tiny-skia is a CPU rasterization backend. SVG is a vector document backend. Wgpu is a GPU backend.

Band scale is a variant of categorical scale that assigns each category a range of pixels (a band) rather than a single point. Used for bar chart positioning.

Bezier curve is a parametric curve defined by control points. Quadratic Beziers use three control points. Cubic Beziers use four. They are the standard curve representation in SVG, PDF, and font outlines.

Borrowing is Rust's mechanism for temporarily accessing a value without taking ownership. Shared borrows (ampersand T) allow reading. Mutable borrows (ampersand mut T) allow reading and writing. The aliasing exclusive or mutability rule prevents simultaneous shared and mutable access.

Box is a smart pointer that allocates a value on the heap. Box of dyn Trait creates a trait object with dynamic dispatch.

Bresenham's algorithm is a classic line drawing algorithm that uses only integer arithmetic to determine which pixels to color when approximating a line.

Cargo is Rust's build system and package manager. It compiles code, manages dependencies, runs tests, and publishes crates.

Clippy is Rust's lint tool. It warns about common mistakes, suggests idiomatic alternatives, and enforces style conventions.

Closure is an anonymous function that captures variables from its surrounding scope. Closures in Rust implement one of Fn, FnMut, or FnOnce depending on how they use captured variables.

Colormap is a function that maps a numeric range (typically 0 to 1) to a sequence of colors. Sequential colormaps go from light to dark. Diverging colormaps have two hues diverging from a neutral center.

Compositing is the process of combining multiple images or layers into a single image. Porter-Duff compositing (the standard) defines operations like source-over, source-in, and destination-out.

Conventional Commits is a commit message format that structures messages as type, optional scope, colon, description. It enables automated changelog generation.

Coverage-based anti-aliasing computes the fraction of each pixel covered by a shape and uses this fraction as the alpha weight.

Crate is Rust's unit of compilation. A library crate produces a reusable library. A binary crate produces an executable.

Crates.io is Rust's package registry where crates are published and downloaded.

De Casteljau algorithm is a recursive linear interpolation method for evaluating points on Bezier curves. It is numerically stable and produces subdivision as a side effect.

Derive macro is a Rust compiler feature that automatically generates trait implementations based on a struct or enum's fields.

Diverging colormap uses two hues that diverge from a neutral center, used for data with a meaningful midpoint like temperature anomalies.

DrawBackend is the trait in starsight that all rendering backends implement. It defines methods for drawing paths, text, and rectangles.

Dynamic dispatch is method invocation through a trait object (dyn Trait), where the concrete implementation is determined at runtime via a vtable lookup.

Edition is a Rust release milestone that changes language defaults and enables new features. Edition 2024 (Rust 1.85) is the current edition.

Even-odd rule is a fill rule that determines whether a point is inside a path by counting ray crossings. Odd count means inside.

Faceting is creating small multiple charts by splitting data on a categorical variable. Each value gets its own chart panel.

Fat pointer is a two-word pointer used for trait objects and slices. For trait objects, the first word points to data and the second to the vtable.

Fill rule determines which pixels are inside a closed path. The two rules are even-odd and winding (non-zero).

FontSystem is cosmic-text's type that manages the font database. Creating one scans system fonts and takes about one second.

Framebuffer is a contiguous memory buffer storing pixel data. Each pixel is typically four bytes for RGBA.

From trait defines a conversion from one type to another. Implementing From automatically provides Into in the reverse direction.

Gamma correction is the nonlinear encoding applied to pixel values in sRGB. It allocates more precision to dark values, matching human visual sensitivity.

Grammar of graphics is a theoretical framework that decomposes charts into composable components: data, aesthetics, marks, stats, scales, coordinates, and facets.

Lifetime is a compile-time concept describing how long a reference is valid. Written as tick a, tick b, and so on.

Line cap determines the shape at the open end of a stroked path. Options are butt (flat), round (semicircle), and square (extended rectangle).

Line join determines the shape where two stroked path segments meet. Options are miter (sharp), round (arc), and bevel (flat cut).

Linear scale maps data values to visual values using a linear formula. The most common scale type.

Logarithmic scale applies a logarithm before linear mapping, compressing large values and expanding small values.

Mark is a visual shape drawn for each data point. Examples: point, line, bar, area, arc, text, rect.

Mask is a grayscale image used for clipping in tiny-skia. White areas allow drawing, black areas block it.

Miter limit prevents infinitely long spikes at acute miter joins by converting them to bevel joins.

Module is Rust's unit of code organization. Modules form a tree within each crate.

Move semantics is Rust's default for non-Copy types: assignment transfers ownership, invalidating the source.

MSRV is the minimum supported Rust version. For starsight, this is 1.85 (edition 2024).

NaN is a floating-point value representing undefined results. NaN does not equal itself and propagates through arithmetic.

Non_exhaustive is an attribute that prevents downstream code from exhaustively matching an enum or constructing a struct with literal syntax.

Object safety is the set of requirements a trait must satisfy to be usable as a trait object (dyn Trait).

Orphan rule prevents implementing a trait for a type unless either the trait or the type is defined in the current crate.

Ownership is Rust's memory management model where every value has exactly one owner that is responsible for cleaning it up.

Paint is tiny-skia's type that controls the appearance of drawing operations, including color, blend mode, and anti-aliasing.

Path is a sequence of drawing commands (move to, line to, curve to, close) that describes a shape.

PathBuilder is tiny-skia's type for constructing paths incrementally.

Perceptually uniform means equal steps in data produce equal perceived visual differences.

Pixmap is tiny-skia's pixel buffer type, owning premultiplied RGBA pixel data.

Point is a position in 2D space, represented by x and y coordinates.

Premultiplied alpha stores RGB channels already multiplied by the alpha value, simplifying compositing arithmetic.

Property test generates random inputs and verifies that mathematical invariants hold.

Qualitative palette uses distinct unrelated colors for categorical data.

Question mark operator propagates errors by returning early from a function if a Result is Err or an Option is None.

Rasterization converts mathematical shape descriptions into discrete pixel values.

Result is Rust's enum for fallible operations with Ok and Err variants.

Rustfmt is Rust's code formatting tool.

Scale maps data values to visual values (positions, colors, sizes).

Semver is semantic versioning: major dot minor dot patch. Major changes break API. Minor changes add features. Patch changes fix bugs.

Sequential palette varies from light to dark for ordered data.

Shaping converts Unicode characters into positioned glyphs, handling ligatures, kerning, and complex scripts.

Snapshot test compares output against a stored reference, failing if anything changes.

Stat transform preprocesses data before rendering: binning, density estimation, aggregation, regression.

Stroke is the visible outline drawn along a path, with properties like width, cap, join, and dash pattern.

SVG is Scalable Vector Graphics, an XML format for 2D graphics.

SwashCache is cosmic-text's type for caching rasterized glyph images.

Symmetric log scale handles zero-crossing data by using linear mapping near zero and logarithmic mapping far from zero.

Thiserror is a derive macro for implementing the Error trait on custom error types.

Tick mark is a small line on an axis indicating a labeled position.

Tiny-skia is a pure Rust CPU rasterization library, a port of a subset of Skia.

Trait is a collection of method signatures that types can implement, enabling polymorphism.

Trait object is a value of type dyn Trait, enabling dynamic dispatch through vtable lookup.

Transform is a geometric operation applied to coordinates before rendering.

Vec2 is a displacement or direction in 2D space, distinct from Point which is a position.

Vtable is a table of function pointers used for dynamic dispatch with trait objects.

WCAG is the Web Content Accessibility Guidelines, defining contrast ratio requirements for readable text.

Winding rule determines interior of a path by tracking the direction of boundary crossings. Non-zero winding number means inside.

Workspace is a Cargo feature for managing multiple related crates with shared configuration and a single lock file.

Xtask is a convention for a development automation binary crate within a workspace, used for gallery generation, benchmarking, and other tasks.


## How macro_rules macros work in Rust

A macro in Rust is a way to write code that generates other code. The plot macro in starsight is a macro that takes simple arguments (like two arrays of numbers) and generates the code to create a Figure, add a LineMark, and return the figure for saving.

The simplest form of macro definition uses the macro_rules keyword. You define patterns (called matchers) and the code to generate for each pattern (called transcribers). When the compiler encounters a macro invocation, it tries each pattern in order and uses the first one that matches.

A pattern consists of literal tokens and captures. A capture is written as a dollar sign followed by a name, a colon, and a fragment specifier. The fragment specifier tells the compiler what kind of Rust syntax the capture should match. The most common fragment specifiers are: expr (any expression, like a number, a variable, or a function call), ident (an identifier, like a variable name), ty (a type), stmt (a statement), tt (a single token tree, which is the most flexible), and literal (a literal value like a number or string).

For the plot macro, the simplest arm matches two expressions (x and y data): dollar x colon expr comma dollar y colon expr. When this pattern matches, the transcriber generates code that creates a Figure, creates a LineMark with the x and y data, adds the mark to the figure, and returns the figure.

Repetition patterns handle variable numbers of arguments. The syntax is dollar parenthesis pattern separator repetition_operator. The asterisk operator means zero or more repetitions. The plus operator means one or more. The question mark operator means zero or one. For example, dollar parenthesis dollar key colon ident equals dollar value colon expr comma asterisk matches zero or more comma-separated key-value pairs.

The plot macro uses repetition for optional configuration parameters. The invocation "plot of x comma y comma title equals chart comma color equals blue" matches a pattern with the x and y captures followed by a repetition of key-value pairs.

Macros in Rust are partially hygienic: local variables declared inside the macro get a unique identifier that does not conflict with variables in the calling scope. This prevents a common class of macro bugs where a macro variable accidentally shadows a user variable.

When debugging macros, the cargo-expand tool shows the generated code. This is invaluable when a macro does not behave as expected: you can see exactly what code was generated and why it does not compile or does not work correctly.

## How SVG documents are structured

SVG (Scalable Vector Graphics) is an XML-based format for vector graphics. A starsight SVG chart is a text file containing XML elements that describe shapes, text, and groups.

The root element is svg, which specifies the document's dimensions and coordinate system. The width and height attributes set the viewport size (the visible area). The viewBox attribute defines the internal coordinate system: the minimum x and y, and the width and height of the coordinate space. When the viewBox and viewport have different aspect ratios, the preserveAspectRatio attribute controls how they map.

Inside the svg element, shapes are defined with specific elements. The rect element draws a rectangle with x, y, width, height, fill, and stroke attributes. The circle element draws a circle with cx (center x), cy (center y), r (radius), and fill attributes. The line element draws a straight line between two points. The path element is the most versatile: its d attribute contains a mini-language of drawing commands.

The path data language uses single letters for commands: M for MoveTo (move the pen without drawing), L for LineTo (draw a line from the current position), C for cubic Bezier (with two control points and an endpoint), Q for quadratic Bezier, A for arc, and Z for ClosePath (draw back to the start). Uppercase letters use absolute coordinates; lowercase use relative coordinates.

The text element positions text at an x and y coordinate. The text-anchor attribute controls horizontal alignment: start (left-aligned), middle (centered), end (right-aligned). The dominant-baseline attribute controls vertical alignment: auto (baseline), central (vertically centered), hanging (top-aligned). To rotate text (like a Y-axis label), you apply a transform attribute: "translate(x, y) rotate(-90)" moves to the position and rotates 90 degrees counterclockwise.

The g element (group) groups child elements and can apply transforms, clipping, and styling to all children. For chart rendering, the plot area is typically a group with a clip-path that prevents data elements from drawing outside the axis boundaries.

The defs element defines reusable elements like gradients, patterns, and clip paths. A clip path is defined in defs with a clipPath element containing the clipping shape (usually a rect the size of the plot area), then referenced from a group element with a clip-path attribute.

For starsight, the SVG backend builds these elements using the svg crate, which provides a builder API. Each element is created with a constructor, configured with chained method calls for attributes, and added to the parent element.

The main limitation of SVG for chart rendering is text measurement. When starsight positions tick labels, it needs to know how wide each label is in pixels. But SVG is a text format: it does not render text until the SVG is opened in a browser or viewer. The width depends on the font available on the viewing system, which starsight does not know in advance. starsight works around this by estimating: digits are approximately 0.55 times the font size wide, and average characters are approximately 0.6 times.

## How premultiplied alpha compositing works mathematically

When you place one image on top of another, the alpha channel determines how they blend. The standard compositing operation is "source over," which represents "paint the source on top of the destination."

In premultiplied alpha, each color channel is already multiplied by the alpha value. A pixel with straight color (R, G, B, A) becomes premultiplied (R times A, G times A, B times A, A), where A is normalized to the range 0 to 1 (or 0 to 255 for integer representations).

The source-over formula for premultiplied alpha is elegantly simple: result equals source plus destination times (1 minus source_alpha). This is computed independently for each channel (red, green, blue, alpha).

Let us work through an example. Suppose you have a semi-transparent red source pixel: straight (255, 0, 0, 128). In premultiplied form (dividing alpha by 255 to get 0.502): premultiplied red equals 255 times 0.502 equals 128. So the premultiplied source is (128, 0, 0, 128).

Suppose the destination is an opaque white pixel: (255, 255, 255, 255) in premultiplied form (since alpha is 255, premultiplication is a no-op).

The compositing formula gives: result red equals 128 plus 255 times (1 minus 128/255) equals 128 plus 255 times 0.498 equals 128 plus 127 equals 255. Result green equals 0 plus 255 times 0.498 equals 127. Result blue equals 0 plus 255 times 0.498 equals 127. Result alpha equals 128 plus 255 times 0.498 equals 255.

The result is (255, 127, 127, 255), which is an opaque pinkish-red. This makes visual sense: a semi-transparent red on white produces pink.

The same formula for straight alpha would require: result_red equals (source_red times source_alpha plus dest_red times dest_alpha times (1 minus source_alpha)) divided by result_alpha. That is more operations, a division (which is slow), and a divide-by-zero hazard when result_alpha is zero.

This efficiency is why tiny-skia (and virtually every professional compositing system) uses premultiplied alpha internally. For starsight, you do not need to perform compositing yourself: tiny-skia handles it. But understanding the format helps you reason about pixel values when debugging rendering issues.

## How DPI affects chart rendering

DPI (dots per inch) determines the resolution of the output. A screen display is typically 96 DPI. A retina display is 192 DPI. A printed document is 300 to 600 DPI.

starsight separates logical size from physical size. The user specifies chart dimensions in logical pixels (like 800 by 600). A scale factor converts logical pixels to physical pixels. At 1x scale (96 DPI), the physical size equals the logical size. At 2x scale (192 DPI, retina), the physical size is 1600 by 1200. At 3.125x scale (300 DPI print at 96 DPI logical), the physical size is 2500 by 1875.

All visual properties (font sizes, line widths, point radii, margins) are specified in logical units and scaled by the same factor. A 12-pixel font at 1x scale is 12 physical pixels. At 2x scale, it is 24 physical pixels. This ensures charts look the same at all resolutions, just sharper at higher DPI.

The tiny-skia backend creates the Pixmap at the physical size and applies a Transform that scales all drawing operations. The marks and layout system always work in logical coordinates. They do not need to know the DPI.

For SVG output, DPI does not apply because SVG is resolution-independent. The viewBox defines the logical coordinate system, and the renderer handles scaling.

## How Cargo.lock works and when to commit it

Cargo.lock records the exact resolved version of every dependency in the project. When you first compile, Cargo resolves all dependencies to their latest compatible versions and writes the results to Cargo.lock. On subsequent compilations, Cargo reads Cargo.lock and uses those exact versions, ignoring newer versions that might be available.

For binary projects (applications), you always commit Cargo.lock to version control. This ensures that every developer and every CI run uses the same dependency versions, preventing "works on my machine" problems caused by different dependency resolutions.

For library projects (which starsight is), the convention is debated. The official Cargo documentation suggests not committing Cargo.lock for libraries because downstream users will have their own Cargo.lock. However, many library projects commit Cargo.lock for CI reproducibility: it ensures that CI always tests against the same dependency versions.

The cargo update command refreshes Cargo.lock to the latest compatible versions. You should run this periodically to pick up bug fixes and security patches.

## How publishing to crates.io works

Publishing makes your crate available for anyone to download and use. The process involves several steps.

First, ensure your Cargo.toml has all required fields: name, version, description, and license (or license-file). The description is especially important because crates.io search uses it for ranking.

Second, run cargo package to create the package and verify what will be included. The package command respects your .gitignore and can be further configured with include and exclude fields in Cargo.toml.

Third, run cargo publish to upload the package. This is irreversible: once a version is published, it cannot be overwritten. If you publish a version with a bug, you must publish a new version with the fix.

For workspace publishing, you must publish crates in dependency order: layer-1 first (no internal dependencies), then layer-2 (depends on layer-1), then layer-3, and so on up to the facade crate. Between each publish, there is a propagation delay (10 to 30 seconds) while crates.io indexes the new version.

Yanking (cargo yank) marks a version as deprecated. Existing projects with the yanked version in their Cargo.lock continue to work, but new projects cannot add a dependency on the yanked version. Yanking does not delete code from crates.io. If you accidentally publish a secret (like an API key), contact the crates.io team for removal.

Cargo supports Trusted Publishing via OIDC tokens, which allows GitHub Actions to publish without a stored API token. This is more secure than storing a long-lived token as a repository secret.

## How to choose the right data representation

starsight needs to accept data from multiple sources: raw arrays, Polars DataFrames, ndarray arrays, and Apache Arrow RecordBatches. Each source has a different API, but the marks and scales need a uniform representation.

The internal representation is the simplest possible: a reference to a contiguous slice of f64 values. All data source adapters convert their input into slices of f64 before passing to the rendering pipeline. This convergence happens in layer five.

For raw arrays, the user passes Vec of f64, arrays, or slices. These are already in the right format. No conversion needed.

For Polars DataFrames, the user specifies column names as strings. The adapter extracts the named column, casts it to f64 (returning an error if the column is not numeric), and accesses the underlying data as a slice.

For ndarray arrays, the adapter calls the as_slice method (which returns an Option, succeeding only if the array is contiguous in memory).

For Apache Arrow RecordBatches, the adapter extracts the named column, downcasts to Float64Array, and accesses the values as a slice.

This design means adding support for a new data source is purely a layer-five concern. You write a new adapter that converts the external format to a slice of f64, and every mark, scale, and coordinate system in the lower layers works automatically.

## How the grammar of graphics maps to Rust types

Each component of the grammar of graphics becomes a Rust type or trait.

An aesthetic mapping is a struct with fields for the source (a column name or an array index) and the target (an aesthetic channel like X, Y, Color, Size, Shape). For the initial implementation, aesthetics are implicit: the x and y data are passed directly to marks.

A mark is a trait with a render method that takes a CartesianCoord (the coordinate mapping) and a mutable reference to a DrawBackend. Each concrete mark type (LineMark, PointMark, BarMark) implements this trait. The mark knows how to convert its data to path commands using the coordinate system and draw them using the backend.

A statistical transform is a function that takes one representation of data and produces another. A Bin transform takes a Vec of f64 and produces a Vec of bin boundaries and a Vec of counts. A KDE transform takes a Vec of f64 and produces two Vecs: x positions and density values. Transforms are applied before marks: the histogram is a Bin transform followed by a BarMark.

A scale is a trait with map and inverse methods. LinearScale, LogScale, CategoricalScale each implement this trait. Scales are owned by axes, which combine a scale with tick positions and formatted labels.

A coordinate system is a struct that holds two axes and a plot area rectangle. Its data_to_pixel method applies both scales and returns a Point in pixel coordinates. The Y axis is inverted (higher data values correspond to lower pixel y coordinates) because screen coordinates increase downward.

Faceting is a layout operation: it creates a grid of sub-charts, each with its own CartesianCoord but potentially shared axis ranges. Faceting is handled in layer four.

The Figure builder in layer five assembles all of these: it holds marks, receives data, creates scales and axes when save or show is called, computes layout (margins, facets, legends), and orchestrates the rendering of each mark on each backend.

## Understanding the difference between f32 and f64 for chart data

starsight uses f32 for pixel coordinates and rendering (because tiny-skia uses f32) and f64 for data values and scale computations (because scientific data may need the extra precision).

f32 (32-bit floating point) has about 7 decimal digits of precision. This is sufficient for pixel coordinates: a chart that is 10,000 pixels wide needs at most 5 digits. Using f32 for rendering keeps the pixel buffer memory small and matches tiny-skia's API.

f64 (64-bit floating point) has about 15 decimal digits of precision. This is necessary for scientific data: a measurement of 123456.789012 requires 12 significant digits, which exceeds f32's precision. Using f64 for data and scales prevents precision loss during computation.

The conversion happens at the coordinate mapping boundary: the CartesianCoord's data_to_pixel method takes f64 data values, computes the pixel position using f64 arithmetic, and casts the result to f32 for the Point type. This is a narrowing conversion that may lose precision, but since the result is a pixel position (which only needs to be accurate to a fraction of a pixel), the precision loss is invisible.

## How color blindness affects chart design and what to do about it

Approximately 8 percent of men and 0.5 percent of women have some form of color vision deficiency. The most common form is red-green color blindness (deuteranopia and protanopia), where red and green hues appear similar. Less common is blue-yellow color blindness (tritanopia).

For chart design, this means: never use color as the only way to distinguish data series. If you have a chart with a red line and a green line, a red-green colorblind viewer cannot tell them apart. Add redundant encoding: different line styles (solid, dashed, dotted), different point shapes (circle, square, triangle), or direct labels.

Use colormaps that are designed for colorblind safety. The viridis colormap varies in both hue and lightness, so even if the hue differences are invisible, the lightness differences remain. The cividis colormap was mathematically optimized to be perceptually identical for both normal and deuteranopic vision.

For discrete color palettes (assigning colors to categories), use palettes that vary in lightness as well as hue. The Tableau10 and Set2 palettes from prismatica are designed with this constraint.

starsight's defaults should prioritize colorblind safety: viridis as the default continuous colormap, a lightness-varying palette as the default discrete palette, and API support for adding redundant encoding (shape and dash style in addition to color).

## How the workspace dependency chain prevents circular imports

The layer-numbered crate structure enforces a strict dependency direction. starsight-layer-2 depends on starsight-layer-1. starsight-layer-3 depends on layers 1 and 2. Each layer depends only on layers with lower numbers.

This is enforced by Cargo itself. If you accidentally add starsight-layer-5 as a dependency of starsight-layer-2, Cargo will reject it because it would create a cycle (layer-5 depends on layer-2, and you are trying to make layer-2 depend on layer-5). Cargo does not allow dependency cycles.

This constraint shapes where types live. The Point type must be in layer one because layer two (scales and coordinates) needs it. The Scale trait must be in layer two because layer three (marks) needs it. The Mark trait must be in layer three because layer five (the Figure) needs it.

If you find yourself wanting to use a type from a higher layer, that is a sign the type is in the wrong layer. Move it down to the lowest layer that makes sense.

## How Rust's module privacy prevents misuse of internal APIs

starsight's public API is the set of types and functions reachable through the starsight facade crate. Everything else is an internal implementation detail.

Layer crates use pub(crate) for types that need to be shared between modules within the crate but should not be visible externally. For example, the SkiaBackend's internal helper methods for path conversion are pub(crate): they are used by the backend module but not exposed to users.

The facade crate's lib.rs uses pub use to re-export exactly the types that should be public. Types not re-exported are effectively invisible to users, even if they are pub within their layer crate.

This layered visibility means you can refactor internal modules freely (renaming, splitting, merging) without breaking the public API, as long as the re-exports in the facade remain the same.

## How to think about memory allocation in the rendering pipeline

Understanding where memory is allocated helps you avoid unnecessary copies and manage performance.

The Pixmap is the largest allocation: width times height times 4 bytes. An 800 by 600 chart at standard resolution is about 1.9 megabytes. At 300 DPI (about 2500 by 1875 pixels), it is about 18.7 megabytes. This allocation happens once when the backend is created.

The FontSystem allocates when loading system fonts: 10 to 50 megabytes depending on the system. This happens once, on first creation.

Each path (for each line, bar, or scatter point) allocates a Vec of path commands. A line chart with 1000 points allocates about 1000 commands (each a few tens of bytes). This is modest.

The scene (if used) allocates a Vec of SceneNode values. Each node is the size of its largest variant (Path, Text, Group, Clip).

PNG encoding allocates a temporary buffer for the compressed output.

For 0.1.0, do not optimize allocation. Use straightforward Vec and String allocations. Profile first (after correctness is established), then optimize only the hotspots.

## How to read compiler error messages

Rust's compiler errors are famously helpful, but they can be overwhelming for complex generic code. Here are patterns you will encounter frequently.

"value used after move" means you assigned or passed a value somewhere and then tried to use the original. The fix is usually to clone the value before passing it, or to pass a reference instead.

"cannot borrow as mutable because it is also borrowed as immutable" means you have a shared reference (ampersand) active while trying to take a mutable reference (ampersand mut). The fix is usually to restructure the code so the shared reference's last use comes before the mutable reference is taken.

"trait bound not satisfied" means you are trying to use a type in a context that requires a specific trait (like Debug, Clone, or Send) but the type does not implement it. The fix is usually to add a derive or a manual implementation.

"the trait DrawBackend is not implemented for" followed by a specific type means you are trying to use a type as a DrawBackend but it does not implement the trait. Check whether you have the right type and whether the implementation exists.

"lifetime mismatch" errors are the trickiest. They mean a reference is not living long enough or is outliving its data. Read the error message carefully: it usually points to exactly which reference is the problem and suggests adding a lifetime annotation.

When error messages are about types from macro-generated code, use cargo-expand to see the actual generated code and understand what types are involved.

## The complete knowledge map for building starsight

You now have the knowledge to build starsight. Here is everything covered in this document, organized by when you need it.

Before writing any code, understand: ownership and borrowing, traits and generics (especially trait objects and object safety), error handling with Result and the question mark operator, iterators and closures, what pixels are and how they are stored, how Bezier curves and paths work, how coordinate systems work (especially Y axis inversion), what premultiplied alpha is, how the sRGB color space works, and what the grammar of graphics is.

While implementing layer one, understand: how tiny-skia's Pixmap, PathBuilder, Paint, Stroke, Transform, and fill/stroke methods work. How cosmic-text's FontSystem, Buffer, and draw callback work. How to create snapshot tests with insta. How to measure text for margin computation.

While implementing layer two, understand: how linear scales map data to pixels (the normalization and interpolation formula). How the Wilkinson Extended tick algorithm scores candidate tick sequences. How CartesianCoord inverts the Y axis.

While implementing layer three, understand: how LineMark handles NaN gaps by starting new sub-paths. How PointMark batches circles for performance. How the Mark trait is object-safe.

While implementing layer five, understand: how the Figure builder assembles marks, creates scales, computes margins, and orchestrates rendering. How the plot macro captures expressions and generates Figure construction code.

Throughout development, understand: how cargo-deny enforces license compliance. How cargo-insta catches visual regressions. How cargo-semver-checks prevents accidental API breakage. How clippy catches style issues and potential bugs. How conventional commits enable automatic changelog generation.

Start with Vec2. Then Rect methods. Then Color methods. Then Transform. Then SkiaBackend. Then snapshot test. Then LinearScale. Then ticks. Then Axis. Then CartesianCoord. Then LineMark. Then Figure. Then plot macro. Then the first PNG.

Good luck.

