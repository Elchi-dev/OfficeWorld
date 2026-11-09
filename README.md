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

Every office suite asks you to accept a trade.

Microsoft 365 opens every file there is. It is a subscription, built
around Microsoft's cloud. Google Workspace makes sharing easy, and it
lives on Google's servers. LibreOffice is free and keeps your files on
your disk, and its interface shows its age. Canva is a pleasure to use,
and it needs an account and a connection.

And each of them is a set of separate programs. A chart copied from the
spreadsheet into the report is a picture of a chart. When the numbers
change, the report is wrong and nobody notices.

OfficeWorld is one app:

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
- **The files you already have.** Word, Excel and PowerPoint files open
  and save faithfully. How faithfully is measured against real documents
  and published, not promised.
- **The rest of the desk.** Files, mail, calendar and contacts sit beside
  the editors and speak the standard protocols, so they work with the
  provider you already have or with your own server.

> OfficeWorld is at the very beginning. What exists today is the
> foundation of how it moves: springs solved in closed form, easing curves
> and values that animate. **There is no editor, no window and nothing to
> open a file with yet.** Everything else on this page describes where it
> is going. [ROADMAP](docs/ROADMAP.md) says what is finished.

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
./tools/check.sh          # everything CI runs, in the same order
python3 tools/stats.py    # lines of code and of test per crate
```

What there is to look at today is the motion crate:

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

**Phase 1 has begun.** The motion crate is built. Nothing else is.

Next to the suites named at the top, OfficeWorld is better at nothing
today: they exist. What can be judged already is how it is built:

- **Few tests, each one needed.** At most one line of test for two lines
  of code, counted without comments and blank lines.
  `tools/stats.py --check` fails the build otherwise, so a test has to
  protect behaviour that matters.
- **Checked against the definition.** The spring solution is put back into
  the differential equation it claims to solve. The easing solver is
  compared with the curve as it is drawn. The CSS curves are compared with
  values worked out in exact fractions.
- **Every test was seen failing.** 75 faults were put into the code on
  purpose, one at a time: a flipped sign, a dropped term, a loosened
  check. The tests caught 74. The one they let through makes the easing
  solver slower and changes no result.
- **Three systems.** The tests run on Linux, Windows and macOS on every
  push.

| | |
| --- | --- |
| Rust, code | 345 lines in 1 crate |
| Rust, tests | 164 lines, 11 tests and 3 checked examples |
| Dependencies | 0 |

`python3 tools/stats.py` prints the current numbers.

---

## Docs

| Document | What |
| --- | --- |
| [ROADMAP](docs/ROADMAP.md) | the phases, what is in and out, and why in that order |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | how the parts fit together |
| [DECISIONS](docs/DECISIONS.md) | why things are the way they are, with what was rejected |
| [CONTRIBUTING](CONTRIBUTING.md) | building, checking and the rules |
| [SECURITY](SECURITY.md) | reporting a vulnerability |
| [CHANGELOG](CHANGELOG.md) | what changed |

---

## Licence

OfficeWorld is source-available under the
[Business Source License 1.1](LICENSE). In short:

- You may use it, at home and at work, free of charge. You may read the
  source, change it and pass it on.
- You may not sell it: not the app, not a changed version, not access to
  it as a hosted service, not a product whose value is mostly OfficeWorld.
  That takes a commercial licence, from
  [legal@elchi.dev](mailto:legal@elchi.dev).
- Four years after a version is published, that version becomes
  Apache 2.0.

The licence text decides, not this summary.

**Made by Elchi**
