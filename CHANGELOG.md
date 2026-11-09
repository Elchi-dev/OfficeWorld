# Changelog

## Unreleased

### Added

- The workspace, on a pinned Rust 1.99.0, with the lints every crate
  inherits.
- The motion crate, `ow-motion`:
  - `Spring`: a damped spring solved in closed form for springs that
    overshoot, arrive exactly and creep in, built from its physical
    constants or from a response time and a damping ratio.
  - Rest detection by energy, so an animation knows when it may stop
    drawing.
  - `Animated`: a value a spring moves, whose target, spring and velocity
    can change mid-flight without a jump.
  - `CubicBezier` and `Tween`: easing curves in the form CSS and the
    office formats use, with the five named CSS curves, and moves that
    take a set time.
  - `Animatable` for `f32`, `f64` and arrays of them.
- `tools/check.sh` with the three project rules: no dashes, dependencies
  on an allow list, at most one line of test for two lines of code.
- CI: every check on Linux, the tests on Windows and macOS, and a search
  for known vulnerabilities on every push and once a week.
- The licence, Business Source License 1.1, and the documents: README,
  ROADMAP, ARCHITECTURE, DECISIONS, CONTRIBUTING, SECURITY.

### Verified

- 11 tests and 3 checked examples pass on Linux, Windows and macOS.
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
  out in exact fractions from the control points CSS gives them.
- Restarting an animation's stored state 144 times a second leaves its
  path where it was, to nine decimal places.
- 75 faults were put into the code on purpose, one at a time. The tests
  caught 74. The one they let through replaces Newton's method in the
  easing solver with bisection, which is slower and changes no result.
- Each rule check was seen refusing what it should: a file with an em
  dash, a manifest with a dependency that is not on the list, and tests
  that had grown past half the code.
