# Contributing

Thanks for looking. Most of the rules below are enforced by tools, so a
change learns quickly whether it fits.

## Before you write code

OfficeWorld is also licensed commercially, so a contribution needs an
agreement that lets the licensor do that with it. The agreement is not
written yet. Until it is, code from outside cannot be merged.

Bug reports, questions and ideas are welcome as issues. If you want to
contribute code, open an issue first and say what you have in mind.

## Setting up

You need:

- rustup, which installs the pinned Rust from `rust-toolchain.toml` by
  itself
- Python 3.11 or newer, for the checks

## Before every commit

```sh
./tools/check.sh
```

It runs the checks of CI's main job, in the same order, and stops at the
first one that fails: formatting, clippy, the tests, the documentation
build and the three project rules. CI's Linux job runs the same script.
CI also runs the tests on Windows and macOS, mutation-tests what a push
changed and looks for known vulnerabilities.

## The rules

- **Tests are few and needed.** At most one line of test for two lines of
  code, across the workspace. A test checks behaviour against its
  definition, is named after what it protects, and says in a comment what
  goes wrong without it. [DECISIONS](docs/DECISIONS.md) has the reasons.
- **Every change to the code has to be noticed.** `cargo mutants` (from
  `cargo install cargo-mutants`) breaks the code in small ways and runs
  the tests on each. It has to end without a missed change. A change no
  test can notice goes into `.cargo/mutants.toml` with the reason.
- **Unit tests live at the end of the file**, in one `#[cfg(test)]`
  module. The line count relies on it.
- **Few dependencies.** A crate from outside needs a line in
  `tools/allowed-deps.txt` with its reason, and a record in
  [DECISIONS](docs/DECISIONS.md). Small, well specified pieces are written
  here.
- **No unsafe code**, unless a module allows it for itself and a decision
  record says why. Every unsafe block then gets a `// SAFETY:` comment.
- **No dashes.** No em dashes and no en dashes anywhere, in code, docs or
  commit messages. Use a comma, a colon or a full stop, and a hyphen in a
  range.
- **Comments say why.** The code says what. A comment that tells the
  story of a bug is worth keeping.
- **Errors say what to do.** A message a person sees is a full sentence.
- **Refuse wrong input out loud.** A constructor returns an error for a
  value that makes no sense. It does not quietly repair it.
- **Decisions are written down.** A change to how OfficeWorld is built or
  behaves gets a record in [DECISIONS](docs/DECISIONS.md), with what was
  rejected.

## Commits

- Conventional prefixes, with a scope where it helps:
  `feat(motion): ...`, `fix(sheets): ...`, `docs: ...`, `build: ...`,
  `ci: ...`, `test: ...`, `refactor: ...`.
- Imperative, a subject under about 70 characters, and a body that says
  why when it is not obvious.
- One logical step per commit, each one building and passing the checks.

## Security

See [SECURITY](SECURITY.md). Please do not open a public issue for a
vulnerability.
