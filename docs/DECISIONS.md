# Decisions

Why OfficeWorld is built the way it is, with what was rejected. Newest
last.

## One app and one document model

Docs, Sheets, Slides, Design, Board and PDF are views of the same model,
drawn by the same engine.

In the classic suites a document belongs to one program, and content
from another program sits in it as a linked or embedded object of that
other program. One model makes content live across apps, gives every app
the same history, comments and merging, and means text layout, tables and
colour are written once.

Rejected: one program per kind of document, sharing libraries. It is how
the classic suites are built, and it is why a chart in a report is a link
to be refreshed and not the numbers themselves.

The cost: the model has to be good enough for a novel, a sheet with a
million cells and a poster at once. It is the hardest design in the
project, and it comes in phase 2, before any editor.

## Rust for every part

The app, the browser build and later the server are Rust.

The same model has to run in the desktop app, in WebAssembly in a browser
and next to the server. A second implementation in another language would
be a second place for the two to disagree about what a document is.

## An interface OfficeWorld draws itself

The interface is drawn on the GPU by code in this repository. There is no
web view and no Electron.

Rejected: a web view with the interface in web technology. It is the
fastest way to a window, and the wrong base for this product. A document
has to break its lines and pages the same on every system, and a web
view's text layout differs by platform and version. The page of a word
processor would be drawn by hand inside the web view anyway. And frame
timing would belong to the browser engine, when feel is the first goal.

Rejected: an existing toolkit. A toolkit decides how text is laid out,
how frames are scheduled and how things move, and those three are what
OfficeWorld is about.

The cost is large and known. Text input with input methods,
accessibility, menus, drag and drop and everything else a toolkit gives
away for nothing have to be built, per platform. Phase 1 exists to pay
that cost before anything depends on it.

## Motion is solved, not stepped

A spring's position is computed directly for the time asked for.

Rejected: advancing a simulation by one step per frame. It is simpler to
write, and its result depends on the frame rate: a long frame overshoots,
a fast screen and a slow one drift apart, and a test has to replay every
frame to learn where something ends up. The closed form has none of that
and costs a handful of floating point operations.

Following from it, nothing in the crate reads a clock. The caller names
the time.

## Few dependencies, on an allow list

A piece that is small and well specified is written here, with tests
against its specification: the zip container, XML, the office formats
themselves, layout, the formula engine.

A dependency is for something large and subtle that a maintained library
does better: access to the GPU, windows and their events, font shaping,
image codecs, TLS and cryptography, the bridge to each platform's
accessibility interface. Every crate a manifest names is listed with its
reason in `tools/allowed-deps.txt`, and CI refuses any other, as it
refuses one taken from git or from a path outside the repository. What an
allowed crate depends on in turn comes with the decision to allow it.
Which library fills each of those places is decided when the part that
needs it is built.

Today the list is empty.

## At most one line of test for two lines of code

`tools/stats.py --check` fails the build when the tests grow past half
the code.

A large test suite is easy to produce and expensive to own. Tests that
restate the code, cover a getter or repeat another test make every change
slower and protect nothing. The limit forces the question of what a test
is for.

What carries the weight instead:

- A test checks against a definition, not against a copy of the formula:
  the differential equation, the construction of the curve, a value
  worked out apart from the code.
- A test is named after what it protects and says in a comment what goes
  wrong without it.
- A test has to notice when the code is wrong. `cargo mutants` breaks the
  code on purpose, one small change at a time, and has to end without a
  change the tests let through. A change that no test can notice is listed
  with its reason in `.cargo/mutants.toml`. CI runs this on what every
  push touched.
- Where a format has thousands of real files, the files are the test: a
  small harness over a large corpus. The corpus is data and does not count
  as test code.
- Examples in doc comments run as tests. They are documentation first and
  count on neither side.

Rejected: a coverage target. It rewards exactly the tests this rule is
meant to keep out.

## Local first, and merging on the devices

A document is a file on the disk of whoever works on it. Sync is optional
and end-to-end encrypted.

That has a consequence for the model. A server that cannot read documents
cannot merge them, so changes made apart are merged on the devices, and
the model has to guarantee that every device reaches the same result in
whatever order the changes arrive. Server-side merging, which is how the
large online suites work, is ruled out by the encryption.

## The licence: Business Source License 1.1

Anyone may use OfficeWorld, read its source, change it and pass it on.
Selling it takes a commercial licence. Four years after a version is
published it becomes Apache 2.0.

- **Licensor:** Elchi.
- **Additional use grant:** production use is allowed as long as the
  licensee does not sell OfficeWorld: not the work, not a changed version,
  not access to it as a service, not a product or service whose value
  comes entirely or substantially from it. Charging for documents made
  with it is not selling it.
- **Change date:** four years after a version is published, the longest
  the licence allows.
- **Change licence:** Apache 2.0.

Rejected: MIT or Apache 2.0 from the start. Anyone could sell it the day
it is usable. Rejected: AGPL. A host could still sell it as a service by
publishing its changes. Rejected: a licence that only lets people look at
the source. Nobody could use the product.

What the licence does and does not do. Its own terms always allow
passing the source on, with the licence attached, and they say nothing
about a price. What it withholds from someone who sells OfficeWorld is
the right to use it in production. So it stops a company from running
OfficeWorld as a paid service. It does not by its wording stop someone
from selling copies without using them.

Still open:

- Whether the grant should also withhold production use from whoever
  bought a copy from such a seller, which would close that gap.
- Who the licensor is as a legal person. The file names a handle.
- What counts as a version before there is a release.
- Where "substantially" begins: paid support, training or hosting of a
  company's own installation are not clearly on either side.

Contributions from outside need an agreement that lets the licensor grant
commercial licences for them. That agreement is not written yet, and
until it is, no outside code is merged.

## A pinned toolchain and strict lints

`rust-toolchain.toml` pins the compiler, so CI and every machine build
with the same one. The workspace switches on clippy's pedantic set,
refuses `unwrap` and `expect` outside tests, wants every public item
documented and denies unsafe code, and the checks treat every warning as
an error. A module that needs unsafe code, such as the one that talks to
the platform, allows it for itself and gets a record here. The lint is
denied and not forbidden because a forbidden lint cannot be allowed again
anywhere below, which would leave such a module no way in.
