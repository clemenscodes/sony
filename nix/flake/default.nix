{ inputs, ... }:
{
  systems = [ "x86_64-linux" ];

  perSystem =
    { system, ... }:
    let
      pkgs = import inputs.nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
      };
      inherit (pkgs) callPackages;
      inherit (inputs) self;

      host = "x86_64-unknown-linux-gnu";

      rustToolchain = pkgs.callPackage ./packages/rustToolchain { pinned = ../../rust-toolchain.toml; };
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchain;

      formatter = pkgs.nixfmt-tree;

      packages = callPackages ./packages {
        inherit rustToolchain;
      };

      devShells = callPackages ./devShells {
        inherit
          host
          rustToolchain
          ;
      };

      checks = callPackages ./checks {
        inherit
          craneLib
          host
          packages
          self
          rustToolchain
          ;
        shells = builtins.attrNames devShells;
      };
    in
    {
      _module.args = { inherit pkgs; };

      inherit
        formatter
        devShells
        packages
        checks
        ;
    };
}
