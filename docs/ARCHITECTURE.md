# Architecture

How OfficeWorld is put together. Six parts are built: motion, geometry,
colour, paths, the reference renderer and compression. The rest of this
page is the plan the code is written
towards. It changes when building a part teaches something, and each part
gets its own section here once it exists.

## The layers

```
  apps        Docs  Sheets  Slides  Design  Board  PDF  Forms
              Drive  Mail  Calendar  Contacts
  ---------------------------------------------------------------
  shell       windows, tabs, the command palette, settings
  ---------------------------------------------------------------
  model       the one document model, its history and merging;
              the file formats translate into it and out of it
  ---------------------------------------------------------------
  engine      text layout, the canvas, formulas
  ---------------------------------------------------------------
  interface   layout, widgets, input, accessibility, motion
  ---------------------------------------------------------------
  renderer    a scene of shapes, images and text, drawn on the GPU
  ---------------------------------------------------------------
  platform    a window and its events, clipboard, files, fonts:
              Windows, macOS, Linux, or a browser
```

A layer uses the layers below it and knows nothing of the ones above.
What is built today sits at the bottom: motion, and what the renderer will
draw with.

## Rules that hold in every layer

**One model.** An app does not own a file format or a kind of document.
It is a view of the shared model with its own tools. That is what lets a
range of a sheet live inside a page of text.

**Nothing that changes the result is left to the platform.** Line
breaking, page breaking, font fallback and rounding are OfficeWorld's own
code and give the same answer everywhere. A document that fits on one page
on Linux fits on one page on Windows.

**Time is passed in.** Nothing below the shell reads a clock. The shell
names the time of the frame it is about to draw, and everything that moves
answers for that time. A test can therefore ask for any moment, and an
animation is the same at 60 Hz and at 144 Hz.

**Nothing is drawn while nothing moves.** Every animation knows when it
has come to rest. A window with nothing in motion asks for no frames and
uses no power.

**The file on your disk is the truth.** Opening, editing and saving need
no account and no network. Sync copies changes between devices that
already hold the whole document. It is never the place where the document
lives.

**The interface says what it shows.** An interface that draws itself is
invisible to a screen reader unless it describes what is on screen. That
description is part of every widget from the first one, not a later pass.

## Motion

`crates/motion`, package `ow-motion`. No dependencies and no unsafe code.
It draws nothing and keeps no clock.

**Springs** are the default way anything moves. A spring is the equation
`mass * x'' + damping * x' + stiffness * x = 0`, where `x` is the distance
from the target. `Spring::transition(elapsed)` solves it in closed form
and returns four numbers: how much of the old distance and the old
velocity is left in the new distance and the new velocity. The equation is
linear, so the same four numbers move every coordinate of a point and
every channel of a colour.

One formula covers the three regimes. With `envelope = exp(-decay * t)`:

```
x(t) = envelope * (x0 * c(t) + (v0 + decay * x0) * s(t))
```

For a spring that swings past its target, `c` is cos and `s` is sin
divided by its frequency. For one that creeps in, they are cosh and sinh
divided by its rate. Between the two they are 1 and `t`, which is what
both turn into as the frequency goes to zero. For the creeping spring the
two exponentials are multiplied out, because cosh alone grows past what a
64-bit float holds while the product is still an ordinary number.

A spring is only built from constants the solution can be computed with:
finite, above zero, and not so far apart that squaring them overflows. A
span of time that is negative or not a number counts as none.

**Rest** is decided by energy. `Spring::is_at_rest` asks whether the
energy left could still carry the value further than a tolerance from its
target. A damped spring only loses energy, so the answer never flips back
to no, and that is the moment an animation stops asking for frames.

**`Animated<T>`** is a value with a target. A new target, a new spring or
a velocity handed over by a gesture takes effect from where the value is
at that moment, at the speed it has. Restarting the stored state does not
bend the path, so the motion does not depend on which frames were drawn. A
clock that steps back does not rewind it.

