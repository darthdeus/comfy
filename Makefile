# EXAMPLE=animated_shapes
EXAMPLE=sprite

default: example
# default: profile-startup

example:
	cargo run --example $(EXAMPLE)

tests:
	cargo test

profile-startup:
	cargo run --example shapes --features exit-after-startup

	# make -C ~/projects/demos
