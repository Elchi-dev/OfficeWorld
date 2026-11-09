# Changelog

## Unreleased

### Added

- The workspace, on a pinned Rust 1.99.0, with the lints every crate
  inherits.
- The motion crate, `ow-motion`:
  - `Spring`: a damped spring solved in closed form in all three damping
    regimes, built from its physical constants or from a response time
    and a damping ratio.
  - Rest detection by energy, so an animation knows when it may stop
    drawing.
  - `Animated`: a value a spring moves, whose target, spring and velocity
    can change mid-flight without a jump.
  - `CubicBezier` and `Tween`: easing curves in the form CSS and the
    office formats use, with the five named CSS curves, and moves that
    take a set time. A tween with a negative or endless duration is
    refused.
  - `Animatable` for `f32`, `f64` and arrays of them.
- The geometry crate, `ow-geometry`, in 64-bit coordinates with y
  downwards:
  - `Point` and `Vector` as two types, `Size`, `Insets`, and `Rect` with
    edges that belong to it on the left and the top only.
  - `RoundedRect`, which fits radii that are too large the way CSS does.
  - `Affine`, the matrix CSS, SVG and PDF write as
    `matrix(a, b, c, d, e, f)`.
  - Points, vectors, sizes and rectangles can be moved by a spring.
- The colour crate, `ow-color`:
  - `Color`: sRGB with alpha, hexadecimal notation read, one colour laid
    over another, luminance and contrast as the accessibility guidelines
    define them.
  - `Oklab` and its polar form for mixing and palettes, with colours no
    screen can show brought back by lowering their chroma.
- The path crate, `ow-path`:
  - `Path`: lines, quadratic and cubic curves and arcs of ellipses, built
    step by step with the rules of an HTML canvas.
  - SVG path data read, every command of it, and written. A text with a
    mistake is refused with the place of the mistake.
  - Rectangles, rounded rectangles, ellipses and polygons.
  - Flattening into straight lines within a tolerance.
  - Tight bounds, the exact area, and whether a point is inside under
    either fill rule.
  - Strokes with miter, round and bevel joins, butt, round and square
    ends and a miter limit.
- `tools/check.sh` with the three project rules: no dashes, dependencies
  on an allow list, at most one line of test for two lines of code.
- CI: every check on Linux, the tests on Windows and macOS, mutation
  testing of what a push changed, and a search for known vulnerabilities
  on every push and once a week.
- The licence, Business Source License 1.1, and the documents: README,
  ROADMAP, ARCHITECTURE, DECISIONS, CONTRIBUTING, SECURITY.

### Fixed

Found by a review after the first push, before any release.

- A time that is not a number, or infinite, no longer leaves an animated
  value stuck or not a number, and a clock that steps back no longer
  rewinds it.
- A spring whose constants are so far apart that the solution overflows
  is refused. It used to answer with a wrong number that looked right.
- A spring built from a response time that is too small or too large is
  told so, and no longer blamed for a stiffness nobody passed.
- A tween between two values whose difference overflows no longer starts
  as not a number.
- The documentation no longer says that a spring at a damping ratio of 1
  or more never passes its target. It can, once, when the value arrives
  with speed.
- The rule checks can no longer be passed by tests kept outside a file's
  test module, by a dependency from git or from a path outside the
  repository, by a manifest in an unusual place, by a dash written as an
  HTML entity or by a text file that is not UTF-8.

### Verified

- 68 tests and 7 checked examples pass on Linux, Windows and macOS.
- The spring solution is put back into its differential equation, with
  its derivatives measured across a short step, for 8 springs across all
  three regimes, 3 starting states and 3 times each. Two of the springs
  sit a billionth beside critical damping, where the formulas divide by
  a frequency that is almost zero.
- A spring damped 500 times past critical still gives a number after 10
  seconds, where the textbook form of the solution gives infinity times
  zero.
- The easing solver is compared with the curve as it is constructed, at
  999 points on each of 6 curves, one of which stands still in time at
  its middle. The four curved CSS curves are compared with values worked
  out in exact fractions from the control points CSS gives them. On the
  CSS `ease` curve the solver needs at most 6 guesses.
- Restarting an animation's stored state 144 times a second, with a clock
  that also steps back, leaves its path where it was, to nine decimal
  places.
- Two rectangles side by side are asked about a point on the edge they
  share, and exactly one says yes. A rounded rectangle is asked about
  points in each of its cut-off corners. A transform followed by its
  inverse puts points back where they were, and a transform that cannot
  be undone has no inverse.
- The Oklab conversion is compared with the values the CSS Color
  specification gives for the corners of sRGB, which pins all eighteen
  of its constants. 256 colours spread through the cube come back from
  Oklab as the bytes they went in as.
- Flattened curves are measured against the curve at 4,001 points each,
  for 4 curves at 3 tolerances, and the number of lines is compared with
  what Wang's formula gives when worked out apart.
- Bounds are compared with a thousand points along each curve. The exact
  area of a shape made of curves is compared with the same shape cut
  into lines a thousand to the curve.
- An arc is checked against the ellipse it was asked to follow for all
  four combinations of its two choices: every curve of it keeps within
  three ten-thousandths of the radius, and half way along it is at the
  angle half way.
- A rounded rectangle's outline and its radii are asked the same
  question, whether a point is inside, at 28,341 points, and agree
  everywhere but within a hundredth of the edge.
- A stroke with a round pen is compared with its definition at 4,941
  points: covered exactly when within half the pen's width of the path.
  The path has corners turning each way, a line shorter than the pen is
  wide, a curve tighter than the pen, a closed shape smaller than the
  pen, a line that turns straight back and a dot.
- `cargo mutants` (27.1.0) makes 1,429 small changes to the four crates.
  176 do not compile, the tests catch 1,241, and the 12 that no test can
  notice are listed with their reasons in `.cargo/mutants.toml`.
- Each rule check was seen refusing what it should: a file with an em
  dash or its HTML entity, a text file that is not UTF-8, a dependency
  that is not on the list, one from git, one from a path outside the
  repository, a `[patch]` table, tests kept outside a file's test module,
  and tests that had grown past half the code.
