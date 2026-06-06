{
  description = "";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { self, ... }@inputs:
    let
      inherit (inputs.nixpkgs) lib;

      supportedSystems = [ "x86_64-linux" ];

      forEachSupportedSystem =
        fn:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          fn {
            inherit system;
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [ (import inputs.rust-overlay) ];
              config.allowUnfree = true;
            };
          }
        );
    in
    {

      devShells = forEachSupportedSystem (
        { pkgs, system }:
        {
          default = pkgs.mkShell rec {
            buildInputs = with pkgs; [
              (rust-bin.stable.latest.default.override {
                extensions = [ "rust-analyzer" ];
                targets = [ "wasm32-unknown-unknown" ];
              })

              udev
              alsa-lib
              vulkan-loader
              libGL
              gtk3
              libx11
              libxcursor
              libxi
              libxrandr
              libxkbcommon
              wayland
              pkg-config
              openssl.dev
              fontconfig
              nasm
              perl
              cmake
              trunk
              whois

              (python3.withPackages (python-pkgs: with python-pkgs; [
                argparse
                dnspython
                requests
              ]))

              self.formatter.${system}
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

            env = { };
            shellHook = "";
          };
        }
      );

      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt-tree);
    };
}
