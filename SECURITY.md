# Security

## Supported versions

None yet. OfficeWorld has no releases. Fixes go to `main`.

## Reporting a vulnerability

Please do not report a security problem in a public issue or pull request.

Use GitHub's private vulnerability reporting: the "Report a vulnerability"
button on the repository's Security tab. Only the maintainer sees the
report.

Say what you found, how to reproduce it and what it lets an attacker do.
You will get an answer within a week. Fixes are credited unless you ask
not to be named.

## What is in scope

Everything in this repository.

Today that is a set of library crates. None of them opens a connection,
contains unsafe code or depends on anything outside this workspace.

Two of them take input that can come from a stranger. `ow-path` reads SVG
path data, and `ow-flate` unpacks compressed data. Both are written to
stop at the first thing they cannot read, to do an amount of work bounded
by the length of what they were given, and to hold no more memory than
the caller allowed. A way to make either of them crash, hang or use more
than that is a security problem and worth a report.

The parts where security will matter most are not built yet: the readers
for whole file formats, and sync, which promises that the server cannot
read a document. Each gets its own section here when it exists.
