# Security

## Supported versions

None yet. OfficeWorld has no releases. Fixes go to `main`.

## Reporting a vulnerability

Please do not report a security problem in a public issue or pull request.

Use GitHub's private vulnerability reporting: the "Report a vulnerability"
button on the repository's Security tab. Only the maintainer sees the
report.

If the button is not there, open an issue that says only that you have a
security report, with no details in it. The maintainer answers there and
arranges a private way to send them.

Say what you found, how to reproduce it and what it lets an attacker do.
You will get an answer within a week. Fixes are credited unless you ask
not to be named.

## What is in scope

Everything in this repository.

Today that is one library crate, `ow-motion`. It reads no files, opens no
connections, contains no unsafe code and depends on nothing outside this
workspace. The parts where security will matter most are not built yet:
the readers for file formats, which take input from strangers, and sync,
which promises that the server cannot read a document. Each gets its own
section here when it exists.
