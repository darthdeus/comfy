# [![Comfy](assets/comfy-mid.png)](https://comfyengine.org)

[![Crates.io](https://img.shields.io/crates/v/comfy.svg)](https://crates.io/crates/comfy)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/darthdeus/comfy#license)
[![Crates.io](https://img.shields.io/crates/d/comfy.svg)](https://crates.io/crates/comfy)
[![Rust](https://github.com/darthdeus/comfy/workflows/CI/badge.svg)](https://github.com/darthdeus/comfy/actions)
[![Discord](https://img.shields.io/discord/720719762031771680.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/6NGGGTUz7x)

# What is `comfy`?

**_Comfy is now archived until further notice._ There's a few reasons for this.
Mainly, due to real life circumstances I have much less time/capacity to
dedicate to side projects. After [abandoning Rust for
gamedev](https://loglog.games/blog/leaving-rust-gamedev/) Comfy became a small
side project for fun, and while this worked for a little while, it's no longer
the case, and I just don't have the energy to constantly play catchup to the
Rust ecosystem.**

It might not be obvious how much effort it takes to manage bugfixes in
dependencies where every few weeks there's a new breaking API change in
egui/winit/wgpu, and while these probably seem extremely minor to those who
spend all their time building an engine on top of said libraries, sinking a day
or two in figuring things out and fixing stuff on every release is a gigantic
waste of time in my view.

At this point I've fully switched to developing games in C++, and I'm _very_
happy with this choice. There's a chance I might come back to Comfy and fix
some of the current issues, but as of right now I really don't want to do that.

I'd suggest those using Comfy to simply fork the repo and adopt things to their
needs. The code should be simple enough for anyone who really cares to just do
things and add/remove stuff without too much friction.

But mainly I'd just suggest people either use Macroquad, or roll their own
solution on top of OpenGL. Both of these are good choices for 2D. If you're
making a 3D game, well, you better know what you're doing :)

As a small tangent, personally, I've lost all faith in WebGPU/wgpu for
2D games, and am really really really happy in OpenGL land in C++. I'm not
saying C++ is a good language, and I'd probably suggest people use C# with
MonoGame/FNA (which I've tried and works with hot reloading), or just use
SDL/OpenTK or some variant (haven't tried too much, but I can't imagine there
being major issues).

Mainly, if your goal is to make a game, focus on the game and not the tech.
Rust is a fun language, but please make your own decisions, instead of just
following what the loud groups of people say. I can't count how many times
people told me "but what about segfaults???" when I said I was switching from
Rust & C# to C++, and honestly, segfaults are the least of my issues. Null
pointers are also a complete non issue. At the risk of being controversial, I
have very little faith in the experience of people saying this, because it
seems that people outside of the Rust community also don't really consider
these to be "a huge deal" compared to what the Rust community makes it sound
like.

**Currently there's many performance improvements on Comfy's master branch that
haven't been released yet. If you're struggling with performance, consider
using the master branch. More info can be found [in the
CHANGELOG](https://github.com/darthdeus/comfy/blob/master/CHANGELOG.md).**

**If you're new here, check out the [comfy announcement on our
blog](https://comfyengine.org/blog/first/) and the [v0.2 release
announcement](https://comfyengine.org/blog/release-v0-2/).**

Comfy is a fun 2D game engine built in Rust. It's designed to be
opinionated, productive, and easy to use. It uses [wgpu](https://wgpu.rs/)
and [winit](https://docs.rs/winit/latest/winit/), which makes it
cross-platform, currently supporting Windows, Linux, MacOS and WASM.
Inspired by macroquad, Raylib, Love2D and many others, it is designed to
just work and fill most of the common use cases.

**Warning**: comfy is currently under heavy development. While there are
games already being made using comfy, the API is not yet stable and
breaking changes will happen. If you want to use comfy for your game you
may be forced to dig into the source code and possibly tweak things
manually. That being said, the source code is designed to be simple and
modifiable. If you want to make a game jam game comfy is definitely mature
enough.

> `comfy` is named comfy, because it is very comfy to use.

```rust
use comfy::*;

simple_game!("Nice red circle", update);

fn update(_c: &mut EngineContext) {
    draw_circle(vec2(0.0, 0.0), 0.5, RED, 0);
}
```

The ultimate goal of comfy is to do the obvious thing as simply as
possible without unnecessray ceremony. If something is annoying to use, it
is a bug that should be fixed. We're not necessarily aiming at beginner
friendliness, but rather productive and ergonomic APIs. If you're a
beginner, comfy should be easy to pick up, but it might not be as polished
as some of the other alternatives. The goal of comfy is ultimately not
polish, cleanliness of API, clean design, type safety, extensibility, or
maximum features. It's an engine that gets out of your way so you can make
your game.

There is nothing that fundamentally prevents comfy from becoming a 3D
engine, but the last thing we want is to try to fight rend3 or bevy in
terms of PBR accuracy or skeletal animations. Comfy is not fighting
against Unreal Engine 5. It would be nice if
[simple](https://store.steampowered.com/app/824600/HROT/)
[stylized](https://store.steampowered.com/app/1055540/A_Short_Hike/)
[3D](https://store.steampowered.com/app/219890/Antichamber/)
[games](https://store.steampowered.com/app/219890/Antichamber/) were
ultimately possible, but we want to get all of the basic building blocks
for 2D first. Some internals of comfy (batching and z-sorting) will need
to be re-implemented to allow for this and ultimately more performant
rendering techniques, but this should not happen at the cost of API
clarity and ergonomics for most games.

# Features

- Simple and productive API.
- Immediate mode rendering for sprites, text and shapes with automatic
  batching. If you want to draw a circle, you call a function `draw_circle`.
- 2D lighting with HDR, tonemapping and bloom.
- Built-in support for z-index, meaning you don't have to worry about the order
  of your draw calls.
- [egui](https://egui.rs/) support built in.
- Parallel asset loading with support for most image and audio formats.
- No complex ECS or abstractions to learn. Just build your game and let comfy
  get out of your way.
- Simple audio using [kira](https://docs.rs/kira/latest/kira/). If you want to
  play a sound, you call a function `play_sound`.
- Simple 2D camera.
- Particles, both simple API for individual particles & systems with lots of
  options.
- Trails with a custom mesh & scrolling texture.
- Text rendering (currently using egui).
- Lots of utilities for common tasks.

# Design goals & philosophy

- Heavy focus on ergonomics and productivity.
- No magic. The code does what it looks like it does.
- Targeted at simple games, currently only 2D.
- Opinionated and useful defaults.
- **Simple** immediate mode APIs for almost everything.
- Exposed internals for when you need more. Almost all struct fields are
  public, comfy doesn't keep things away from its user.
- Reasonable compile times. Comfy is slower to compile than macroquad, but we
  want to avoid things getting out of hand. End users are not going to be
  required to use any proc macros to use comfy.
- Global variables are nice. Comfy uses a lot of them.
- Typing less is nice. Comfy has a single context object that gets passed around everywhere.
- Constraints are nice. Comfy wants to be used for a lot of games, but not all
  of them.
- `RefCell`'s are nice. Comfy uses them a lot to work around partial borrows.
  We tried doing things without them multiple times, it was more painful.

# Non-goals

- AAA 3D support. While it's entirely possible to extend the renderer to handle
  3D, this was intentionally not done yet. There is a lot of complexity that
  comes with 3d models, materials, skeletal animations, etc. Comfy may grow to
  support simple 3d games in the future, but it is extremely unlikely it'll
  ever attempt to be competitive with big 3D engines. We want to make sure that
  the stuff we have works well and is usable before adding lots more complex
  features.
- ECS based engine. While comfy does embed [hecs](https://docs.rs/hecs) and
  provides some helpers for using it, it is by no means required or even
  optimal for most cases.
- Modularity. Comfy is not a modular engine. It's an opinionated toolkit
  with defaults that make sense for most games. There is no intention of
  having a plugin system or the ability to replace wgpu with something
  else.
- Maximum performance. Comfy is not designed to be the fastest engine out
  there. There are many tradeoffs made for the sake of ergonomics and ease of
  use, some of which affect performance. If you're looking for the fastest way
  to draw a million quads, comfy is not for you. If however you have a
  legitimate use case where the performance is not good enough, please open an
  issue. There is a lot of low hanging fruit with respect to performance, but
  as the development is driven by real world usage, unless something shows up
  in a profiler in a game, it's unlikely to be optimized further.

# Getting started

The repository contains many examples under the `comfy/examples` folder.
While there is currently no documentation, the API is simple enough that
just reading the examples should explain things.

# Why use comfy and not X?

## [macroquad](https://macroquad.rs/)

Before I started working on comfy I was using [macroquad](https://macroquad.rs/)
for my games. It works great, but a few things were missing, most notably
RGBA16F textures, which are a feature of OpenGL 3.x, and without which HDR is
not really possible. This is because macroquad targets older versions of GLES
to achieve better cross-platform support. While this is great for many use
cases, at the time I really wanted to play with HDR, bloom and tonemapping,
which lead me down the [wgpu](https://wgpu.rs/) path.

The first version of comfy actually had an API almost identical to macroquad,
where I basically copy pasted function definitions and implemented most of the
functionality on top of wgpu instead. Over time I realized I wanted a few more
things, namely built-in z-index so that my game code wouldn't have to worry
about draw order.

If you like the idea of comfy but it's not stable enough for your use case I
very highly recommend giving macroquad a try. While it is not perfect it has
helped me build a bunch of small games, and most importantly I had fun while
making them.

### Differences between `comfy` and `macroquad`

Macroquad is the biggest inspiration to comfy, and as such there are many
things which are similar, but there are quite a few differences.

**Coordinate systems:**

- Macroquad's coordinate system is `[0, 0]` in top left, `y`-down, measured in
  pixels.
- Comfy's coordinate system is `[0, 0]` in the center, `y`-up, measured in
  world units. Default camera has zoom set to `30`, which means you can see
  roughly `30` world units. In a pixel-art game with 16x16 sprites, you would
  ideally set your camera's zoom so each sprite is `1` world unit.

**Z-index built in**. In macroquad, draw calls happen in the order you call
them. In comfy, almost everything (excluding text and UI) accepts a `z_index:
i32`. This means you don't need to sort the calls yourself, comfy will do it
for you while still batching the draw calls as best it can.

**HDR render textures**: Macroquad targets GLES2/3 to support as many platforms
as possible, and as such it can't support RGBA16F textures. Comfy targets
desktop and WASM through WebGL 2, both of which allow `f16` textures, and thus
all rendering is done with HDR and tonemapped accordingly. This allows our
bloom implementation to work off of HDR colors and greatly simplify working
with lights, as the light intensity can go well beyond 1.

**Batteries included**: Comfy includes many extra things that macroquad does
not, for example egui itself is part of comfy and likely will remain this way
until a better alternative comes along. Macroquad and miniquad provide small
flexible building blocks, while comfy aims to be a full and relatively
opinionated way of making games.

There are many more subtle differences, but in principle you can think of as
comfy as "macroquad with more batteries included, built on top of wgpu, with
less cross platform capabilities". Note that because comfy builds on wgpu and
not OpenGL we don't have the same immediate mode interactions with GL. This
makes some things more difficult, e.g. render targets, changing shader
uniforms, etc.

Comfy intends to support all of these features, but it will take a bit more
development. Many engines (e.g. bevy and rend3) end up using render graphs in
order to expose the rendering logic to users. While these are very flexible and
offer high performance their APIs are anything but simple.

Since our intention is not to support AAA graphics the goal should be to find
some form of middle ground, where we could achieve something similar to
macroquad in terms of API simplicity, expressivity, and fun, while utilizing
all of the power wgpu has to offer.

The ultimate design goal of comfy is that most of its API should be
understandable from just looking at the type signatures, without needing to
study documentation in depth, and without excessive footguns.

## [rend3](https://rend3.rs/)

I don't have much experience with rend3 apart from digging a bit through its
code, but as a 3d renderer it fills a very different niche than comfy. If you're
building a 3d game and don't want to do PBR rendering, rend3 is probably
something you want to consider.

## [Fyrox](https://fyrox.rs/)

Fyrox seems like it is trying to fight Unity, Godot and Unreal head on by
currently being the only fully featured Rust game engine, notably also
including a full 3D scene editor. Its 3D demos are very impressive in
particular, and if you're looking for a fully featured 3D engine it's
definitely something to consider.

That being said, comfy is unapologetically focused on simple games, and as such
fills a very different niche than Fyrox.

## [bevy](https://bevyengine.org/)

Bevy is another contender for the "big Rust game engine" spot. In terms of its
2D features Bevy definitely wins on the size of community and overall crate
support and modularity, but this is something where comfy is not even attempting
to compete. comfy is designed to be opinionated, simple and pragmatic, while
Bevy's goal is to be modular, extensible and build on top of its
all-encompassing ECS.

Due to its modularity Bevy offers many more features through community asset
crates which greatly extend it, but also has a rather distributed and unstable
ecosystem.

Comfy's goal is opposite in many ways. The goal is to provide a simple, stable
and pragmatic foundation. comfy is not a platform for experimenting with Rust's
type system, ECS, or other abstractions. It's a toolkit designed for making
small games.

The only features you'll find in comfy are those which can be immediately used,
understood, and that work from day 1. If a feature is not being used in a real
game it won't appear in the engine source code.

## [godot-rust](https://godot-rust.github.io/)

If the goal is to "actually make a game", especially in 3D, then godot-rust is
very likely the winner. No rust engine can match what Godot offers, and having
used godot-rust to make
[BITGUN](https://store.steampowered.com/app/1673940/BITGUN/) over the course of
a year we can say that it is very mature, stable and well maintained.

However, the main benefit (Godot) is also its greatest downside for us. We've
found that code-based frameworks are much more fun to use. Many people consider
GDScript to be the problematic part in Godot, but when working on BITGUN it
actually helped us quite a bit, as there are many things which "only need a few
lines of code" and don't really benefit from using Rust.

Especially if you're considering making a 3D game, godot-rust is probably the best
option of helping you ship something.

## [ggez](https://ggez.rs/)

ggez is one of those libraries that have been around for a while, but I've
never really got a chance to use it. It does seem to have a bit of a history
with losing maintainers, which is why I never got to use it, as both times when
I was switching frameworks/engines in Rust it was unmaintained. Although in the
current version it did get upgraded to a wgpu-based backend, but I can't speak
to its quality. I would imagine it's a great alternative to macroquad.

---

There are many other frameworks/engines in Rust, but I haven't had a chance to
interact with those in any significant way, hence why they're not in this
comparison.

# Roadmap

The following goals are not in any particular order, but should come reasonably
soon. Comfy is not an aetheral project that will only materialize in 2 years.
Only features that require maximum few weeks of work are listed here.

- Improved lighting. Right now we do have 2d lights, but they're basic,
  ugly in some scenarios, and not very flexible.
- Configurable bloom. Currently bloom is hard-coded to simplify a few things
  and always enabled. We don't want to delay the release to fix this since it
  does make games look better by default, but it is one of the first few things
  that will get fixed after v0.1 release.
- Configurable post processing.
- Custom shaders/materials.
- Render targets.
- Gamepad & touchpad support.
- Antialiasing.
- 2D shadowcasters with soft shadows.
- Asset packing without `include_dir`. Right now comfy relies on either its
  builtin use of [include_dir](https://github.com/darthdeus/include_dir) (a
  small fork with a few extra features), or the user handling asset loading
  manually. There are many other ways of packing assets, and it would be cool
  to support those, but we don't currently because for reasonably (<1GB) sized
  assets `include_dir` works well enough.
- Text rendering without egui. Right now all text (drawn with `draw_text` and
  friends) is rendered using `egui`'s painter on a separate layer. This gives
  us a lot of features in terms of text rendering, but also comes with some
  limitations. The goal is to implement text rendering on top of just wgpu.
  We've tried a few different approaches (e.g. `glyphon`) but ultimately found
  none to be easy enough to just replace what we have in `egui`, and since no
  games were yet blocked on more flexible rendering this remains a relatively
  low priority problem.
- Overall engine/renderer code cleanup. The code in comfy is not beautiful as
  it developed organically while building multiple games. There are some
  features that could be better exposed, and some remains of what our games
  needed. The provided examples should serve as a foundation to make sure comfy
  is flexible enough, but it is an ongoing effort to improve the codebase. That
  being said, almost everything you find in comfy should work to a reasonable
  extent.
- Reduce re-borrows & `RefCell`s. Right now we use _a lot_ of `RefCell`
  for almost everything. While this helps in a few places there are many
  places where it is not necessary, and where we also excessively borrow
  and re-borrow multiple times per frame. Currently we haven't noticed any of
  this impacting performance, but it is something that should be cleaned up.
  There's also a few things which use a `Mutex` unnecessarily.

While comfy is ready to use, the codebase is far from clean. The engine
evolves rapidly as we work on our games, and there are many parts that can
and will be improved. Comfy is being released before it is 100% perfect,
because even in its current state it can be very well used to make 2D games.

There may be a few oddities you may run into, and some internals are
planned to be re-done, but anything covered by the examples should 100%
work. We have been using comfy internally for over 6 months, and a large
part of its codebase has been ported from our previous OpenGL based
engine. This doesn't mean the engine is mature, but we have had real
players play our games made with comfy.

# Contributing

Comfy is still very early in its lifecycle. While it has been used to make
games, only a few people have used it or even seen the source code so far.
The best way to contribute is to use comfy and report any issues you find.

The codebase is not clean by any means. It is not the goal of comfy to be the
most beautiful codebase out there. Many things may be suboptimal, and for some
of them it makes a lot of sense to have an open discussion about it. But pull
requests which just reformat the code or move things around or do some kind of
re-organization will likely be rejected unless there was a prior discussion.

If you find anything that does not work as expected please do open an issue.
Comfy is meant to be a productive and ergonomic companion for those who want to
make games.

If something is not ergonomic or you have an idea for how it could be more ergonomic
without sacrificing too much, please open an issue.

If you really just want to make a pull request to contribute _something_
without a prior discussion, the best place are the examples. Both simple and
advanced examples, as well as small example games, are welcome.

Comfy is not currently aiming for heavy documentation coverage due to the rapid
pace of development. Examples are preferred to documentation as they're easier
to fix when APIs change. Most things should be self-explanatory.

If you'd like to chat about anything comfy related, [join our discord
server](https://discord.gg/M8hySjuG48).

# License

Comfy is free and open source and dual licensed under MIT and Apache 2.0 licenses.

# Games using comfy

Comfy is being used by [LogLog Games](https://loglog.games/), with one game released on Steam, namely [Unrelaxing Quacks](https://store.steampowered.com/app/2331980/Unrelaxing_Quacks/) - survivors, but fast!

We've also used comfy in a few smaller games, e.g. our [1-bit jam
entry](https://logloggames.itch.io/bitmob-1-bit-jam) where we experimented with
CPU textures and 2D raytracing.

Comfy has been supported by [Jetbrains](https://www.jetbrains.com/) through their opensource initiative and providing licenses for Comfy development.
