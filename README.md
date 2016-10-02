# Overview

This is a template for starting 2D game projects in Rust. The choice of libraries are only meant as examples, but they have been carefully chosen such that:

  * We get cross-platform support (defined to be at least Linux, Windows, and OSX)
  * They are compatible with each other

Furthermore, we define the minimal requirements of 2D games as follows:

  * HUD elements -> font rendering, orthographic textured quads, etc
  * GUI elements -> font rendering, click events, text entry fields, buttons, that sort of thing
  * Audio -> sound effects and background music
  * Mouse events
  * Keyboard events

In addition to the minimal requirements we also hope to reuse this template for starting 3D game projects. As such we also include full OpenGL support.

# Building

## Widnows

TODO

## Linux

TODO

## OSX

1. Checkout/update git submodules:

    ```sh
    $ git submodule init
    $ git submodule update
    ```

2. Follow the instructions for `rust-sdl2` for OSX: [https://github.com/AngryLawyer/rust-sdl2#mac-os-x](https://github.com/AngryLawyer/rust-sdl2#mac-os-x)

    For homebrew users this is roughly:

    ```sh
    $ brew install sdl2
    $ export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"
    ```

3. Use cargo as normal, eg., `cargo build`
