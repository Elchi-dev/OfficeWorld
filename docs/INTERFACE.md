# The interface

This page is a design. The layer it describes is not built: what exists
is the platform layer under it and the reference renderer. It is written
down before the code because every app of every phase is written against
it, and because phase 1 is judged by how this layer feels.

## What it has to do

- Feel right. A panel that is half open when it is told to close turns
  around from where it is. Nothing waits for an animation to finish, and
  nothing moves differently on a faster screen.
- Answer in the next frame. A key press or a click is on screen with the
  frame that follows it, on a screen that shows 120 of them a second.
- Rest. A window in which nothing moves draws nothing.
- Say what it shows. A screen reader, a test and the command palette all
  read the same description of what is on screen.
- Carry the apps. A few million lines will be written against it. What an
  app has to write for a button, a list or a dialog has to be short, has
  to compile fast and has to be testable without a screen.

## Three trees

**The description.** An app does not build its interface once and then
poke at it. It has a state, and a function from the state to a
description of what should be on screen: a column, in it a title, under
it a button named Save that is dimmed while nothing has changed. The
description is plain data and cheap. It is made again whenever the state
changes.

**The widgets.** The description is not what is drawn. Behind it stands a
tree of widgets that stays: each one has a place and a size, knows
whether the pointer is over it, where it is scrolled to and how far its
animations have come. When a new description arrives, it is compared
with the one before, part by part. A part that is still there keeps its
widget and tells it what changed. A part that is new gets a widget, and
one that is gone loses it.

**The semantics.** Every widget says what it is to someone who cannot
see it: a button, its name, that it is dimmed, what activating it does.
Together that is the tree a screen reader walks. Tests read it too, so a
test says "press the button named Save" and not "click at 412, 96".

The reason for this shape is motion. The app says where things should
be. The widget knows where they are. Between the two sits a spring. A
description that changes its mind three times in a tenth of a second
gives one continuous movement, because each change only moves the
target. Nobody writes an animation for a panel. They say that it is open.

## A frame

Nothing in this layer reads a clock or starts a thread. The platform
calls it with events, and one of them is "a frame is due at this time".

1. **Events** arrive: the pointer moved, a key went down. Each goes to
   the widget it belongs to, by where the pointer is or by which widget
   has the keyboard. A widget answers by changing itself, and by sending
   an action when it means something to the app: the button was pressed.
2. **Actions** reach the app's state through the description that asked
   for them. If the state changed, the description is made again and the
   widgets are brought up to it.
3. **The frame.** Every widget that is moving is asked where it is at the
   time of the frame. Widgets whose content or size changed are laid out
   again, and those alone. Widgets whose look changed are drawn again,
   and those alone: what a widget draws is kept until it changes, so a
   list that scrolls is moved and not drawn anew.
4. **Drawing.** The result is a scene, and the renderer draws the part of
   the window that differs from the frame before.
5. If something is still moving, the next frame is asked for. If nothing
   is, nothing is asked for, and the app sleeps until the next event.

Because the time of a frame is passed in, a test can play an afternoon
of use in a second, and gets the same pictures every time.

## Layout

A widget is told how much room there is, at least and at most, and
answers with the size it takes. It asks the same of its children and
then puts them in their places. One pass down and up, and a widget that
did not change and is asked the same question is not asked again.

Rows, columns and grids follow the rules of CSS flexible boxes and CSS
grids. The rules are known to everyone who has built a web page, they
are specified to the pixel, and a browser can be asked what the right
answer is. The layout code is checked against one.

Sizes are in units that do not depend on the screen. A screen that shows
two pixels to the unit gets twice the pixels and the same layout, and
hairlines and the edges of text are placed on whole pixels of the screen
that shows them.

## Pointer and keyboard

The widget under the pointer is found by going through what was drawn,
topmost first. A widget that is pressed keeps the pointer until the
button is let go, also when the pointer leaves it, so a drag does not
end at the edge of a slider.

One widget has the keyboard. A key goes to it first and then up through
its parents, which is how a shortcut reaches the window when a text
field has no use for it. Tab moves the keyboard through the widgets in
the order of the tree. Every action that can be done with the pointer
can be done with the keyboard, and what has the keyboard always shows
that it does.

## The design system on screen

Widgets take their colours, sizes and springs from the design system
(`ow-design`) and from nowhere else. No widget names a colour of its
own. Light and dark are two sets of the same thirteen colours, and a
person who asked their system for less motion gets springs that do not
swing and moves that are short.

## The parts

- `ow-scene`: what a frame looks like, as data. Shapes, pictures and
  text with their paints, clips and layers. A renderer draws it, a PDF
  writer can write it, a test can read it.
- `ow-layout`: rows, columns and grids. Pure arithmetic over boxes, so
  pages and tables can use it too.
- `ow-ui`: the three trees, events, focus, frames, and a harness that
  runs an interface without a screen for tests.
- `ow-widgets`: buttons, fields, lists, menus, tabs, scrolling and the
  rest of the controls, drawn with the design system.
- `ow-text`: from characters and fonts to lines that can be drawn,
  measured and clicked into.

## What is decided and what is not

Decided: the interface is drawn here and not by the system's own
controls. The app describes and the widgets remember. Animation is the
difference between the two and is on by default. Time comes from
outside. The semantics tree exists from the first widget.

Not decided:

- **The renderer on the GPU.** The reference renderer draws on the CPU.
  It is exact, and it is fast enough for a window in which a part
  changes. It is not fast enough to redraw a large screen 120 times a
  second. Which graphics interface the GPU renderer speaks on each
  system is open.
- **Input methods.** Writing Chinese, Japanese or Korean, and accents
  typed in two strokes, need a conversation between the text field and
  the system that the platform layer does not have yet.
- **How the semantics tree reaches the screen readers** of each system.
- **Windows inside the window.** Menus and tooltips that reach past the
  edge of their window need windows of their own on some systems.

Each of these gets a section here when it is settled, with what was
rejected.