**`CubicBezier` and `Tween<T>`** are for motion whose length is given: a
fade, or an animation that a presentation file describes by its duration.
The curves are the cubic Bézier form CSS and the office formats use. The
solver is Newton's method inside a bracket, so a curve that stands still
in time cannot throw it off. A tween returns exactly the value it was sent
to at its end, and it is refused when its duration is negative or not
finite.

**`Animatable`** is what a value needs for all of this: adding, scaling
and a length. It is implemented for `f32`, `f64` and arrays of them, and
the geometry and colour crates implement it for their types.

## Geometry

`crates/geometry`, package `ow-geometry`. It depends on motion alone, for
`Animatable`.

Coordinates are `f64`, x to the right and y downwards, as on a screen and
on a page. A sheet can be a million rows tall, and that far from the
origin an `f32` no longer tells two neighbouring pixels apart.

A `Point` is a place and a `Vector` is the way from one place to another.
They are two types, so the compiler catches an offset used as a place. A
`Rect` is stored as its four edges and is half open: its left and its top
edge belong to it, its right and its bottom edge do not. Two cells side by
side share an edge and no point, so a click on the line between them lands
in exactly one. A `RoundedRect` fits radii that are too large for it the
way CSS does. `Affine` is the matrix that CSS, SVG and PDF write as
`matrix(a, b, c, d, e, f)`.

Points, vectors, sizes and rectangles can be moved by a spring.

## Colour

`crates/color`, package `ow-color`. It depends on motion alone.

`Color` is what files and screens speak: sRGB, with alpha kept apart from
the colour. It reads hexadecimal notation, lays one colour over another,
and measures luminance and contrast the way the accessibility guidelines
(WCAG) define them.

`Oklab` is the space colours are mixed in. In sRGB a blend from blue to
yellow passes through grey, and equal steps do not look equal. Oklab was
made so that they do. Its polar form, lightness, chroma and hue, is what
palettes are built from. A colour that no screen can show is brought back
by lowering its chroma, which keeps its lightness and its hue. The numbers
are checked against the ones the CSS Color specification gives.

## Paths

`crates/path`, package `ow-path`. It depends on geometry alone.

A `Path` is an outline of straight lines and of quadratic and cubic Bézier
curves, in one or more pieces: the shape of a letter, the line of a chart,
the border of a cell. The crate draws nothing. It builds paths, measures
them and turns them into the simpler shapes that drawing needs.

**Building.** A path is built step by step, with the rules an HTML canvas
has for it. It is read from SVG path data, every command of it, and that
is also how it prints. Or it is one of the shapes: a rectangle, a rounded
rectangle, an ellipse, a polygon. An arc of an ellipse becomes one cubic
curve for each quarter turn, which strays from the ellipse by less than
three ten-thousandths of its radius.

**Flattening.** Drawing and measuring work on straight lines. A curve is
cut into equal steps, as many as Wang's formula asks for: from how fast
the curve changes direction it bounds how far the curve can stray from its
chords. The bound is never too low, so the lines stay within the tolerance
they were asked for.

**Measuring.** Bounds are tight: they come from where the curve turns
around, not from its control points. The area is exact, computed on the
curves themselves. Whether a point is inside is decided by how often the
path winds around it, under either fill rule, and an edge belongs to a
shape exactly as it belongs to a `Rect`.

**Strokes.** The stroke of a path is not one outline. It is many convex
pieces that all run the same way round: a four-sided piece for each line,
a fan for the outside of each corner, a cap for each end. Filled by the
non-zero rule, pieces that overlap add up, and nothing can cancel anything
else out. The usual construction, two offset curves joined into one
outline, has to get the inside of tight curves right, where the offset
doubles back over itself, and leaves holes when it does not. The test is
the definition of a stroke: with a round pen, a point is covered exactly
when it lies within half the pen's width of the path.

## The reference renderer

