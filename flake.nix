{
  description = "";

  inputs.nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0";

  outputs =
    { self, ... }@inputs:
    let
      supportedSystems = [ "x86_64-linux" ];

      forEachSupportedSystem =
        fn:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          fn {
            inherit system;
            pkgs = import inputs.nixpkgs {
              inherit system;
              config.allowUnfree = true;
            };
          }
        );
    in
    {

      devShells = forEachSupportedSystem (
        { pkgs, system }:
        {
          default = pkgs.mkShellNoCC {
            packages = [
              self.formatter.${system}
            ];

            env = { };
            shellHook = "";
          };
        }
      );

      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt-tree);
    };
}
