# EXAMPLE=animated_shapes
# EXAMPLE=animated_text
# EXAMPLE=animated_sprites
# EXAMPLE=alpha_sprite
# EXAMPLE=blood_canvas
# EXAMPLE=bloom
# EXAMPLE=custom_config
# EXAMPLE=cooldowns
# EXAMPLE=custom_fonts
# EXAMPLE=circle
# EXAMPLE=colors
# EXAMPLE=color-bars
# EXAMPLE=ecs_sprite
# EXAMPLE=ecs_topdown_game
# EXAMPLE=egui
# EXAMPLE=exr-hdr-image
# EXAMPLE=full_game_loop
# EXAMPLE=framerate_vsync
# EXAMPLE=fragment-shader
# EXAMPLE=music
# EXAMPLE=ldtk
# EXAMPLE=lighting
# EXAMPLE=single_particle
# EXAMPLE=particle_systems
# EXAMPLE=perspective-camera
# EXAMPLE=physics
# EXAMPLE=post_processing
# EXAMPLE=render-target
# EXAMPLE=sprite
# EXAMPLE=shapes
# EXAMPLE=sound
# EXAMPLE=text
# EXAMPLE=timed_draw
# EXAMPLE=version
# EXAMPLE=y_sort
# EXAMPLE=z_index_test

# default: build-examples
# default: wasm-build
# default: profile-startup
# default: bitmob
# default: crash
# default: benchmarks
default: example
# default: fun
# default: example-wasm
# default: wasm-egui-scaling
# default: egui-demo
# default: lint
# default: test

# FLAGS=--features=blobs,git-version,dev,ldtk,exr
FLAGS=--features=git-version,dev,tracy
ENV_VARS=RUST_LOG=info,wgpu=warn,symphonia=warn,naga=warn RUST_BACKTRACE=1 COMFY_DEV_TITLE=1 COMFY_VSYNC_OVERRIDE=0

# Crashes on i3 without COMFY_DEV_TITLE=1
i3-crash:
	cargo run --example alpha_sprite $(FLAGS)

bitmob:
	$(ENV_VARS) cargo run --bin bitmob $(FLAGS)

example:
	$(ENV_VARS) cargo run --example $(EXAMPLE) $(FLAGS)

fun:
	$(ENV_VARS) cargo run --bin fun $(FLAGS)

example-wasm:
	$(ENV_VARS) cargo run --example $(EXAMPLE) $(FLAGS) --target wasm32-unknown-unknown

egui-demo:
	$(ENV_VARS) cargo run --bin egui-scaling

benchmarks:
	$(ENV_VARS) make -C ../comfy-benchmark

profile-startup:
	cargo run --example shapes --features exit-after-startup

wasm-egui-scaling:
	cargo run --target wasm32-unknown-unknown --bin egui-scaling

build-examples:
	./build-examples.sh

lint:
	cargo fmt --all -- --check
	cargo clippy

timings:
	cargo clean
	cargo build --timings --example sprite
	# RUSTFLAGS="-Z threads=8" cargo build --timings --example sprite

serve:
	simple-http-server target/generated -c wasm,html,js -i

publish-crates:
	cargo publish -p comfy-core
	cargo publish -p comfy-wgpu
	cargo publish -p comfy

test:
	cargo fmt --check
	cargo clippy
	cargo test --all --features=blobs
	./build-examples.sh

duplicates:
	simian **/*.rs
