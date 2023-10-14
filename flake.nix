# confirmed to work on nixos with wayland (sway)
# use with `nix develop`
# then run `cargo run --example music -F winit/wayland`
{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };
  outputs = { nixpkgs, fenix, ... }:
    let
      forAllSystems = function:
        # insert more systems here
        nixpkgs.lib.genAttrs [ "x86_64-linux" ] (system:
          function (import nixpkgs {
            inherit system;
            overlays = [ fenix.overlays.default ];
          }));

    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            # rust stuff
            (pkgs.fenix.complete.withComponents [
              "cargo"
              "clippy"
              "rust-src"
              "rustc"
              "rustfmt"
            ])
            clang
            mold
            # rust-analyzer-nightly # optional

            # necessary to build
            pkg-config # locate C dependencies
            alsaLib # sound
            libxkbcommon # keyboard
            # wayland

            vulkan-tools
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers

            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi

            # extra tooling
            tracy # profiler, call with ~Tracy~
          ];
          # stuff we need to run
          LD_LIBRARY_PATH = with pkgs;
            lib.makeLibraryPath [
              libxkbcommon # keyboard
              # wayland
              libGL # OpenGL I think
              alsaLib # sound
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
            ];
        };
      });
    };
}
