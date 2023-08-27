# What is Embi?

Embi is a genuinely breezy 2D game framework built in Rust. It's designed
to be opinionated, pragmatic, and easy to use. It uses
[wgpu](https://wgpu.rs/) and [winit](https://docs.rs/winit/latest/winit/),
which makes it cross-platform, currently supporting Windows, Linux, MacOS
and WASM. Inspired by macroquad, it is designed to just work and fill most
of the common use cases.

**Warning**: Embi is currently under heavy development. While there are
games already being made using Embi, the API is not yet stable and
breaking changes will happen. If you want to use Embi for your game you
may be forced to dig into the source code and possibly tweak things
manually. That being said, the source code is designed to be simple and
modifiable.

# Features

- Simple and productive API.
- Immediate mode rendering for sprites, text and shapes with automatic batching. If you want to draw a circle, you call a function `draw_circle`.
- [egui](https://egui.rs/) support built in.
- Parallel asset loading with support for most image and audio formats.
- No complex ECS or abstractions to learn. Just build your game and let Embi get out of your way.
- Simple audio using [kira](https://docs.rs/kira/latest/kira/). If you want to play a sound, you call a function `play_sound`.
- Simple 2D camera.

# Design goals

- No magic.
- Heavy focus on 2D games.
- Opinionated and useful defaults.
- **Simple** immediate mode APIs for almost everything.
- Exposed and simple internals for when you need more.
- Lots of utilities for common tasks.

# Non-goals

- 3D support. While it's entirely possible to extend the renderer to
  handle 3D, it is an intentional feature to not even attempt this. There
  is a lot of complexity that comes with 3d models, materials, skeletal
  animations, etc. If you need this complexity, Embi is not for you.
- ECS based engine. While Embi does embed [hecs](https://docs.rs/hecs) and
  provides some helpers for using it, it is by no means required or even
  optimal for most cases.
- Modularity. Embi is not a modular engine. It's an opinionated toolkit
  with defaults that make sense for most games. There is no intention of
  having a plugin system or the ability to replace wgpu with something
  else.

# Getting started

The repository contains many examples under the `embi/examples` folder.
While there is currently no documentation, the API is simple enough that
just reading the examples should explain things.

# License

Embi is free and open source and dual licensed under MIT and Apache 2.0 licenses.
