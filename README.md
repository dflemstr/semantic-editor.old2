# `semantic-editor` [![CircleCI](https://circleci.com/gh/dflemstr/semantic-editor.svg?style=svg)](https://circleci.com/gh/dflemstr/semantic-editor)

A versatile editor for different kinds of content.

Edits content *semantically*.  You don't manipulate characters, but rather the structure of your
content.  It is impossible to make syntax errors or break style guides. 

This program is in an early state of development!

## Development

To develop on `semantic-editor`, you need to install:

  - Rust, it's recommended to follow the instructions at <https://rustup.rs/>.
  - The nightly Rust compiler with `wasm32` support:

    ```text
    rustup toolchain install nightly
    rustup target add wasm32-unknown-unknown --toolchain nightly
    ```
  - node.js, at least version 8.9.4 (<https://nodejs.org/>)
  - The `yarn` build tool for node (<https://yarnpkg.com/>)
  - `wasm-bindgen` from <https://github.com/alexcrichton/wasm-bindgen>

Starting the application should be as simple as doing `yarn start`.

## Architecture

For now, `semantic-editor` doesn't have a frontend on its own, and must use a web browser to render
itself.

The general idea is that the editor runs as a HTTP service, and you can open a view of the editor in
your web browser.  Your browser tabs share the same editor state, and can be thought of as "frames"
in an editor like Emacs; if you have a file open in two tabs, any changes made in one tab will
eventually show up in the other.

The architecture involves these components:

```text
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Backend                                                       ┃
┃ ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ ┃
┃ │ semantic-editor │  │ platform native │  │   File system   │ ┃
┃ │    process      │←→│       tool      │←→│   Network I/O   │ ┃
┃ │  (rust native)  │  │   (e.g. javac)  │  │  Native Runtime │ ┃
┃ └─────────┬───────┘  └─────────────────┘  └─────────────────┘ ┃
┗━━━━━━━━━━━┿━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
            │
        Unreliable
        connection
            │
┏━━━━━━━━━━━┿━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Frontend  │                                                   ┃
┃ ┌─────────┴───────┐  ┌─────────────────┐  ┌─────────────────┐ ┃
┃ │ Browser service │  │ semantic-editor │  │Editing interface│ ┃
┃ │     worker      │←→│     module      │←→│   in browser    │ ┃
┃ │   (Javascript)  │  │   (rust wasm)   │  │ (TS/React/Redux)│ ┃
┃ └─────────────────┘  └─────────────────┘  └─────────────────┘ ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛
```

The user creates a new frontend instance by connecting via HTTP to the backend process.  The backend
transfers a copy of all of the static files needed to bootstrap the service worker, the
semantic-editor WebAssembly module and the editing interface code.  At this point, the frontend and
backend communicate via an event/action-based protocol to synchronize state and they are completely
independent.

The backend is responsible for handling interaction with the outside world (invoking compilers,
reading and writing files, running programs, making commits etc).

The frontend is independently capable of performing any editor operation such as changing content,
searching symbolically, etc.

If the connection between the backend and the frontend is lost, events are buffered up and will be
consolidated when the connection is back up.  This might cause conflicts if the editor state has
changed significantly in the meantime.

## Semantics

The editor edits all content semantically.  That means that it's not possible to edit any general
text file; the editor requires support for each specific content format.

For now, the structure of content is described using Rust `struct`s.  When opening a file, the
contents are parsed into a format-specific AST and then manipulated as such, then serialized back
into an actual text format upon save.  `semantic-editor` will intentionally not preserve the
specific indentation/line breaks/etc of the original file, but rather enforce a global, consistent
style guide for each format.

Since the `semantic-editor` backend is written in Rust, it is possible to use native tools for
interacting with content.  `semantic-editor` should not implement its own Java linter/highlighter;
it should bind to `libjvm` and use the Java compiler directly to perform that task!
