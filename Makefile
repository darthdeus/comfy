
# default: example-shapes
default: profile-startup

example-shapes:
	cargo run --example shapes

tests:
	cargo test

profile-startup:
	cargo run --example shapes --features exit-after-startup

	# make -C ~/projects/demos
