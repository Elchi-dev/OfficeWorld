# Roadmap

Seven phases. Each one makes the next one possible, and each ends with
something a person can try. There are no dates: a phase is finished when
its bar is met.

## Phase 1: feel (in progress)

The design system, motion and the interface layer.

Why first: an office suite is open for hours a day, so how it feels is the
product. Everything later is drawn with what this phase builds, and a slow
or stiff foundation cannot be fixed from above.

Done:

- **Motion.** Springs solved in closed form, easing curves, tweens, and
  values that can be given a new target mid-flight.
- **Geometry and colour.** Points, rectangles, rounded rectangles and
  transforms. sRGB as files and screens speak it, and Oklab to mix in.
- **Paths.** Lines, curves and arcs, built step by step or read from SVG
  path data. Flattening, tight bounds, exact areas, what is inside, and
  strokes with their joins and ends.

- **A reference renderer.** Paths filled and stroked on the CPU into
  pictures in memory, with the exact area of each pixel that a shape
  covers. The renderer on the GPU will be measured against it.
- **Compression.** Deflate and zlib in both directions, needed to write a
  picture to a file now and to open office files later.

Open:

- **Pictures as files.** PNG first, so that what is drawn can be looked
  at and compared.
- **Gradients, pictures and text** in the reference renderer.
- **The design system.** Colour, type, spacing and the motion vocabulary
  as tokens every app shares.
- **The renderer.** Shapes, images and text on the GPU, in a window on
  Windows, macOS and Linux and in a browser.
- **Layout, widgets and input.** Pointer, keyboard, text input with input
  methods, scrolling with momentum.
- **Accessibility.** What is on screen is described to screen readers from
  the first widget on.
- **The shell.** Windows, tabs, the command palette, settings.
- **Budgets in CI.** Frame time and input latency are measured on every
  push, and a change that breaks a budget fails the build.

The bar:

- A frame takes at most 8.3 ms, so a 120 Hz screen never drops one.
- A key press is on screen in the next frame.
- From starting the app to a window that takes input: at most 300 ms.
- Nothing is drawn while nothing moves.

These are targets the phase is built against. None of them is measured
yet, and the machine they are measured on is fixed when the renderer
exists.

Then a person decides. Speed can be measured, and whether something feels
good cannot. Before phase 2 builds on this one, someone uses a build of
the shell and says whether it does.

## Phase 2: the shared engine

- **Text layout.** Fonts, shaping, text in both directions, line breaking,
  hyphenation, paragraphs and pages, with the same result on every system.
- **The canvas.** Shapes, images, groups, transforms and hit testing.
- **The document model.** One model for text, cells, shapes and pages.
  Every change is an operation, which gives history, undo and the merging
  of changes that were made apart.
- **The formats.** .docx, .xlsx, .pptx and OpenDocument, read and written
  by code in this repository.
- **The fidelity measurement.** A corpus of real documents. Each one is
  opened, drawn and compared with how its own application draws it, and
  the score per format is published.

Why here: every editor is this engine with a different toolbar. Building
an editor first would mean building the engine inside it, and again inside
the next one.

## Phase 3: Docs and Sheets

- **Docs.** Styles, tables, pictures, headers and footers, footnotes,
  tracked changes, comments, PDF export.
- **Sheets.** The formula engine with the functions Excel files use,
  recalculation of large sheets, formatting, charts, pivot tables.

Why together: they share almost no logic above the engine, so they are
built side by side. They are also what most people open first, and what
decides whether someone can switch.

## Phase 4: Slides, Design and Board

Presentations with a presenter view, layouts for print and screen with
templates and brand kits, and whiteboards with diagrams.

Why here: all three are the canvas of phase 2 with different tools on top,
and Slides needs the text and the tables phase 3 finished.

## Phase 5: PDF

Read, annotate, fill in, sign, edit and merge.

Why here: writing a PDF comes with phase 3. Reading and editing one is a
format of its own and stands on the finished canvas.

## Phase 6: Sync, Drive and working together

- **The server.** One program you can host yourself. It stores what it is
  sent and cannot read it: documents are encrypted on the devices.
- **Drive.** Storage, sync, sharing and version history.
- **Working together.** Several people in one document at the same time,
  and changes made offline merged when a device is back.
- **Forms.** Surveys whose answers arrive in a sheet.

Why here: the model has merged changes since phase 2, so this phase adds
transport and storage, not a second way of editing.

## Phase 7: Mail, Calendar and Contacts

Clients for the standard protocols: IMAP and SMTP for mail, CalDAV for
calendars, CardDAV for contacts. They work with any provider that speaks
them and with your own server.

Why last: they share the interface and little else, and the editors are
the reason to install OfficeWorld.

## Not planned

- **Hosting mail.** Delivering mail reliably is an operations business,
  with reputation, abuse handling and deliverability. A program cannot
  ship that.
- **Chat and video calls.** Good tools exist, and they are not what makes
  an office suite.
- **A photo library.**
- **A template library the size of Canva's.** It is content made by
  designers and licensed from them. OfficeWorld ships the editor and a
  starter set.
- **Phones and tablets**, until the desktop and the browser are good.
