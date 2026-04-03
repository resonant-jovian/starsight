# Everything you need to know to build starsight

Authored by Claude Opus 4.6 (Anthropic) with Albin Sjogren. A complete teaching document designed for text-to-speech. No code blocks, no backticks, no tables, no formatting that breaks screen readers. Just words.

This document assumes you know that in Rust, an impl block attaches methods to a type so you can call variable dot method, that String new creates an empty string, and that you have seen basic Rust syntax like let, fn, struct, and enum. It assumes high-school-level math and physics. It does not assume you know anything about computer graphics, data visualization, library design, or advanced Rust.

---

## Chapter 1: What starsight is and why it needs to exist

Imagine you are a physicist running simulations in Rust. You have computed a million data points describing the trajectory of particles in a plasma. Now you want to see what the data looks like. In Python, you would type import matplotlib, call plot, and get a chart. In Rust, there is no equivalent. This is the gap that starsight fills.

starsight is a scientific visualization library for Rust. It takes numerical data and turns it into charts: line charts, scatter plots, bar charts, histograms, heatmaps, box plots, violin plots, contour maps, 3D surfaces, and about fifty other chart types. It produces PNG images, SVG vector graphics, PDF documents, and interactive displays in terminal windows and native GUI windows.

The Rust ecosystem has a few existing plotting libraries, but each has limitations. Plotters is the most mature, but its API is verbose, development has slowed, and it has a design flaw where its rendering trait requires the Sized bound, preventing runtime backend selection. plotly-rs and charming generate JavaScript specifications and require a web browser to render, defeating the purpose of a compiled language. egui-plot is locked to the egui GUI framework and cannot produce static images for publications. textplots draws only ASCII art in the terminal.

starsight aims to be the matplotlib of Rust: one library covering the full range of scientific visualization, from quick exploratory plots to publication-quality figures. It is organized into seven layers, each a separate crate. Layer one is rendering primitives and backends. Layer two is scales and axes. Layer three is marks (the visual elements like lines and bars). Layer four is layout (grids, faceting, legends). Layer five is the high-level API. Layer six is interactivity. Layer seven is animation and export. Each layer depends only on layers below it, enforced by the package manager.

starsight belongs to the resonant-jovian ecosystem. Its sister crates are prismatica (308 scientific colormaps as compile-time data) and chromata (1104 editor themes as compile-time data). These provide the actual color and theme systems starsight uses internally.

## Chapter 2: Why you need to understand Rust deeply to build this

Building a visualization library exercises nearly every feature of the Rust language. You need ownership and borrowing to manage pixel buffers efficiently. You need traits and trait objects to support multiple rendering backends. You need generics to write reusable scales. You need error handling to report rendering failures gracefully. You need iterators to process data. You need closures because the text rendering library delivers glyph pixels through a callback. You need lifetimes because some types borrow data from others. You need smart pointers because marks are stored as boxed trait objects. You need modules and visibility to organize nine crates. You need macros for the plot shorthand. You need conditional compilation for optional features.

None of these topics is individually hard, but they interact in ways that only become apparent when building something substantial. This document teaches each piece and then shows how they fit together.

## Chapter 3: How ownership works and why it matters for pixel buffers

Every value in Rust has exactly one owner. The owner is the variable that holds the value. When the owner goes out of scope, the value is automatically dropped: its memory is freed, file handles are closed, network connections are shut down. This automatic cleanup is called RAII: Resource Acquisition Is Initialization.

When you write let y equals x for most types, ownership transfers from x to y. This is called a move. After the move, x is no longer valid, and the compiler rejects any attempt to use it. This prevents double frees (freeing memory twice), use-after-free (accessing freed memory), and data races (concurrent unsynchronized access).

For starsight, think about a pixel buffer. A pixel buffer stores the color of every pixel in an image. For an 800 by 600 chart with 4 bytes per pixel (red, green, blue, alpha), that is nearly two megabytes of heap memory managed by a Vec. When you create a pixel buffer, the variable owns that allocation. If you move it to another variable, the original becomes invalid. You cannot accidentally have two variables pointing to the same buffer, which would create confusion about which one should free it.

What if you want to pass the buffer to a function that draws on it without giving up ownership? This is where borrowing comes in.

## Chapter 4: How borrowing works and why rendering needs mutable references

Instead of transferring ownership, you can lend a value by creating a reference. A shared reference gives read-only access. You can have as many simultaneous shared references as you want. A mutable reference gives exclusive read-write access. You can have exactly one mutable reference at a time, and while it exists, no shared references can exist either.

This rule, sometimes called aliasing XOR mutability, is the cornerstone of Rust's memory safety. It prevents data races (two pieces of code reading and writing the same memory simultaneously), iterator invalidation (modifying a collection while iterating through it), and aliased mutation (two references disagreeing about what the data looks like).

For starsight, when you pass the pixel buffer to a drawing function, you pass a mutable reference. The function modifies pixels through this reference and returns it when done. The tiny-skia library's fill-path method takes a mutable reference to the Pixmap (the pixel buffer being modified) and shared references to the Path (the shape) and Paint (the color). The mutable reference ensures exclusive access during rendering. The shared references ensure the shape and color data is not modified mid-render.

This means you cannot use the same pixmap as both the rendering target and a pattern source simultaneously. That would require both a mutable reference (to write) and a shared reference (to read pattern data) to the same object. The compiler rejects this. The solution is to copy the pattern data first, or use separate buffers.

## Chapter 5: What Copy and Clone mean

When you assign one variable to another, the default behavior is a move: the old variable becomes invalid. But some types are so small and simple that this restriction is unnecessary. These implement the Copy trait.

Copy is a marker trait with no methods. It tells the compiler: instead of moving, duplicate the value bit-for-bit. Both variables remain valid. Types that implement Copy include integers, floating point numbers, booleans, characters, shared references, and tuples or arrays where every element is also Copy. String, Vec, Box, and anything owning heap memory cannot implement Copy because duplicating them would require duplicating the heap allocation, which is not a simple bit copy.

Clone is different. Clone requires you to explicitly call dot clone, and the operation can be expensive. Cloning a String allocates new heap memory and copies all bytes. Clone is always visible in the code, so you know when you are paying the cost.

For starsight, the geometry types (Point, Vec2, Rect, Size, Color) should implement Copy because they are small, contain no heap data, and copying them is trivially cheap. Types that hold data arrays (like LineMark with its Vec of f64) should implement Clone but not Copy.

## Chapter 6: What Debug, PartialEq, Eq, and Hash mean

The Debug trait generates a text representation useful for debugging. When a test assertion fails, Rust prints the Debug representation of both values so you can see what went wrong. Every public type in starsight should derive Debug.

PartialEq generates an equality comparison. The word Partial refers to partial equivalence relations from mathematics: not all values need to be comparable to themselves. The canonical example is floating-point NaN (Not a Number), which is defined to not equal itself. So f32 and f64 implement PartialEq (you can compare them) but not Eq (the comparison is not reflexive because NaN does not equal NaN).

Eq is a marker trait asserting that equality is reflexive: every value equals itself. It is required for types used as HashMap keys. Do not derive Eq on types containing floating-point fields.

