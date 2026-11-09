# Architecture

How OfficeWorld is put together. One part is built: the motion crate. The
rest of this page is the plan the code is written towards. It changes when
building a part teaches something, and each part gets its own section here
once it exists.

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
Only motion is built today.

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

`c` and `s` are cos and sin for a spring that overshoots, cosh and sinh
for one that creeps in, and 1 and `t` for the one between them. For the
creeping spring the two exponentials are multiplied out, because cosh
alone grows past what a 64-bit float holds while the product is still an
ordinary number.

**Rest** is decided by energy. `Spring::is_at_rest` asks whether the
energy left could still carry the value further than a tolerance from its
target. A damped spring only loses energy, so the answer never flips back
to no, and that is the moment an animation stops asking for frames.

**`Animated<T>`** is a value with a target. A new target, a new spring or
a velocity handed over by a gesture takes effect from where the value is
at that moment, at the speed it has. Restarting the stored state does not
bend the path, so the motion does not depend on which frames were drawn.

**`CubicBezier` and `Tween<T>`** are for motion whose length is given: a
fade, or an animation that a presentation file describes by its duration.
The curves are the cubic Bézier form CSS and the office formats use. The
solver is Newton's method inside a bracket, so a curve that stands still
in time cannot throw it off. A tween returns exactly the value it was sent
to at its end.

**`Animatable`** is what a value needs for all of this: adding, scaling
and a length. It is implemented for `f32`, `f64` and arrays of them, and
the geometry and colour crates will implement it for their types.

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
inherits the workspace lints: clippy's pedantic set as errors, no
`unwrap` or `expect` outside tests, every public item documented, and
unsafe code forbidden until a crate needs it and says why.

`tools/check.sh` runs every check CI runs. The three rules no compiler
knows are small scripts beside it: no dashes, dependencies on the allow
list, and at most one line of test for two lines of code.
