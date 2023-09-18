EXAMPLE=animated_shapes
# EXAMPLE=sprite
# EXAMPLE=text
# EXAMPLE=particles
# EXAMPLE=post_processing
# EXAMPLE=shapes

# default: build-examples
# default: wasm-build
default: example
# default: profile-startup

FLAGS=--release

example:
	RUST_LOG=info,wgpu=warn,symphonia=warn cargo run --example $(EXAMPLE) $(FLAGS)

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