Hash generates a hash function for HashMap and HashSet. The critical rule: if two values are equal according to PartialEq, their hashes must also be equal. Because NaN breaks this (two NaN values might have different bit patterns but are both "not equal to themselves"), f32 and f64 do not implement Hash. starsight's Color type uses u8 channels, so Hash is safe.

Default generates a value using the default for each field: zero for numbers, false for booleans, empty for strings, None for Options. Only derive Default when the default value is meaningful. A default Point at zero, zero makes sense. A default Figure with no data does not.

## Chapter 7: What traits are and why starsight needs them

A trait in Rust defines a set of methods that a type can implement. If you have worked with interfaces in Java or protocols in Swift, traits are similar. You declare the trait with the method signatures, then write impl blocks for each type that provides the behavior.

For starsight, traits are essential because the library supports multiple rendering backends. The tiny-skia backend rasterizes to a pixel buffer on the CPU. The SVG backend generates an XML document. The PDF backend produces a PDF file. The wgpu backend renders on the GPU. The terminal backend outputs escape sequences. Each of these backends needs to support the same set of drawing operations: fill a rectangle, stroke a path, draw text. But each implements these operations completely differently.

The solution is a trait called DrawBackend. It declares methods like fill-rect, draw-path, draw-text, dimensions, and save-png. Each backend implements the trait in its own way. The SkiaBackend implementation of fill-rect writes pixels into a Pixmap. The SvgBackend implementation writes an XML rectangle element into a document. Same method name, completely different behavior.

When a chart mark (like LineMark) needs to render itself, it does not need to know which backend it is talking to. It calls the DrawBackend methods and the correct implementation runs. This is the fundamental power of traits: they decouple the interface (what operations are available) from the implementation (how those operations work).

## Chapter 8: How trait objects enable runtime backend selection

There are two ways to use traits in Rust: static dispatch with generics, and dynamic dispatch with trait objects.

With generics, you write a function that is parameterized by a type that implements a trait. The compiler generates a specialized version of the function for each concrete type used. This is called monomorphization. The generated code is as fast as if you had written separate functions by hand, because the compiler knows the exact type at compile time and can inline the method calls. The downside is that the generic function must know the type at compile time.

With trait objects, you use the dyn keyword: dyn DrawBackend. A trait object is a fat pointer consisting of two machine-sized words (16 bytes on a 64-bit system). The first word points to the actual data. The second word points to a vtable, which is a table of function pointers generated at compile time for each concrete-type-and-trait pair. When you call a method on a trait object, the runtime loads the function pointer from the vtable and calls it indirectly. This is slightly slower than a direct call (because of the indirection and the inability to inline) but it enables runtime polymorphism: the concrete type is determined at runtime.

For starsight, trait objects are essential because the user decides the backend at runtime. When you call save with a file path, starsight checks the file extension: if it ends in png, it creates a SkiaBackend; if it ends in svg, it creates an SvgBackend. This decision happens at runtime, not at compile time. The Figure stores the backend as a dyn DrawBackend reference and calls methods through the vtable.

## Chapter 9: Why the DrawBackend trait must be object-safe

Not all traits can be used as trait objects. A trait is object-safe (also called dyn-compatible) if it follows certain rules. First, none of its methods can use Self as a parameter type or return type (because the concrete type is erased behind the trait object). Second, none of its methods can have generic type parameters (because you cannot store infinite vtable entries for every possible type parameter). Third, the trait cannot require the Sized bound on Self (because trait objects are unsized by definition).

The Plotters library made the mistake of requiring Sized on its DrawingBackend trait. This means you cannot write dyn DrawingBackend. Every function that accepts a backend must be generic over the backend type, which means every function in the call chain must also be generic, all the way up. This makes it extremely difficult to extract helper functions or store backends in data structures. It is one of the most common complaints about Plotters.

starsight avoids this by keeping DrawBackend object-safe from the beginning. No Self in return types, no generic type parameters on methods, no Sized bound. The render method on Scene takes ampersand mut dyn DrawBackend. This enables runtime backend selection, heterogeneous backend storage, and clean API design.

## Chapter 10: How generics work and when to use them instead of trait objects

Generics create specialized copies of code for each concrete type. When you write a function with a type parameter T that implements a trait bound, the compiler generates a separate version of the function for each T it encounters. The generated code is identical to hand-written specialized code: all method calls can be inlined, the optimizer can see through the abstractions, and there is no indirection overhead.

Use generics when: performance matters and inlining is important, the concrete type is known at compile time, and you do not need to store heterogeneous collections of different types.

Use trait objects when: you need to store different types in the same collection (like different mark types in a Vec), the concrete type is determined at runtime (like backend selection), or you want to reduce binary size (one function body shared across all types, instead of N copies).

For starsight, the rule is: concrete types at the bottom (where performance matters), trait objects in the middle (where heterogeneous collections are needed), and generics at the top (where user ergonomics matter). The DrawBackend trait is used through trait objects. The Mark trait is used through trait objects (a Figure stores Vec of Box of dyn Mark). The data acceptance functions in layer five use generics (impl Into of DataSource) for ergonomic callers.

## Chapter 11: How the From and Into conversion traits work

The From trait defines a conversion from one type to another. When you implement From of SomeType for YourType, you are saying: given a SomeType value, I can produce a YourType value, and this conversion always succeeds.

The Into trait is the reverse direction. Implementing From automatically provides Into through a blanket implementation in the standard library. You should always implement From rather than Into, because you get Into for free.

The practical use in function signatures is to accept impl Into of YourType as a parameter. This means the caller can pass either a YourType directly (the Into implementation is the identity) or any type that converts to YourType. For example, a function set-color that accepts impl Into of Color lets the caller pass a Color, a three-element tuple (if you implement From of tuple for Color), or a chromata Color (if you implement From of chromata Color for starsight Color).

TryFrom and TryInto are the fallible versions. Instead of always succeeding, they return a Result. Use TryFrom when the conversion can fail: for example, converting a string to a Color might fail if the string is not valid hexadecimal.

There is a rule called the orphan rule that limits where you can implement traits. You can only implement a trait for a type if your crate defines either the trait or the type (or both). This prevents two different crates from implementing the same trait for the same type, which would create ambiguity. For starsight, this means you can implement From of chromata Color for starsight Color (because starsight Color is your type), but you cannot implement From of starsight Color for tiny-skia Color (because neither type is yours). The workaround is to add a method like to-tiny-skia on your Color type instead of a trait implementation.

## Chapter 12: What the question mark operator does and how errors propagate

The question mark operator is a shorthand for error propagation. When you write a function call followed by a question mark, it means: if the result is Ok, unwrap the value and continue; if the result is Err, convert the error using From and return it from the current function.

This is enormously useful because it lets you write a chain of fallible operations without explicitly matching each result. Instead of writing four lines of match or if-let for every operation that might fail, you write one line with a question mark at the end. The errors propagate automatically.

