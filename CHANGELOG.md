# v0.2.0

The main change in this release is that `EngineContext` does not exist
anymore. This should simplify a lot of the confusion, as the #1 question
about Comfy from _many_ people was about `GameContext` and `EngineContext`
and why are there two any why do we even need them.

Since Comfy already uses globals for many things, it makes sense to just
embrace this fully and move the remaining things to globals as well. Many
of the values provided in `EngineContext` were already available through
globals and just re-exported, so this shouldn't be a huge issue.

Comfy will still use `EngineContext` internally for some things, but this
won't be re-exported to the users as everything should be accessible by
other means.

List of removed things and where to find them now:

- `c.delta` -> `delta()`. This is likely going to be something that most users (including US)
  re-export into their `GameContext/GameState` anyway.
- `c.world()` -> `world()`. ECS world already lived in a single instance, it's now moved into a single global.
- `c.commands()` -> `commands()`. Same as above.
- `c.cooldowns()` -> `cooldowns()`. This might be worth re-exporting into
  `GameContext` if accessed frequently, but in either way there's no extra
  overhead compared to before.
- `c.egui` -> `egui()`. Note that before this was a field, now it's a
  global function. Though in this case `egui::Context` is already
  internally `Arc<Mutex<ContextImpl>>`, so this function is actually very
  cheap to call as it just returns a `&'static egui::Context` :)
- `c.egui_wants_mouse` -> `egui().wants_pointer_input()`

NOTE: Comfy still includes many APIs which are not currently documented but are
still exposed. The current goal is to work through codebase and cleanup some odd bits
and document them at the same time. If you find something that is not mentioned
on the website or any examples, it's very likely to change in the future.
