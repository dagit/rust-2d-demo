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

## Windows

1. Visit SDL [download page](https://www.libsdl.org/download-2.0.php)
  and grab the correct SDL development libraries. I use the msvc
  toolchain, so I grab the Visual C++ development libraries.

  If you're on msvc you'll need to set `LIB` to point to `SDL2.lib`. In
  my case, I unzipped the files to my desktop and I set the following
  path:

  ```sh
  export LIB="C:\Users\dagit\Desktop\SDL2-2.0.5\lib\x64"
  ```

  On the GNU toolchain you set `LIBRARY_PATH` instead of `LIB`.

2. Copy `SDL2.dll` into the top of your crate (next to `Cargo.toml`).

3. Checkout/update git submodules:

    ```sh
    $ git submodule update --init --recursive
    ```

  If you encounter an error during this step, I may need to give you access to
  the scene-rs repository.

4. Use cargo as normal, eg., `cargo build`


## Linux

TODO

## OSX

1. Checkout/update git submodules:

    ```sh
    $ git submodule update --init --recursive
    ```

2. Follow the instructions for `rust-sdl2` for OSX: [https://github.com/AngryLawyer/rust-sdl2#mac-os-x](https://github.com/AngryLawyer/rust-sdl2#mac-os-x)

    For homebrew users this is roughly:

    ```sh
    $ brew install sdl2
    $ export LIBRARY_PATH="$LIBRARY_PATH:/usr/local/lib"
    ```

3. Use cargo as normal, eg., `cargo build`