The conversion part is key. If your function returns Result of T, StarsightError, and you call a function that returns Result of T, std io Error, the question mark operator calls From of io Error for StarsightError. This converts the io error into your error type. You just need to implement the From conversion (or use thiserror's from attribute to generate it automatically).

For starsight, nearly every function that interacts with the rendering backend, the file system, or external libraries returns a Result with StarsightError as the error type. The question mark operator propagates errors through the rendering pipeline without boilerplate.

## Chapter 13: How thiserror generates error types

thiserror is a library that provides a derive macro for the standard Error trait. You define your error enum with variants, annotate each variant with an error attribute containing the display message, and optionally annotate fields with from (to auto-generate From implementations) or source (to link to an underlying cause).

For starsight, the error type has seven variants: Render (for rendering backend failures), Data (for data format issues), Io (for file system errors, with a from attribute on the std io Error field), Scale (for invalid scale configurations), Export (for output format problems), Config (for invalid configuration), and Unknown (for unexpected situations). The from attribute on the Io variant means the question mark operator automatically converts io errors. The other variants take String messages that describe the specific failure.

The thiserror derive generates implementations of the standard Error trait (which provides method chaining through the source method), the Display trait (using the message format strings), and any From implementations specified by from attributes. This saves about 50 lines of boilerplate per error type.

The alternative to thiserror is anyhow, which provides a type-erased error type for application code. Libraries should use thiserror (to give callers typed errors they can match on). Applications should use anyhow (when you just need to report errors to the user). starsight is a library, so it uses thiserror.

---

## How to use this document

This document is designed to be listened to. Everything is plain prose. No tables, no code blocks, no formatting tricks.

Five arcs. Chapters 1 through 17 teach Rust: ownership, borrowing, traits, generics, errors, iterators, closures, lifetimes, modules. Chapters 18 through 21 teach computer graphics: pixels, lines, curves, anti-aliasing, color science. Chapters 22 through 28 teach visualization theory: grammar of graphics, scales, ticks, chart types, the data-to-pixel pipeline. Chapters 29 through 45 cover starsight architecture, tooling, code standards, and implementation. Chapters 46 onward are supplementary. The glossary at the end defines every term.

If you know Rust, skip to Chapter 18. If you know graphics, skip to Chapter 22. For starsight specifics only, skip to Chapter 29.

## Chapter 14: Iterators and how they process data

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

## Chapter 15: Closures and callbacks in rendering code

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

## Chapter 16: Lifetimes and references that stay valid

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

## Chapter 17: Modules and how code is organized

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

## Chapter 18: How pixels and screens work

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

## Chapter 19: How lines and curves are drawn

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

## Chapter 20: Anti-aliasing and why charts need it

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

## Chapter 21: Color science for visualization

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

## Chapter 22: The grammar of graphics

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

## Chapter 23: Scales and how data maps to visual space

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

## Chapter 24: Choosing the right chart for the data

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

## Chapter 25: The tiny-skia rendering library in depth

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

## Chapter 26: The cosmic-text library for rendering text

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

## Chapter 27: The data-to-pixels pipeline

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

## Chapter 28: The SVG rendering backend

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

## Chapter 29: Design patterns for library code

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

## Chapter 30: The scene graph pattern and builder design

Scene is a struct that holds a vector of SceneNode values. A SceneNode is an enum with variants for Path, Text, Group (with children and a transform), and Clip (with a rect and a child). The Scene does not know how to render itself. It is pure data. You build a Scene by pushing nodes into it, and then you hand the Scene to a backend which reads the nodes and renders them.

This is the pattern used by Vello (flat encoding), egui (clipped shapes list), and every modern Rust graphics library. The alternative, used by Plotters, is to make charts call backend methods directly during construction. That approach tangles chart logic with rendering logic, makes testing harder (you cannot inspect the scene without rendering it), and prevents optimizations like batching or reordering draw calls.

With a data-based scene, you can serialize it for debugging, compare two scenes for equality in tests, render the same scene to multiple backends without re-running the chart logic, and build the scene on one thread while rendering it on another.

The Figure builder uses mutable reference returns: each setter takes and mut self and returns and mut Self. This lets you chain calls or use them separately. The chained style looks like figure dot title of "Chart" dot x label of "Time" dot size of 800 comma 600. The separate style looks like: let mut fig equals Figure new, then on the next line fig dot title "Chart", then fig dot size 800 600.

This pattern was chosen over consuming self (where each method takes self by value and returns Self) because consuming self is awkward with conditional configuration. With mutable references, you can write: if show legend then fig dot legend of true. With consuming self, you would have to write: let fig equals if show legend then fig dot legend of true else fig. The consuming style also prevents partially configuring a builder, storing it, and configuring more later.

The exception is the build or save method, which does consume self (or borrows immutably and clones what it needs). This prevents accidentally modifying a figure after it has been rendered.

For mark types like LineMark and PointMark, the types are plain structs with public fields. No builder needed. You construct them with struct literal syntax. This is simpler and appropriate for types with a small number of fields where most fields are always specified.

## Chapter 31: Thread safety in a visualization library

Send means a value can be transferred between threads. Sync means a value can be shared (by reference) between threads. Most Rust types are automatically Send and Sync if all their fields are Send and Sync.

For starsight, Send and Sync matter for two reasons. First, users might want to render charts on a background thread to avoid blocking the UI. Second, the wgpu backend requires Send plus Sync for GPU resources.

The tiny-skia Pixmap type is Send but not Sync (it contains mutable state). This means you can move a SkiaBackend to another thread, but you cannot share it between threads without a mutex. This is fine for starsight's architecture because the rendering pipeline is sequential: build the scene, then render it. There is no need for concurrent access to the backend.

The Scene type should be Send and Sync because it is immutable data. Once built, it can be shared between threads. This enables a pattern where the scene is built on one thread and rendered on another.

The Figure type should be Send but does not need to be Sync because it is a builder that accumulates mutable state. You build a Figure on one thread and render it on the same thread or move it to another.

Make sure all public types are Send by default. Check with a compile-time assertion: const _ colon fn of unit where Figure colon Send equals open close curly braces. This is a zero-cost way to verify Send bounds.

---

## Chapter 32: Testing a visualization library

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

## Chapter 33: The Rust development toolchain

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

## Chapter 34: Continuous integration with GitHub Actions

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

## Chapter 35: The workspace structure and how crates connect

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

## Chapter 36: API design principles for a long-lived crate

The Rust API Guidelines checklist has about 70 items. Here are the ones most relevant to starsight, with specific guidance on how to apply them.

Types eagerly implement common traits. For starsight: every public struct should implement Debug (for print debugging and error messages), Clone (for users who want to modify a copy of a configuration), and Display where meaningful (for colors, points, errors). Send and Sync should be implemented or verified for types that users might want to move between threads.

Conversions use the standard From, Into, TryFrom, and TryInto traits. For starsight: Color implements From for chromata Color and prismatica Color. Point implements From for two-element arrays and tuples. Rect implements TryFrom for tiny_skia Rect (TryFrom because the conversion can fail if bounds are invalid). Use Into in function signatures for ergonomic callers.

Error types implement std Error. For starsight: StarsightError implements Error through thiserror's derive macro. The Display implementation provides human-readable messages. The source method links to underlying errors.

Builder methods are named well. For starsight: methods that create a new modified copy use the with prefix (with_alpha on Color). Methods that mutate in place use the set prefix or take and mut self. Methods that convert use the to or into prefix.

Public dependencies are re-exported. If starsight's public API exposes a type from tiny-skia (it should not, but if it ever does), the type must be re-exported so users do not need to add tiny-skia as a separate dependency. This is one more reason to wrap external types in your own types: it avoids the re-export requirement entirely.

Sealed traits prevent external implementations. If the DrawBackend trait should only be implemented by starsight's own backends (not by external crates), use the sealed trait pattern: add a method that returns a private type. External crates cannot implement the private method, so they cannot implement the trait. However, for starsight, DrawBackend should probably be implementable externally (a user might want to implement a custom backend for their own use), so do not seal it.

The orphan rule says: you can only implement a trait for a type if either the trait or the type (or both) is defined in the current crate. This prevents two crates from independently implementing the same trait for the same type, which would create ambiguity.

For starsight, the orphan rule affects color conversions. You want to implement From of chromata Color for starsight Color. This is allowed because starsight Color is defined in the current crate (starsight-layer-1). You also want to implement From of starsight Color for tiny_skia Color. This is NOT allowed because neither starsight Color nor tiny_skia Color is the standard From trait, and the From trait is from the standard library, not from your crate.

The workaround is a method instead of a trait implementation: add a to_tiny_skia method on starsight Color that returns a tiny_skia Color. This is not a From implementation, so the orphan rule does not apply. The downside is that you cannot use the Into syntax or the question mark operator for conversion. But since these conversions happen inside backend code (not in user-facing API), the ergonomic cost is acceptable.

Similarly, you cannot implement the ratatui Widget trait for a type defined in starsight unless the Widget trait is in scope. Since Widget is defined in the ratatui crate and starsight's widget type is defined in starsight, this IS allowed: the type is local. But if you wanted a type from prismatica to implement a trait from chromata, neither is local, and the orphan rule blocks it. This is why wrapper types (newtypes) exist: wrap the foreign type in a local newtype and implement the foreign trait on the newtype.

Adding non_exhaustive to a type that is already published is a breaking change. This is because downstream code that exhaustively matches on the enum or constructs the struct with literal syntax will no longer compile. Removing non_exhaustive is also a breaking change for structs (because it changes the struct's constructibility). For enums, removing non_exhaustive is technically not breaking (it only makes matches easier), but cargo-semver-checks may still flag it.

The practical rule for starsight: add non_exhaustive to every public enum and every public struct that might gain fields, before the first publish. Once it is on the type, adding new variants or fields is a non-breaking change in any future version.

The exception is types whose fields are their complete mathematical definition. Point (x, y), Vec2 (x, y), Color (r, g, b), and Size (width, height) have fields that are fundamental to what the type is. Adding a third field to Point would change it from a 2D point to something else entirely, which would be a redesign, not an incremental change. These types should not have non_exhaustive.

For configuration structs like RenderOptions or ThemeConfig, non_exhaustive is essential. You will definitely want to add fields like dpi, background_color, or font_family in future versions. With non_exhaustive, these additions are non-breaking.

For error enums like StarsightError, non_exhaustive is essential. You will discover new error conditions as you implement more backends and chart types. Adding a new variant like Gpu(String) or Font(String) should not break downstream match statements.

The tradeoff: non_exhaustive makes the API slightly less ergonomic. Users cannot construct the struct with literal syntax, so they need a constructor function. Users cannot exhaustively match, so they need a wildcard arm. But this tradeoff is overwhelmingly worthwhile for a pre-1.0 library that will evolve rapidly.

A common question when designing a Rust API is whether to make a function generic. For starsight, the answer depends on the layer.

In layer 1 (rendering), use concrete types. The DrawBackend trait methods take specific types: Path, PathStyle, Color, Rect, Point. Making them generic would add complexity without benefit. There is one Path type, one Color type, one Rect type. The backend implementations need to know exactly what they are receiving.

In layer 3 (marks), use trait objects where needed. The Mark trait is object-safe and marks are stored as Box dyn Mark in the Figure. This allows different mark types (LineMark, PointMark, BarMark) to coexist in the same marks vector without generics.

In layer 5 (high-level API), use generics on entry points. The data acceptance functions should accept impl Into DataSource, which enables passing a Polars DataFrame, a pair of slices, or an ndarray without the user explicitly converting. The builder methods should accept impl Into String for labels and titles.

The general rule: concrete types at the bottom (where implementation details matter), generic types at the top (where user ergonomics matter), trait objects in the middle (where heterogeneous collections are needed).

The prelude module re-exports the types that every user needs in every program. It should contain the types that appear in the most common usage pattern: use starsight prelude star, then call plot and save.

The prelude should export: Figure (the builder everyone uses), the plot macro (the one-liner everyone starts with), Color (needed to customize colors), Point (needed for manual positioning), StarsightError and Result (needed for error handling), and whatever trait is needed for save and show to work.

The prelude should not export: backend types (SkiaBackend, SvgBackend), internal types (PathCommand, PathStyle, SceneNode), mark types (LineMark, PointMark), scale types (LinearScale), or any type that is only needed for advanced compositional use. These live in the crate's module tree and users import them explicitly when needed.

The principle is: if a type appears in the getting started example, it belongs in the prelude. If it appears only in the advanced composition example, it does not. Overstuffing the prelude pollutes the user's namespace and causes name collisions. Understuffing it forces the user to write long import lists for basic operations.

## Chapter 37: Code standards for every line in the workspace

Every public struct and enum should derive Debug, Clone, and PartialEq at minimum. Debug is required for readable test failure messages and for users to println their chart configurations. Clone is required because users will want to create a chart configuration, modify it slightly, and render both versions. PartialEq is required for assertions in tests.

For types that represent values (colors, points, sizes), also derive Copy, Eq, and Hash. Copy is appropriate because these types are small (under 32 bytes) and there is no ownership semantic. Eq is appropriate because bitwise equality is meaningful for u8 color channels and f32 coordinates (with the caveat that NaN does not equal itself, but we handle that separately). Hash is needed for using colors as HashMap keys when batching draw calls by color.

For types that hold heap data (Figure, LineMark with Vec data), derive Debug and Clone but not Copy. Implement PartialEq if meaningful comparison exists.

Do not derive Default on types where the default is not useful. A default Point at zero zero is sensible. A default Figure with no data and no marks is not: it produces an empty chart with no axes and no content, which is never what anyone wants. If Default does not produce something useful, force the user to call a constructor.

The clippy configuration forbids unwrap_used and expect_used. These are panicking operations. A library should never crash the caller's program because a color string was malformed or a path was empty.

Use the question mark operator to propagate errors. Use ok_or_else to convert Option to Result. Use map_err to convert external error types to StarsightError. If an operation truly cannot fail (because you have already validated the inputs), use a comment explaining why and use the match or if-let pattern instead of unwrap.

The only permitted exception is in tests. Test code may use unwrap because a panic in a test is an expected failure mode. But even in tests, prefer the question mark operator with a test function that returns Result, because the error message from a propagated error is more informative than the generic "called unwrap on a None value" message.

If your DrawBackend trait has a method that takes a tiny_skia Point, you have coupled your public API to tiny-skia's versioning. When tiny-skia releases a breaking change, your API breaks too, even if your code is unchanged. This forces a major version bump for something you did not control.

Wrap external types in your own types. starsight has its own Point, Rect, Color, and Transform types specifically for this reason. The DrawBackend trait takes starsight types. The backend implementation internally converts to tiny-skia types. This insulates the public API from dependency churn.

The same principle applies to error types. StarsightError variants contain Strings, not tiny_skia::png::EncodingError or cosmic_text::SomeError. When a backend encounters a dependency-specific error, it wraps it in a StarsightError with a descriptive message. The dependency error type never leaks through the public API.

The user who writes cargo add starsight should get a working library with CPU rendering, SVG output, and PNG export. They should not be forced to compile wgpu, polars, ratatui, nalgebra, or any other heavyweight dependency they do not need.

Every optional dependency goes behind a feature flag. The feature flag is defined in the starsight facade crate's Cargo.toml and forwarded to the appropriate layer crate. When the user enables the gpu feature, the facade crate enables the gpu feature on starsight-layer-1, which activates the wgpu dependency and compiles the wgpu backend code.

Feature flags must be additive. Enabling a feature must never remove functionality. A crate compiled with all features enabled must work exactly the same as one compiled with the default features, plus additional capabilities. This means feature flags should never be used for exclusive choices (either wgpu or tiny-skia, but not both). Both backends are always available; the user chooses at runtime which to use.

Do not return mutable references from builder methods if the builder will be consumed later. If the Figure builder returns and mut Self from title() but then save() takes self by value, the user has to call save on a temporary, which is syntactically awkward. Either make all methods take and mut self and have save take and self, or make all methods take self by value and have save also take self by value.

Do not use type aliases to hide complexity. If a function returns Result of Vec of Box dyn Mark plus Send plus Sync, StarsightError, do not create a type alias MarkList that hides the Box dyn part. Users need to see the boxed trait object to understand the ownership and dynamic dispatch implications. Type aliases are appropriate for Result T StarsightError (because every function in the crate uses this pattern) but not for application-specific composed types.

Do not add a method to a trait when a free function or a blanket impl would work. Every method on the DrawBackend trait requires every backend to implement it. If a method can be implemented in terms of other trait methods (like drawing a dashed rect by drawing four dashed lines), provide a default implementation so backends get it for free.

starsight uses Conventional Commits. Every commit message starts with a type, an optional scope, and a description. The type determines how git-cliff categorizes the commit and how cargo-release determines the version bump.

The type feat indicates a new feature. It maps to a minor version bump under semver. The type fix indicates a bug fix. It maps to a patch version bump. The type perf, refactor, docs, test, and chore are informational and do not trigger version bumps. The type feat with an exclamation mark (feat bang) or a BREAKING CHANGE footer indicates a breaking change, which maps to a major version bump (or minor in pre-1.0).

The scope is the area of the codebase affected. For starsight, useful scopes are layer-1, layer-2, primitives, scale, backend, skia, svg, tick, and ci. The scope appears in parentheses after the type: feat layer-2 colon implement log scale.

The description is imperative mood, lowercase, no period. "add linear scale support" not "added linear scale support" and not "adds linear scale support." The description should complete the sentence "this commit will" followed by the description.

Bad commit messages: "fix stuff", "wip", "updates", "more changes." These tell you nothing about what changed or why. Good commit messages: "fix layer-2 colon correct Y axis inversion in CartesianCoord," "feat layer-1 colon implement SVG backend fill_rect," "test layer-1 colon add snapshot for blue rect on white."

---

## Chapter 38: How versioning and deprecation work

During the pre-1.0 phase, the API changes frequently. New features are added, design mistakes are corrected, type signatures are improved. But users adopt the library from the first published version. Every change you make to the API requires every user to update their code.

The tension is between API quality (improving the design by changing things) and user convenience (not breaking things). The resolution is to front-load the hardest design decisions: get the primitive types right before publishing 0.1.0, get the trait interfaces right before publishing 0.2.0, and get the builder patterns right before publishing 0.3.0. Once these foundations are stable, later versions can add features (new chart types, new backends, new data sources) without changing existing interfaces.

The specific things to get right early: the fields and methods on Point, Vec2, Rect, Color (because these types appear everywhere and are copied into user code), the methods on DrawBackend (because backend implementors depend on them), the methods on Scale and Mark (because every chart type and scale type depends on them), and the signature of the plot macro (because it is the first thing in every tutorial).

The specific things that can change later without much pain: the Figure builder's method names (builders are called in one place, easy to update), the layout algorithm's behavior (affects visual output but not API), the internal module structure of each layer crate (users only see the facade re-exports), and the set of available chart types (adding new ones is never breaking).

Use the pre-1.0 period wisely. This is the time when breaking changes are socially acceptable. After 1.0, every breaking change requires a major version bump, which fractures the ecosystem. Make the hard decisions now so that 1.0 is a stable foundation for years of compatible evolution.

Rust has built-in deprecation support via the deprecated attribute. When you mark a function as deprecated, any code that calls it produces a compiler warning. The warning includes the deprecation message, which should tell the user what to use instead.

For starsight, the deprecation cycle works like this. In version 0.2.0, you realize that the draw_path method on DrawBackend should take a reference to a PathStyle, not an owned PathStyle. You cannot just change the signature because that breaks all existing backend implementations. Instead: in 0.2.0, add a new method draw_path_ref that takes a reference. Mark the old draw_path as deprecated with a note saying "use draw_path_ref instead; draw_path will be removed in 0.4.0." Provide a default implementation of draw_path that calls draw_path_ref. In 0.4.0, remove draw_path.

This gives users two full releases to migrate. The deprecation warning is visible but not blocking (it is a warning, not an error, unless the user has turned warnings into errors). The migration path is clear: find all calls to draw_path, change them to draw_path_ref.

In the changelog, deprecations appear under the Deprecated heading. Removals appear under the Removed heading in the version where the deprecated item is finally removed. Each removal entry should reference the version where the item was deprecated and the replacement.

---

## Chapter 39: Publishing to crates.io

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

## Chapter 40: Your first coding session

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

## Chapter 41: Managing complexity and motivation

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

## Chapter 42: The 0.1.0 MVP

The exit criteria for 0.1.0 is: plot exclamation of array 1.0, 2.0, 3.0 comma array 4.0, 5.0, 6.0 dot save "test.png" produces a correct line chart. This is not a full visualization library. It is the minimum vertical slice that proves the architecture works.

To get there, you need: the primitive types (Point, Vec2, Rect, Color, Transform), the tiny-skia backend (creating a pixmap, drawing paths, filling rects, rendering text, saving PNG), the SVG backend (at least fill_rect and save_svg), a linear scale, the Wilkinson tick algorithm, a Cartesian coordinate system, axis rendering (tick lines, tick labels, axis labels), a line mark, the Figure builder, the plot macro, and snapshot tests proving it all works.

You do not need: log scales, categorical scales, bar charts, histograms, box plots, faceting, legends, GPU rendering, terminal rendering, interactivity, streaming data, PDF export, WASM, Polars integration, ndarray, Arrow, or any of the 60 chart types beyond basic lines and points.

Resist the temptation to add features before the vertical slice is complete. A library that renders one chart type correctly and has tests is more valuable than a library that has stubs for 60 chart types and renders nothing.

## Chapter 43: Debugging charts

When a chart renders incorrectly, the bug is at one of the pipeline boundaries. Here is how to isolate it.

First, check the data. Print the raw values. Are they what you expect? Are there NaN values or infinities? Are the x and y arrays the same length?

Second, check the scales. Print the domain min and max. Are they reasonable? Did the Wilkinson tick algorithm produce sensible tick positions? If the ticks look wrong, the scale domain is wrong, which means the data range computation is wrong.

Third, check the coordinate mapping. Pick a known data point and manually compute its expected pixel position using the formula: pixel x equals plot area left plus normalized x times plot area width, pixel y equals plot area bottom minus normalized y times plot area height. Does the actual pixel position match?

Fourth, check the path commands. Before sending them to the backend, print the PathCommand sequence. Are the move to and line to positions correct? Are there unexpected NaN values producing gaps?

Fifth, check the rendering. Render to SVG instead of PNG. Open the SVG in a browser and inspect the elements. SVG is human-readable. You can see the exact coordinates, colors, and transforms applied to each element. If the SVG looks correct but the PNG does not, the bug is in the tiny-skia backend translation.

Sixth, check clipping. Temporarily disable the mask (pass None instead of the plot area mask). If elements appear that were missing, the clipping rect is wrong, which means the margin or plot area computation is wrong.

The snapshot test approach helps here too. When you fix a visual bug, the snapshot test captures the corrected output. If the bug regresses, the snapshot comparison fails immediately.

## Chapter 44: Building starsight step by step

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

## Chapter 45: How to think about performance

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

## Chapter 46: How to handle edge cases gracefully

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

## Chapter 47: How to write good documentation

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

## Chapter 48: The resonant-jovian ecosystem

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

## Chapter 49: Language and licensing choices

Rust edition 2024 (shipped with Rust 1.85) changed several things relevant to starsight. The gen keyword is now reserved for future generators, so any identifier named gen must become r#gen. The unsafe_op_in_unsafe_fn lint is now warn by default, meaning unsafe operations inside unsafe functions need explicit unsafe blocks. RPIT (return position impl trait) lifetime capture rules changed: functions returning impl Trait now capture all in-scope lifetimes by default, which can affect public API signatures.

Resolver 3 (implied by edition 2024) adds MSRV-aware dependency resolution. If a dependency's latest version requires a newer Rust than your declared rust-version, Cargo falls back to an older compatible version. Feature unification behavior is unchanged from resolver 2.

starsight is GPL-3.0-only. Not MIT, not Apache-2.0, not dual-licensed. This is an intentional choice. The sister crates chromata and prismatica are also GPL-3.0. The license is viral: any program that links starsight into its binary must also be distributed under GPL-3.0 or a compatible license. This means proprietary applications cannot use starsight without releasing their source code.

For the codebase, this means every dependency must be GPL-3.0 compatible. MIT, Apache-2.0, BSD, ISC, Zlib, and similar permissive licenses are all compatible. LGPL is compatible. Proprietary licenses and SSPL are not. The deny.toml file configures cargo-deny to check this: any dependency with an incompatible license will fail CI.

The practical impact during development: before adding a new dependency, check its license. All current workspace dependencies (tiny-skia is BSD-3, cosmic-text is MIT/Apache-2.0, thiserror is MIT/Apache-2.0, image is MIT/Apache-2.0, svg is MIT/Apache-2.0) are permissive and therefore GPL-compatible.

Rust's async ecosystem is powerful but adds significant complexity: every async function requires an executor runtime (tokio, async-std, smol), error types must be Send and Sync, and the colored function problem means async infects every call site above it.

starsight is a visualization library, not a network service. Its operations are CPU-bound (rasterization, layout computation, text shaping), not I/O-bound (waiting for network responses, reading files). CPU-bound work does not benefit from async. An async rasterizer is just a synchronous rasterizer with extra overhead.

The one place where async might seem natural is streaming data: receiving sensor readings from an async channel and updating a chart. starsight handles this with a push-based synchronous API instead. The user calls append from their own async context (or synchronous context, or signal handler, or whatever). The figure does not know or care whether the caller is async.

This design means starsight has zero dependency on any async runtime. It works equally well in a tokio application, a bare metal embedded system, a WASM browser environment, and a simple synchronous command-line tool. Adding a tokio dependency to a visualization library would be an architectural mistake that constrains every downstream user.

---

## Chapter 50: Ecosystem positioning and accessibility

The Rust ecosystem for data science and visualization is growing but fragmented. starsight positions itself as the comprehensive solution that bridges the gap between quick-and-dirty plotting (textplots, plotters) and full-featured interactive dashboards (plotly-rs, which bundles JavaScript).

The closest competitors are plotters (the most mature Rust plotting library, with good API documentation but limited chart types, stagnating development, and the Sized bound issue described earlier), plotly-rs (which generates Plotly.js charts and requires a JavaScript runtime or opens a browser tab), charming (which generates ECharts configurations and has the same JavaScript dependency), and egui_plot (which is excellent but locked to the egui framework).

starsight's differentiator is: no JavaScript runtime, no C dependencies in the default build, 66 chart types from a single library, both static export and interactive native windows, terminal rendering, GPU acceleration, and deep integration with the Rust data science stack (Polars, ndarray, Arrow). No existing Rust library offers all of these.

The risk is scope. Building a library this comprehensive takes years. Many Rust visualization projects have been abandoned after the initial enthusiasm. starsight mitigates this risk with a narrow initial scope (0.1.0 is just line charts and scatter plots), a sustainable development pace, and an architecture that allows incremental expansion without restructuring.

The opportunity is timing. The Rust data science ecosystem is maturing rapidly. Polars is approaching feature parity with pandas. ndarray is stable and widely used. Arrow support is standardized. The missing piece is visualization. The first Rust visualization library that reaches maturity will become the default choice for the ecosystem, just as matplotlib became the default for Python. starsight aims to be that library.

Accessibility in data visualization means ensuring that charts communicate information to people with visual impairments, including color blindness, low vision, and blindness.

For color blindness (affecting about 8 percent of men and 0.5 percent of women), the primary mitigation is using color palettes that are distinguishable by people with all common forms of color vision deficiency. The most common form, deuteranopia, makes red and green appear similar. Protanopia has a similar effect but with different wavelengths. Tritanopia makes blue and yellow appear similar.

prismatica's perceptually uniform colormaps are designed with color vision deficiency in mind. The viridis colormap (and its variants inferno, magma, plasma) were specifically created to be distinguishable by people with all common forms of color blindness. starsight should default to these colormaps rather than rainbow colormaps (like jet) which are notoriously bad for accessibility.

Beyond colormaps, starsight should support redundant encoding: using both color and shape, or both color and pattern, to distinguish data series. A scatter plot where series A is blue circles and series B is orange squares is accessible to color-blind users because the shape distinction is sufficient even if the colors look similar.

For low vision, starsight should support configurable font sizes, line widths, and point sizes. The default values should be large enough to read at typical viewing distances (14 pixel font minimum for screen, 10 point minimum for print). High-contrast modes (black on white, white on black) should be available.

For blindness, the most accessible approach is to provide the underlying data table alongside the chart. This is straightforward for HTML export (include a hidden table element that screen readers can access) but not possible for static image formats. An alternative is to generate a text description of the chart: "Line chart showing temperature from January to December. The minimum is 5 degrees in January, the maximum is 32 degrees in July."

These accessibility features are not planned for 0.1.0 but should inform design decisions from the start: do not hardcode colors that are only distinguishable by people with full color vision, do not hardcode font sizes that are too small, and design the API so accessibility options can be added later without breaking changes.

## Chapter 51: Long-term maintenance and sustainability

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

## Chapter 52: What happens after 1.0

You now have the complete mental model for building starsight. The architecture is seven layers, each a separate crate, with strict dependency direction. The rendering pipeline goes from data to marks to scales to coordinates to path commands to backend to pixels. The color pipeline goes from user specification to sRGB Color to tiny-skia premultiplied pixels. The text pipeline goes from string to cosmic-text shaped glyphs to per-pixel callback to pixmap fill_rect. The testing strategy is snapshot tests for visual output, property tests for mathematical invariants, and unit tests for everything else.

The tools are: rustfmt for formatting, clippy for linting, cargo-deny for dependency governance, cargo-semver-checks for API compatibility, cargo-insta for snapshot testing, cargo-llvm-cov for coverage, cargo-nextest for fast test execution, cargo-hack for feature flag verification, git-cliff for changelogs, taplo for TOML formatting, criterion for benchmarks, and cargo-flamegraph for profiling.

The rules are: no unsafe in layers 3 through 7, no panics in library code, no println or eprintln, no async, no JavaScript dependencies, no C dependencies in the default feature set, no nightly-only features. Every public type derives Debug and Clone. Every public item has a doc comment. Every error is a StarsightError. Every feature-gated module is behind a cfg attribute at the module level.

Start with the vertical slice. Get plot save to produce a PNG. Everything else follows from there.

## Chapter 53: DPI and resolution handling

Charts need to render at different resolutions depending on the output target. A screen display might be 96 DPI. A retina display is 192 DPI. A print PDF is 300 DPI. A poster is 600 DPI.

starsight separates logical size from physical size. The user specifies the chart size in logical pixels (800 by 600). The rendering pipeline multiplies by a scale factor to get physical pixels. A scale factor of 1.0 gives 800 by 600 physical pixels (for screen). A scale factor of 3.75 gives 3000 by 2250 physical pixels (for 300 DPI print at the same logical size).

Font sizes, line widths, and point radii are all specified in logical units and scaled by the same factor. A 12-pixel font at scale factor 1.0 is 12 physical pixels. At scale factor 3.75, it is 45 physical pixels. This ensures charts look the same at all resolutions, just sharper at higher DPI.

The tiny-skia backend creates the Pixmap at the physical size and applies a Transform that scales all drawing operations by the scale factor. This is transparent to the marks and layout system, which always work in logical coordinates.

For SVG output, DPI does not apply because SVG is resolution-independent. The viewBox is set to the logical size, and the SVG renderer handles scaling to the display resolution.

---

## Chapter 54: The xtask pattern

The xtask crate is a binary that lives in the workspace but is never published. It automates development tasks that are too complex for shell scripts but do not belong in the library code. The standard Rust convention is to run these tasks with cargo xtask followed by a subcommand.

For starsight, xtask will eventually handle: generating the gallery (running all examples and collecting their PNG output into a directory), running benchmarks (rendering a standard set of charts and measuring time and memory), checking that all example files compile, and preparing release artifacts. For now, its main dot rs is an empty function. Fill it in as the need arises.

The xtask pattern works because Cargo allows running binaries from workspace members without installing them. You add a .cargo/config.toml file at the workspace root with the alias: xtask equals run minus minus manifest-path xtask/Cargo.toml minus minus. Then cargo xtask gallery runs the xtask binary with the gallery argument.

---

## Chapter 55: Default themes and sensible defaults

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

## Chapter 56: What makes a chart effective versus misleading

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

## Chapter 57: Accessibility in visualization

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

## Chapter 58: Everything comes together

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

## Chapter 59: How affine transforms compose and why order matters

An affine transform is a mathematical operation that combines translation (shifting), scaling (stretching), rotation (spinning), and skewing (shearing) into a single operation. It is represented by six numbers arranged conceptually as a matrix. Every drawing operation in tiny-skia can have a transform applied, which repositions the output without modifying the source path or shape.

The crucial thing about transforms is that the order of composition matters. Translating by 100 pixels and then rotating 45 degrees produces a different result than rotating 45 degrees and then translating 100 pixels. In the first case, you move to a new position and then rotate around the new position. In the second case, you rotate around the origin and then translate, which moves you in a rotated direction.

tiny-skia provides two sets of composition methods: pre-methods and post-methods. A pre-method applies the new operation before the existing transform. This means the new operation acts on the input points first, and then the existing transform acts on the result. A post-method applies the new operation after the existing transform. The existing transform acts first, and then the new operation acts on its output.

For starsight, transforms are used in two main places. First, for DPI scaling: the entire chart is scaled by a factor (like 2 for retina displays) using a single transform on every draw call. Second, for rotated text: Y-axis labels are typically rotated 90 degrees counterclockwise, which requires translating to the label position, then rotating. Getting the order wrong produces labels that appear in the wrong location or at the wrong angle.

The Transform identity is the do-nothing transform. Composing any transform with identity produces the original transform unchanged. This is what you pass when you do not want any transformation.

## Chapter 60: How PNG compression works at a high level

When starsight saves a chart as a PNG file, the pixel data goes through a two-stage compression pipeline.

Stage one is filtering. Each row of pixels (called a scanline) is independently filtered to reduce redundancy. There are five filter types: None passes the bytes through unchanged. Sub stores the difference between each byte and the corresponding byte in the pixel to the left. Up stores the difference from the pixel directly above. Average uses the average of the left and above pixels. Paeth uses a more complex predictor that selects whichever of left, above, or upper-left is closest to a computed value. The filter is chosen per row to minimize the resulting byte values (values near zero compress better).

Stage two is Deflate compression, the same algorithm used in gzip and zip files. It combines two techniques: LZ77 (finding repeated byte sequences within a sliding window and replacing them with back-references to earlier occurrences) and Huffman coding (replacing fixed-length symbols with variable-length codes based on frequency, so common symbols get short codes).

The filtering stage is what makes PNG compression effective for chart images. Chart images typically have large areas of solid color (backgrounds, bars, filled regions) where adjacent pixels are identical. The Sub or Up filter reduces these areas to runs of zeros, which Deflate compresses extremely well. This is why a chart PNG with a white background and a few colored lines is much smaller than a photograph of the same pixel dimensions.

For starsight, PNG encoding is handled by tiny-skia's encode-png method. You do not interact with the filtering or compression directly. But understanding the process explains why different chart designs produce different file sizes and why adding unnecessary gradients or textures to charts bloats the file.

## Chapter 61: How SVG differs from raster graphics

A raster image (like PNG) stores individual pixel values. An SVG (Scalable Vector Graphics) document stores a description of shapes: draw a line from here to there, fill this rectangle with this color, place this text at these coordinates. The SVG viewer (a web browser, an image viewer, or a rasterizer library) interprets the description and renders it at whatever resolution is needed.

The advantage of SVG for charts is resolution independence. A PNG chart at 800 by 600 pixels looks blurry when zoomed in. An SVG chart at any nominal size looks sharp at any zoom level because the viewer re-rasterizes the shapes at the target resolution. This makes SVG ideal for web embedding and print publication.

SVG is an XML format. Each visual element is an XML element: rect for rectangles, path for arbitrary shapes, text for text, circle for circles, g for groups. Attributes control appearance: x and y for position, fill for interior color, stroke for outline color, stroke-width for outline thickness, font-size for text size.

The SVG coordinate system uses the same convention as screen graphics: origin at the top-left, x increasing right, y increasing down. The viewBox attribute defines the coordinate system: viewBox equals "0 0 800 600" means the internal coordinates span from (0,0) to (800,600). The actual display size is set by the width and height attributes on the root svg element.

For starsight, the SvgBackend generates SVG documents using the svg crate. Instead of rasterizing shapes onto pixels, it creates XML elements. A fill-rect call adds a rect element. A draw-path call adds a path element with d attribute containing the path data (M for MoveTo, L for LineTo, C for CubicTo, Z for Close). A draw-text call adds a text element.

The major limitation of SVG for charts is text measurement. When you generate an SVG, you need to know how wide a text label is in pixels (to compute margins and avoid overlaps). But the width depends on the font, which depends on what fonts are installed on the viewer's system. A browser in Japan might use a different font than a browser in Sweden, producing different text widths. starsight works around this by estimating text width (digits are about 0.55 times the font size, average characters about 0.6 times) and accepting the approximation. For precise measurement, use the PNG backend with cosmic-text, which shaping engine provides exact glyph widths.

## Chapter 62: How the Rust type system prevents unit confusion

One of the most common bugs in graphics code is mixing up coordinate systems: passing a pixel value where a data value is expected, or using screen Y (increasing downward) where mathematical Y (increasing upward) is expected. These bugs do not cause compiler errors with plain f32 values because the compiler sees all f32s as interchangeable.

The newtype pattern solves this. You define a struct with a single field, like: struct Pixels wrapping f32, and struct DataUnits wrapping f32. These are distinct types. A function that expects Pixels will not accept DataUnits, and vice versa. The compiler catches the mismatch at compile time, before the code ever runs. The newtype has zero runtime overhead: the wrapping struct compiles to exactly the same machine code as a bare f32.

For starsight, the Point and Vec2 distinction is a practical example of this principle. Point represents a position in space. Vec2 represents a displacement (a direction and magnitude). They are both two f32 fields, but they support different operations. Point minus Point gives Vec2 (the distance between two positions). Point plus Vec2 gives Point (shifting a position). Point plus Point is a compile error (adding two positions is meaningless). Vec2 times f32 gives Vec2 (scaling a displacement). Point times f32 is a compile error (scaling a position is meaningless).

This catches real bugs. In layout code, you deal with positions (where to place a label) and offsets (how much margin to add). If they were both plain f32 tuples, nothing prevents accidentally adding two positions together and getting garbage coordinates. With separate types, the compiler catches this immediately.

## Chapter 63: How the standard library traits interact with each other

The standard traits (Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Display) have specific relationships and rules.

Copy requires Clone. This makes sense: if a type can be trivially bit-copied, it can certainly be cloned. The reverse is not true: many types can be cloned (deeply copied) but not bit-copied because they own heap memory.

Eq requires PartialEq. Eq adds the reflexivity guarantee (a equals a for all a). Types that implement Eq can be used in more contexts, like HashMap keys.

Hash requires Eq (conceptually, though the compiler does not enforce this). The invariant is: if a equals b, then hash of a must equal hash of b. Types that implement PartialEq but not Eq (like f32) should not implement Hash because NaN violates the invariant.

Ord requires PartialOrd and Eq. Ord provides total ordering (every pair of values is comparable). PartialOrd allows incomparable values (like NaN compared to any number). Types that implement Ord can be sorted deterministically and used in BTreeMap.

Display provides human-readable formatting (using the curly-brace format specifier). Debug provides developer-readable formatting (using colon-question-mark). It is common to derive Debug but implement Display manually, because Debug has a reasonable derived form but Display is type-specific (Color should display as a hex string, not a struct dump).

For starsight, the standard trait derivations are: Point, Vec2, Rect, Size, and Color derive Debug, Clone, Copy, and PartialEq. Color also derives Eq and Hash (u8 channels support both). Transform derives Debug, Clone, Copy, and PartialEq. StarsightError derives Debug (through thiserror, which also generates Display and Error). Figure derives Debug and Clone. Mark implementations derive Debug and Clone.

## Chapter 64: How Result and Option combine with iterators

One of Rust's most powerful patterns is collecting an iterator of Results into a Result of a collection. If you have an iterator where each element is Result of T comma E, you can collect into Result of Vec of T comma E. This gathers all the Ok values into a Vec, or returns the first Err if any element fails. This pattern is extremely useful for processing data where each element might be invalid.

For starsight, this pattern appears when parsing data. If you have a column of strings that should be parsed as floating-point numbers, you iterate over the strings, call parse on each one (which returns Result), and collect into Result of Vec of f64. If all strings parse successfully, you get Ok of a Vec of f64. If any string is invalid, you get Err with the parse error, and none of the valid values are wasted on a partial result.

Similarly, Option works with iterators through filter-map. If you have an iterator and a function that returns Option (Some for items you want, None for items you want to skip), filter-map applies the function and yields only the Some values. This is perfect for processing data with missing values: map NaN to None, filter-map to remove them, and process the remaining valid values.

The flatten method unwraps one level of nesting: an iterator of Option of T becomes an iterator of T (skipping Nones), and an iterator of Vec of T becomes an iterator of T (concatenating the Vecs). This is useful for processing nested data structures.

## Chapter 65: How Rust's module privacy protects internal implementation

The module system's privacy rules serve a specific purpose in library design: they let you change internal code without breaking external users. If a function or type is private (not pub), external code cannot depend on it, which means you can rename it, change its signature, or delete it without a semver violation.

For starsight, this means: the internal conversion between starsight Color and tiny-skia Color should be a private method or a pub-crate function, not a fully public one. The internal layout computation algorithm should be private. The internal path construction for each mark should be private. Only the types and methods that users need to interact with should be public.

The prelude module (starsight prelude) re-exports the minimal set of types needed for basic usage: Figure, the plot macro, Color, Point, StarsightError, and Result. Everything else lives in the crate's module tree and is imported explicitly by users who need advanced functionality. This keeps the namespace clean for the common case while providing full access for power users.

## Chapter 66: How to think about memory allocation in rendering code

Memory allocation (asking the operating system for memory) is relatively expensive compared to computation. In a tight rendering loop that processes thousands of data points, unnecessary allocations can dominate the runtime.

The primary sources of allocation in starsight's rendering pipeline are: the Pixmap (one large allocation per render, around 2 megabytes for an 800 by 600 chart), PathBuilder's internal buffer (one allocation per path, growing as commands are added), Vec of PathCommand (one allocation per mark's render call), cosmic-text's FontSystem (one large allocation on initialization), and the PNG encoder's output buffer (one allocation per encode).

For version 0.1.0, do not optimize allocations. Use straightforward Vec and String allocations everywhere. The code should be clear and correct first. Profile with cargo-flamegraph after correctness is established, and optimize only the actual hotspots.

The most likely optimization targets (after profiling, not before) are: reusing PathBuilder buffers between marks instead of creating new ones for each mark, pre-allocating the path command Vec with an estimated capacity based on the data size, caching shaped text so the same tick label is not shaped repeatedly, and reusing the Pixmap between renders in interactive mode instead of allocating a new one each frame.

## Glossary

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
