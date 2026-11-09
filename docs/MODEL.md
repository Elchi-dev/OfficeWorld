# The document model

This page is a design. None of it is built. It is written down before the
code because everything in phases 2 to 6 is built on it, and a mistake
here is paid for in every editor and every file format. It changes when
building a part teaches something.

## What it has to do

- Hold a novel, a sheet with a million cells and a poster in one kind of
  document, so that a range of a sheet inside a page of text is that
  range and not a copy.
- Let two people change the same document apart, on a train and at a
  desk, and bring the changes together later on their own devices. The
  server of phase 6 only passes on what it cannot read, so it cannot do
  the merging.
- Give every editor history, undo and "who changed this" without writing
  them again.
- Open in an instant and stay fast: typing into a long document and
  pasting a hundred thousand cells must not be felt.

## The idea

A document is a set of **objects**. An object is one of a few kinds of
container, and each kind knows how to merge changes that were made apart.
Every change to a document is an **operation** on one object, and the
document is the result of all its operations. Two devices that have seen
the same operations hold the same document, in whatever order the
operations reached them.

Nothing else in the model merges. A paragraph style, the width of a
column and the points of a curve are all values in containers, and get
their merging from the container they are in.

## Two layers

**Containers** (crate `ow-crdt`, phase 2) know nothing about offices.
They are data structures of the kind called conflict-free replicated data
types, and this layer is the only place where merging is implemented:

- A **map** from names to values. When two devices set the same name,
  the later one wins, and "later" is decided the same way everywhere.
- A **sequence**, in which things have an order and can be inserted
  between other things: the characters of a text, the rows of a sheet,
  the pages of a deck. Two people who type at the same place both keep
  their words, and the words of one are not shuffled into the words of
  the other.
- **Text**, a sequence of characters with **marks** on ranges of it:
  bold from here to there, a comment on this sentence. A mark is tied to
  the characters at its ends and not to their positions, so it stays
  where it belongs while others type before it.
- A **tree**, in which things have a parent and an order among their
  siblings, and can be moved: the shapes of a slide and their groups.
  Two moves that would make a thing its own ancestor are settled the
  same way everywhere.

A value in a container is a number, a text, a truth value, bytes, or
another container.

**The document** (crate `ow-model`, phase 2) says what the containers
mean. It defines the objects an office document is made of, each as a
fixed arrangement of containers, and it is the only way editors and file
formats touch a document:

- A **text flow** is text whose marks carry character formatting, with
  the end of each paragraph as a character that carries the paragraph's
  properties. A table, a picture or a footnote sits in the flow as one
  character that stands for another object.
- A **grid** is a sequence of rows, a sequence of columns, and a map from
  a row and a column to a cell. Rows and columns are named by what they
  are and not by where they are, so inserting a row above does not
  change what any formula points at, and two people inserting rows at
  the same place both get theirs.
- A **canvas** is a tree of shapes, each a map of its properties. A shape
  that holds text holds a text flow.
- A **document** is a map of these: the pages of a deck, the sheets of a
  workbook, the flow of a text. One document can hold all three, which
  is how a chart on a slide can be drawn from a sheet in the same file.

## Operations, and who made them

Every device that edits has a number of its own, chosen at random. Each
operation carries the device's number and a count, and that pair names
the operation for ever. It also names what the operation made: the
character that was typed, the row that was inserted.

Operations that were made together, by one stroke of a key or one drag,
form a **change**. A change says which changes its device had seen when
it was made. A device applies a change once it has applied those, and
keeps it waiting until then. So operations are never applied to a
document that does not have what they refer to yet.

"Later wins" needs a clock that every device agrees on without sharing
one. Each change carries a count that is higher than every count its
device has seen, and where two counts are equal the device number
decides. A wall clock is recorded for people to read and decides nothing.

## Where two people type at once

A sequence remembers, for every element, what was beside it when it was
inserted. From that, every device builds the same order. The rule is the
one published as Fugue: an element is attached to its left neighbour,
unless that neighbour already has something attached to its right, in
which case it is attached to its right neighbour instead. Elements
attached at the same place are ordered by device number. Words typed at
the same place by two people come out one after the other and never
letter by letter in between.

Deleting does not remove an element. It marks it as gone, because an
operation that has not arrived yet may refer to it.

## History, undo, and the file

The operations are the history. A version of the document is a set of
changes, and any version can be rebuilt.

Undo is not "go back to the version before". Other people's changes may
have arrived since, and they must stay. Undoing a change makes new
operations that take back what that change did and nothing else.

A file holds the document as it is now in a compact form, and the
operations since. Opening a file must not replay a million operations.
A file that was imported from another format starts as a document with
no history, so that a sheet of a million cells does not carry a million
operations.

## What is decided and what is not

Decided: operations and not states are what devices exchange. Containers
are the only thing that merges. Rows and columns have identities.
Deleted elements are kept as marks.

Not decided:

- How operations are encoded in a file and on the wire, and how history
  that nobody needs any more is dropped.
- How a formula names a cell once rows have identities, and how fast a
  range of a million cells can then be read.
- How much history a document keeps when it is shared with someone who
  should not see its past.
- How large a document can get before the marks of deleted elements have
  to be collected, and how that is done while devices are apart.

Each of these gets a section here when it is settled, with what was
rejected.
