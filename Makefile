# EXAMPLE=animated_shapes
EXAMPLE=sprite
# EXAMPLE=text
# EXAMPLE=particles
# EXAMPLE=post_processing
# EXAMPLE=shapes

# default: wasm-build
# default: build-examples
default: example
# default: profile-startup

example:
	cargo run --example $(EXAMPLE)

tests:
	cargo test

profile-startup:
	cargo run --example shapes --features exit-after-startup

build-examples:
	./build-examples.sh

	# make -C ~/projects/demos

wasm-build:
	./build.bash web-bin release sprite

serve:
	./build.bash serve
