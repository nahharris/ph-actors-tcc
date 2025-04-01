# Patch Hub

`patch-hub` is a text user interface (TUI) application that is ran on the terminal.
It's a helper for kernel developers to view patches using [lore](https://lore.kernel.org).
Currently, it's a mostly single-threaded application written in Rust. Due to Rust
rigorous type system and _don't pay for what you don't use_ philosophy, `patch-hub`
,as is, is hitting the flexibility limits. Which means that huge refactorings
are needed in order to create new features.

The best example is the popup system. Currently, popups are limited to only displaying
data. They sadly can't modify data due to some architectural decisions and Rust
type system restrictions.

In Rust, every value must have one and only owner. There's an `app` variable that
holds both the configurations and the popups to be displayed, together with the
application state information. To modify the app, we need to take a `&mut`
(mutable reference) to it. But for a popup to modify data in the app it needs
this `&mut` (remember that the popup is owned by the app). There's a tricky rule
in Rust that prohibits you to have two simultaneous `&mut` to the same piece of data
or even have `&` (immutable references) and `&mut` at once. So, to access the popup,
we need to have a reference to the app, but we also need to take a reference to
the app and pass it to the popup. So we need to have two simultaneous references
where one of them is mutable. This is a Rust rule violation.

Those Rust rules might seem restrictive, but that's what makes Rust safe. Rust
won't absolutely prohibits you from modifying values. But you need to explicitly
use safe abstractions to do so.

## Modules

Currently `patch-hub` is broken up into the following modules:

- `lore`: interface to interact with the lore API
- `ui`: draw to the terminal
- `cli`: CLI arg parse for patch-hub binary
- `app`: manages patch-hub data and state
  - `logger`: handles logging
  - `popup`: creates popups
  - `config`: manages app configurations
- `handler`: handle user input

## Actors

Not every module translates directly to an actor, since actors should be responsible
or one thing and one thing only. So this is a rough idea of the actors that we
would build.

- **Lore**(`lore`): interface to interact with the lore API
- **View**(`ui`): draws to the terminal
- **App**(`app`): manages app state
- **Config**(`app::config`): manages app configuration
- **Controller**(`handler`): handles user input
- **File**(spread across different locations): manages file IO
- **Logger**(`app::logging`): logs messages
- **Terminal**: handles terminal interactions

Also, keep in mind that we won't stick to the "everything is an actor" philosophy.
Some pieces of the `patch-hub` code will be just reorganized in different folders
but still be just regular library structs, functions, etc. outside of the actor model.

The actors will communicate with each other using messages. The main messages that will
be handled are:

- **Lore**: will be responsible for fetching data from the lore API

  - `Lists`: return the mailing lists from lore
  - `Page`: get a page of patches from a mailing list
  - `Details`: get the details of a patch

- **View**: is responsible for rendering the UI on the terminal

  - `Render`: render a given screen with some payload data
  - `Detach`: detaches the view from the terminal allowing for other CLI tools to use it
  - `Attach`: attaches the view back to the terminal

- **App**: controls the application state

  - `[Set]State`: get/set the state of the app
  - `Text`: send a text character to the app for input buffers

- **Config**: deals with configuration options

  - `[Set]*`: couple of getters and setters for each configuration option

- **Controller**: won't receive incoming messages, but will send messages to other actors when a given chord or melody happens based on the app state and defined keybindings

  - `Chord`: register that a combination of keys where pressed at the same time
  - `Melody`: defines an action to be performed when a given melody happens

- **File**: deals with file IO

  - `Read`: read a file
  - `Write`: write a file

- **Logger**: responsible for logging information

  - `Debug`: log a debug message
  - `Info`: log an info message
  - `Warn`: log a warning message
  - `Error`: log an error message

- **Terminal**: handles terminal interactions

- Git: will do git operations

The amount of messages each actor receives is a good indicator that the actor is not
doing too much. If an actor receives too many messages, it might be a good idea to
refactor it into smaller actors.
