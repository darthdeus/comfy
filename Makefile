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
# EXAMPLE=ecs_sprite
# EXAMPLE=ecs_topdown_game
# EXAMPLE=full_game_loop
# EXAMPLE=framerate_vsync
# EXAMPLE=fragment-shader
# EXAMPLE=music
# EXAMPLE=lighting
# EXAMPLE=single_particle
# EXAMPLE=particle_systems
# EXAMPLE=perspective-camera
# EXAMPLE=physics
# EXAMPLE=post_processing
# EXAMPLE=render-target
EXAMPLE=sprite
# EXAMPLE=shapes
# EXAMPLE=sound
# EXAMPLE=text
# EXAMPLE=timed_draw
# EXAMPLE=version
# EXAMPLE=y_sort

# default: build-examples
# default: wasm-build
# default: profile-startup
# default: bitmob
default: example
# default: lint
# default: test

FLAGS=--features=blobs,git-version,dev
ENV_VARS=RUST_LOG=info,wgpu=warn,symphonia=warn,naga=warn RUST_BACKTRACE=1

bitmob:
	$(ENV_VARS) cargo run --bin bitmob $(FLAGS)

example:
	$(ENV_VARS) cargo run --example $(EXAMPLE) $(FLAGS)

profile-startup:
	cargo run --example shapes --features exit-after-startup

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