`crates/raster`, package `ow-raster`. It depends on geometry, colour and
paths. It fills paths into a picture in memory, on the CPU, exactly and
the same on every machine. Exports, thumbnails and tests draw with it,
and the renderer on the GPU will be measured against it.

**Coverage.** A `Mask` holds, for every pixel a path reaches, the exact
area of the path within that pixel. It does not sample points, so an edge
is as smooth as the pixels allow at any angle. Each edge adds its part to
the pixels it passes through, the parts of a row are summed from the left,
and the fill rule is applied to the sum. Font rasterizers work this way,
and the method has their known limit: where two edges of a shape that
overlaps itself cross inside one pixel, that pixel can be a little off.
A mask only holds the rows and columns its path reaches.

**Pictures.** A `Pixmap` is rows of red, green, blue and alpha bytes, the
colour multiplied by its alpha, which makes laying a colour over a pixel
one step for each byte. Colours are blended as the numbers they are stored
as and not as amounts of light. That is how browsers and the office
suites blend, and a half transparent shape has to look here as it looks
there. A picture is at most a quarter of a billion pixels; a larger one
has to be drawn in parts.

Not built: gradients, pictures as a fill, clipping, blend modes, text.

## Compression

`crates/flate`, package `ow-flate`. No dependencies. Every .docx, .xlsx
and .pptx is a ZIP archive of deflate streams, a PNG picture holds its
pixels in one and so does most of a PDF.

**Unpacking** handles all three kinds of block. Whoever calls it names
the most bytes it will take, because compressed data can unpack to a
thousand times its size and a file can be built to. It reads one bit at a
time, which is easy to check against the specification and slow.

**Packing** finds earlier data that comes back and writes it as a length
and a distance, in one block with the format's fixed codes. Any program
that reads deflate reads the result. It is not the smallest the format
allows: a block with codes fitted to its data is not built yet.

Both directions are tried against zlib by `tools/check_flate.py` on every
run of the checks. Readers for the formats above this one follow the same
rule: an interoperability check against a program that was not written
here.

## Planned parts

**The renderer** takes a scene, which is a list of shapes, images and runs
of text with transforms and clips, and draws it on the GPU. The same scene
can be written to a PDF or sent to a printer, so what is on screen and
what is on paper come from one description.

**The interface** is a tree of widgets that OfficeWorld lays out, draws
and animates itself. It is built for documents first: a page of text, a
grid of a million cells and a canvas are its normal case, not a special
widget.

**Text layout** turns characters, fonts and a width into lines and pages.
It is the largest single part of the engine, and the part where "almost
like Word" is not good enough: a line that breaks one word earlier moves
every page after it.

**The document model** stores a document as its history of operations.
The server of phase 6 cannot read documents, so it cannot merge them:
changes made apart are merged on the devices, and the model is built so
that every device arrives at the same result whatever order the changes
reach it in.

**The formats** are translators between the model and .docx, .xlsx, .pptx
and OpenDocument. What a translator does not understand, it keeps and
writes back unchanged where the format allows, so opening and saving a
file in OfficeWorld does not strip what it could not show.

## The workspace

One Cargo workspace, one crate per part in `crates/<part>`, package name
`ow-<part>`. The toolchain is pinned in `rust-toolchain.toml`. Every crate
inherits the workspace lints: clippy's pedantic set, no `unwrap` or
`expect` outside tests, every public item documented, and unsafe code
denied. The checks treat every warning as an error. A module that has to
use unsafe code, to talk to a platform or to the GPU, allows it for
itself and gets a decision record.

`tools/check.sh` runs the checks of CI's main job: formatting, clippy, the
tests, the interoperability checks, the documentation build and three
rules no compiler knows, each a small script beside it: no dashes,
dependencies on the allow list, and at most one line of test for two
lines of code. CI also runs the tests on
Windows and macOS, runs `cargo mutants` on what a push changed, and looks
for known vulnerabilities.
