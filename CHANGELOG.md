# v0.3.0

- Added `game_config().comfy_in_title` which controls whether the window title
  contains `(COMFY ENGINE)` suffix. As a small sidenote, one might want to keep
  this enabled to allow e.g. tiling window manager rules like `for_window
[title=".*COMFY ENGINE.*"] floating enable` in e.g. i3 and only disable it in
  release builds. Or just disable it altogether :)
- Notable upgrades: `wgpu 0.16.3 -> 0.17.1`, `egui 0.22.0 -> 0.23.0`. The
  `egui` upgrade is somewhat important, as `egui::plot` got moved into a
  separate `egui_plot` crate that Comfy now re-exports.
- Added `--feature git-version` that embeds the current git commit hash
  into the binary at build time. Note that this will make compilation fail
  if `cargo build` is run without there being any git history. See [the
  version
  example](https://github.com/darthdeus/comfy/blob/master/comfy/examples/version.rs)
  for details.
- Removed `--feature lua` and `mlua` integration. This was mainly a remnant of NANOVOID
  but was never implemented properly and missed a lot of bindings. If we do end up wanting
  to have official `mlua` bindings I'd rather that be done in a more principled approach
  where we make sure things are exported in a consistent way.

# v0.2.0

The main change in this release is that `EngineContext` is not necessary to
pass around anymore. This should simplify a lot of the confusion, as the #1
question about Comfy from _many_ people was about `GameContext` and
`EngineContext` and why are there two any why do we even need them.

Since Comfy already uses globals for many things, it makes sense to just
embrace this fully and move the remaining things to globals as well. Many
of the values provided in `EngineContext` were already available through
globals and just re-exported, so this shouldn't be a huge issue.

Comfy will still use `EngineContext` internally for some things, but this
won't be re-exported to the users as everything should be accessible by
other means.

List of removed things and where to find them now:

- `c.delta` -> `delta()`. This is likely going to be something that most users
  (including us) will re-export into their `GameContext/GameState` anyway.
- `c.world()` -> `world()`. ECS world already lived in a single instance, it's
  now moved into a single global.
- `c.commands()` -> `commands()`. Same as above.
- `c.cooldowns()` -> `cooldowns()`. This might be worth re-exporting into
  `GameContext` if accessed frequently, but in either way there's no extra
  overhead compared to before.
- `c.mouse_world` -> `mouse_world()`. This already existed before, and may
  also be worth re-exporting anyway.
- `c.egui` -> `egui()`. Note that before this was a field, now it's a
  global function. Though in this case `egui::Context` is already
  internally `Arc<Mutex<ContextImpl>>`, so this function is actually very
  cheap to call as it just returns a `&'static egui::Context` :)
- `c.egui_wants_mouse` -> `egui().wants_pointer_input()`
- `c.config` -> `game_config()` and `game_config_mut()`.
- `c.cached_loader.borrow_mut()` -> `cached_loader_mut()` (for `&` just
  omit `_mut`).
- similarly `c.changes.borrow_mut()` -> `changes()` and `c.notifications.borrow_mut()` -> `notifications()`.
  The last three were undocumented and are not really intended for public
  use yet, but documenting them here anyway.

NOTE: Comfy still includes many APIs which are not currently documented but are
still exposed. The current goal is to work through codebase and cleanup some
odd bits and document them at the same time. If you find something that is not
mentioned on the website or any examples, it's very likely to change in the
future.

As a secondary note, it should be noted that comfy is still _very early_ on in
its lifecycle. Comfy will do its best not to break existing games, but we may
need to iterate on some ideas, and some of them might be controversial, such
as the use of globals.

Comfy is not a project ran by RFCs, and while we do appreciate feedback, some
things have to be figured out by actually using the tool to build games,
running benchmarks, and making decisions based on real world usage.

In our usage of Comfy we've found many things that many in the Rust community
would consider "bad ideas" to be incredible boosts in ergonomics and
productivity. This is something that seems to happen more often than not, and
as such we're not really afraid to make changes like the above where a large
portion of the state is moved into globals. If you find the above change
unacceptable and what we had before "good", maybe take a look at the source
code and see how many globals we already had :)

That being said, the #1 priority of Comfy is and always will be making real
games. If any changes we make become problematic _in real world use cases_,
please do report these. If you think something is slow, please submit a
benchmark showing this. Comfy has enough examples using all of the systems, and
a builtin integration with Tracy, so it should be easy to extend. We do care
about reasonable games performing well on consumer hardware, but we do not care
about being the fastest at rendering 500k particles.

Our own games are not locked behind on an ancient version of Comfy, and we're
doing our best to stay up to date with the latest changes, to make sure things
are actually working smoothly.

## Bloom

Comfy `v0.1.0` had bloom turned on by default. This turned out to be quite
problematic on older integrated GPUs as some users reported, as the builtin
bloom does 20 blur passes :)

In `v0.2.0` bloom is now turned off by default. You can still enable it by
calling `game_config_mut().bloom_enabled = true;`. There's also a [new
example](https://github.com/darthdeus/comfy/blob/master/comfy/examples/bloom.rs)
that showcases bloom and how it can be configured.

## Chromatic aberration

Comfy `v0.1.0` also had chromatic aberration enabled by default, but
considering this isn't even a documented feature and the API for using it is
quite ugly we turned it off for now in `v0.2.0`. I don't think there's any
chance anyone actually used it, but if you did, it'll come back soon I promise.

Post processing is one of the things that should improve after `v0.2.0` is out,
and we'll be able to add more effects and make them easier to use.

## Minor changes:

- `GameConfig` is no longer `Copy`. This shouldn't really affect anyone in
  any way, as it was behind a `RefCell` anyway.

## Next up

The global namespace is currently polluted by a lot of things. The next
`v0.3.0` release will focus on cleaning this up and making some things more
directly accessible (e.g. some globals which are now currently not public).
