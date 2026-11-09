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

- 12 tests and 3 checked examples pass on Linux, Windows and macOS.
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
- `cargo mutants` (27.1.0) makes 269 small changes to the motion crate.
  13 do not compile, the tests catch 250, and the 6 that no test can
  notice are listed with their reasons in `.cargo/mutants.toml`.
- Each rule check was seen refusing what it should: a file with an em
  dash or its HTML entity, a text file that is not UTF-8, a dependency
  that is not on the list, one from git, one from a path outside the
  repository, a `[patch]` table, tests kept outside a file's test module,
  and tests that had grown past half the code.
