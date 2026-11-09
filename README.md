<div align="center">

```
 ██████╗ ███████╗███████╗██╗ ██████╗███████╗██╗    ██╗ ██████╗ ██████╗ ██╗     ██████╗
██╔═══██╗██╔════╝██╔════╝██║██╔════╝██╔════╝██║    ██║██╔═══██╗██╔══██╗██║     ██╔══██╗
██║   ██║█████╗  █████╗  ██║██║     █████╗  ██║ █╗ ██║██║   ██║██████╔╝██║     ██║  ██║
██║   ██║██╔══╝  ██╔══╝  ██║██║     ██╔══╝  ██║███╗██║██║   ██║██╔══██╗██║     ██║  ██║
╚██████╔╝██║     ██║     ██║╚██████╗███████╗╚███╔███╔╝╚██████╔╝██║  ██║███████╗██████╔╝
 ╚═════╝ ╚═╝     ╚═╝     ╚═╝ ╚═════╝╚══════╝ ╚══╝╚══╝  ╚═════╝ ╚═╝  ╚═╝╚══════╝╚═════╝
```

**One fast app for documents, sheets, slides and design. Your files stay on your disk.**

[![CI](https://github.com/Elchi-dev/OfficeWorld/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/Elchi-dev/OfficeWorld/actions/workflows/ci.yml)
![Status](https://img.shields.io/badge/status-phase%201%3A%20feel-orange)
[![Rust](https://img.shields.io/badge/Rust-1.99.0-B7410E?logo=rust)](rust-toolchain.toml)
[![License: BSL 1.1](https://img.shields.io/badge/license-BSL%201.1-2F7CF6)](LICENSE)
[![Made by Elchi](https://img.shields.io/badge/made%20by-Elchi-8A2BE2)](https://github.com/Elchi-dev)

</div>

---

## Overview

> **OfficeWorld is at the very beginning, and most of this page describes
> a plan.** What exists today is the bottom of the stack: how things move,
> the geometry, colour and paths that drawing is made of, a renderer that
> fills them into a picture in memory, and compression. There is no editor,
> no window and nothing to open a file with yet.
> [ROADMAP](docs/ROADMAP.md) says what is finished.

Every office suite asks you to accept a trade.

Microsoft 365 is where most of the world's documents come from. It is a
subscription, built around Microsoft's cloud. Google Workspace makes
sharing easy, and your documents live on Google's servers. LibreOffice is
free and keeps your files on your disk, on a code base that goes back to
the 1980s. Canva made design easy for everyone, as an online service
behind an account.

In the three classic suites a document also belongs to one program: the
word processor, the spreadsheet or the presentation program. A chart from
the spreadsheet can be linked into the report. It remains a link to a
second file that has to be refreshed, and to change it you go back to the
other program.

OfficeWorld is being built as one app. The goals:

- **Everything is one document.** Docs, Sheets, Slides, Design, Board and
  PDF are views of the same engine. What you make in one is live in the
  others.
- **Fast enough to feel instant.** Written in Rust and drawn on the GPU,
  native on Windows, macOS and Linux, and the same code in a browser.
  Every animation can be interrupted, and none of it depends on your
  screen's refresh rate.
- **Local first.** Your files are files on your disk. There is no account
  and nothing needs a cloud. Sync and working together in real time are
  optional, end-to-end encrypted, and run on a server you can host
  yourself.
- **The files you already have.** Word, Excel and PowerPoint files have to
  open and save faithfully. How faithfully will be measured against real
  documents and published as a number.
- **The rest of the desk.** Files, mail, calendar and contacts beside the
  editors, speaking the standard protocols, so they work with the provider
  you already have or with your own server.

---

## The trick

**There is one document model, and every app is a view of it.**

```
   Docs     Sheets    Slides    Design    Board      PDF
     |         |         |         |         |         |
  +--+---------+---------+---------+---------+---------+--+
  |  one document model                                   |
  |  text, cells, shapes, pages, comments, history        |
  +---------------------------+---------------------------+
                              |
  +---------------------------+---------------------------+
  |  one engine                                           |
  |  text layout, formulas, drawing, motion               |
  +---------------------------+---------------------------+
                              |
        GPU on Windows, macOS and Linux, WebAssembly
        in a browser: the same code
```

Everything else follows from that:

- **Live across apps.** A range of a sheet placed in a text document or on
  a slide is that range, not a copy of it. Change the number and the
  report is right.
- **The same result everywhere.** One implementation of fonts, text
  layout, tables and colour. A table looks the same in a document, on a
  slide, in a PDF and on every operating system, because nothing is left
  to the platform.
- **Learned once.** The same commands, shortcuts, comments and history in
  every app.
- **Working together, built in.** History, undo and merging changes from
  several people belong to the model, so every app has them, online and
  offline.
- **Office files are translations.** A .docx, .xlsx or .pptx is translated
  into the model and back. A translation can be measured, so its quality
  is a number and not a claim.

---

## The parts

| Part | What it is | State |
| --- | --- | --- |
| [`crates/motion`](crates/motion) | springs solved in closed form, easing curves, tweens, values that animate and can change target mid-flight | built |
| [`crates/geometry`](crates/geometry) | points, vectors, rectangles, rounded rectangles and affine transforms, in 64-bit coordinates | built |
| [`crates/color`](crates/color) | sRGB with hex notation, layering and contrast, and Oklab for mixing, palettes and gamut mapping | built |
| [`crates/path`](crates/path) | lines, curves and arcs, SVG path data read and written, flattening, bounds, areas, hit testing and strokes | built |
| [`crates/raster`](crates/raster) | the reference renderer on the CPU: the exact area of a shape in every pixel, pictures in memory, fill and stroke | built |
| [`crates/flate`](crates/flate) | deflate and zlib, packed and unpacked, with the CRC-32 and Adler-32 checksums: what ZIP, PNG and PDF are made of | built |
| Design system | colour, type, spacing and the motion vocabulary every app shares | phase&nbsp;1 |
| Interface | the GPU renderer, layout, widgets, input and accessibility | phase&nbsp;1 |
| Engine | text layout, the canvas, the document model with history and merging | phase&nbsp;2 |
| Formats | .docx, .xlsx, .pptx and OpenDocument, with the fidelity measurement | phase&nbsp;2 |
| Docs, Sheets | the word processor and the spreadsheet | phase&nbsp;3 |
| Slides, Design, Board | presentations, layouts for print and screen, whiteboards and diagrams | phase&nbsp;4 |
| PDF | read, annotate, fill in, sign, edit, merge | phase&nbsp;5 |
| Sync, Drive, Forms | the server you can host yourself, shared files, working together in real time, surveys that feed a sheet | phase&nbsp;6 |
| Mail, Calendar, Contacts | clients for the standard protocols | phase&nbsp;7 |

---

## Quick look

You need rustup, which installs the pinned Rust 1.99.0 from
`rust-toolchain.toml` by itself, and Python 3.11 or newer for the checks.

```sh
git clone https://github.com/Elchi-dev/OfficeWorld
cd OfficeWorld
./tools/check.sh          # every check of CI's main job, in the same order
python3 tools/stats.py    # lines of code and of test per crate
```

What there is to look at today is code. How a panel moves:

```rust
use ow_motion::{Animated, Spring};

let mut width = Animated::new(240.0, Spring::from_response(0.3, 1.0)?);
width.set_target(320.0, 10.0);

// Half way open, the panel is told to close again. It turns around from
// where it is, at the speed it has: no jump, no stall.
width.set_target(240.0, 10.1);

// Where it is at any time, on any screen, however many frames were drawn.
let now = width.value(10.15);
```

And what an icon is made of:

```rust
use ow_path::{Cap, FillRule, Path, Stroke};

// A tick, as SVG path data, drawn with a round pen two wide.
let tick: Path = "M4 12 L10 18 L20 6".parse()?;
let pen = Stroke::new(2.0).with_cap(Cap::Round);
let shape = tick.stroke(pen, 0.05);

// The shape is what gets filled, and what a click is tested against.
assert!(shape.contains((10.0, 18.5), FillRule::NonZero, 0.05));
```

---

## Planned

In the order it gets built. The reasons for the order are in
[ROADMAP](docs/ROADMAP.md).

1. **Feel first.** The design system, motion and the interface layer, with
   budgets for frame time and input latency that fail the build when they
   are broken. A person judges this phase before anything is built on it.
2. **The shared engine.** Text layout, the canvas, the document model, the
   file formats and the fidelity measurement.
3. **Docs and Sheets**, side by side.
4. **Slides, Design and Board** on the shared canvas.
5. **PDF.**
6. **Sync, Drive and working together**, end-to-end encrypted, with Forms.
7. **Mail, Calendar and Contacts.**

---

## Non-goals

- **A web page in a window.** No Electron and no web view. The interface
  is drawn by OfficeWorld itself.
- **Hosting your mail.** Delivering mail reliably is an operations
  business, not a program. OfficeWorld's mail is a client for the provider
  or the server you choose.
- **Chat, video calls and a photo library.** Other tools do these well.
- **A template library the size of Canva's.** That is content made by
  designers. OfficeWorld ships the editor and a starter set.
- **Phones and tablets, for now.** Desktop and browser come first.

---

## Status

**Phase 1 has begun.** Six crates are built: motion, geometry, colour,
paths, the reference renderer and compression. Nothing of it can be seen
on a screen yet.

Next to the suites named at the top, OfficeWorld is better at nothing
today: they exist. What can be judged already is how it is built:

- **Few tests, each one needed.** At most one line of test for two lines
  of code, counted without comments and blank lines.
  `tools/stats.py --check` fails the build otherwise, so a test has to
  protect behaviour that matters.
- **Every change to the code has to be noticed.** `cargo mutants` changes
  the code in small ways, one at a time: a flipped comparison, a function
  that returns a constant. It has to end without a change the tests let
  through. The few that no test can notice are listed, each with its
  reason, in [`.cargo/mutants.toml`](.cargo/mutants.toml). CI repeats this
  for everything a push touches.
- **Checked against the definition.** The spring solution is put back into
  the differential equation it claims to solve. The easing solver is
  compared with the curve as it is drawn. The colour conversions are
  compared with the numbers the CSS specification gives. A stroke with a
  round pen has to cover exactly the points within half its width of the
  path, measured point by point. What the renderer says a shape covers of
  a pixel is compared with the shape cut into triangles and each triangle
  cut to that pixel.
- **Checked against other programs.** Compression is tried against zlib
  in both directions on every run of the checks: what is packed here zlib
  unpacks, and what zlib packs, in every way it has, is unpacked here.
- **Three systems.** The tests run on Linux, Windows and macOS on every
  push.

`python3 tools/stats.py` prints the lines of code and of test per crate,
and [CHANGELOG](CHANGELOG.md) says what was verified and how.

---

## Docs

| Document | What |
| --- | --- |
| [ROADMAP](docs/ROADMAP.md) | the phases, what is in and out, and why in that order |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | how the parts fit together |
| [DECISIONS](docs/DECISIONS.md) | why things are the way they are, with what was rejected |
| [MODEL](docs/MODEL.md) | the design of the one document model, before it is built |
| [CONTRIBUTING](CONTRIBUTING.md) | building, checking and the rules |
| [SECURITY](SECURITY.md) | reporting a vulnerability |
| [CHANGELOG](CHANGELOG.md) | what changed |

---

## Licence

OfficeWorld is source-available under the
[Business Source License 1.1](LICENSE). In short:

- You may read the source, change it and pass it on.
- You may use it for real work, at home and in a company, free of charge,
  as long as you do not sell it. Selling means charging for OfficeWorld
  itself, for a changed version, for access to it as a hosted service, or
  for a product or service whose value comes entirely or substantially
  from it. Whoever does that needs a commercial licence, from
  [legal@elchi.dev](mailto:legal@elchi.dev).
- Four years after a version is published, that version becomes
  Apache 2.0.

This is a summary and not the licence. The licence text decides.

**Made by Elchi**
