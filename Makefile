# EXAMPLE=animated_shapes
# EXAMPLE=animated_text
# EXAMPLE=custom_fonts
# EXAMPLE=ecs_sprite
# EXAMPLE=ecs_topdown_game
# EXAMPLE=full_game_loop
# EXAMPLE=music
# EXAMPLE=lighting
# EXAMPLE=single_particle
# EXAMPLE=particle_systems
EXAMPLE=physics
# EXAMPLE=post_processing
# EXAMPLE=sprite
# EXAMPLE=shapes
# EXAMPLE=sound
# EXAMPLE=text

# default: build-examples
# default: wasm-build
# default: profile-startup
# default: bitmob
default: example

FLAGS=--release --features=blobs
ENV_VARS=RUST_LOG=info,wgpu=warn,symphonia=warn,naga=warn

bitmob:
	$(ENV_VARS) cargo run --bin bitmob $(FLAGS)

example:
	$(ENV_VARS) cargo run --example $(EXAMPLE) $(FLAGS)

tests:
	cargo test

profile-startup:
	cargo run --example shapes --features exit-after-startup

build-examples:
	./build-examples.sh

wasm-build:
	./build-examples.sh

serve:
	simple-http-server target/generated -c wasm,html,js -i
