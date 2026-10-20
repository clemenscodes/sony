{
  callPackage,
  craneLib,
  host,
  lib,
  packages,
  shells,
  self,
  rustToolchain,
}:
let
  inherit (lib)
    any
    attrNames
    concatStringsSep
    hasPrefix
    hasSuffix
    removePrefix
    ;
  inherit (lib.sources) cleanSourceWith;
  sourceTree =
    name: paths:
    cleanSourceWith {
      inherit name;
      src = self;
      filter =
        path: _type:
        let
          relativePath = removePrefix "${self}/" (toString path);
        in
        any (prefix: hasPrefix prefix relativePath || hasPrefix relativePath prefix) paths;
    };
  crateSource =
    crate:
    cleanSourceWith {
      inherit (crate) name;
      src = self;
      filter =
        path: type:
        let
          relativePath = removePrefix "${self}/" (toString path);
        in
        any (prefix: hasPrefix prefix relativePath || hasPrefix relativePath prefix) crate.paths
        || (hasPrefix "crates" relativePath && (type == "directory" || baseNameOf path == "Cargo.toml"));
    };
  nix = cleanSourceWith {
    name = "nix";
    src = self;
    filter = path: type: type == "directory" || hasSuffix ".nix" path;
  };
  gates = {
    statix = callPackage ./statix { source = nix; };

    deadnix = callPackage ./deadnix { source = nix; };

    format = callPackage ./format { source = nix; };

    rustfmt =
      let
        source = cleanSourceWith {
          name = "rust";
          src = self;
          filter =
            path: type:
            let
              fileName = baseNameOf path;
            in
            type == "directory" || fileName == "Cargo.toml" || hasSuffix ".rs" fileName;
        };
      in
      callPackage ./rustfmt {
        inherit source rustToolchain;
      };

    comment =
      let
        source = sourceTree "comment" [ "crates" ];
      in
      callPackage ./comment {
        inherit source;
      };

    word =
      let
        source = sourceTree "word" [
          "crates"
          "docs"
          "nix"
          "README.md"
        ];
      in
      callPackage ./word {
        inherit source;
      };

    interface =
      let
        source = sourceTree "crates" [ "crates" ];
      in
      callPackage ./interface {
        inherit source;
      };

    test =
      let
        source = sourceTree "crates" [ "crates" ];
      in
      callPackage ./test {
        inherit source;
      };

    signature =
      let
        source = sourceTree "crates" [ "crates" ];
      in
      callPackage ./signature {
        inherit source;
      };

    book =
      let
        source = cleanSourceWith {
          name = "books";
          src = self;
          filter =
            path: type:
            let
              fileName = baseNameOf path;
            in
            type == "directory" || fileName == "book.toml" || hasSuffix ".md" fileName;
        };
      in
      callPackage ./book {
        inherit source;
      };

    sony =
      let
        sony = import ../../../crates/sony/nix;
        source = crateSource sony;
      in
      callPackage ./sony {
        inherit craneLib;
        inherit (sony) name;
        inherit source;
      };

    coverage-sony =
      let
        sony = import ../../../crates/sony/nix;
        source = crateSource sony;
      in
      callPackage ./coverage {
        inherit craneLib host rustToolchain;
        inherit (sony) name;
        inherit source;
      };
  };
  checkNames =
    let
      lines = attrNames gates ++ [
        "outputs"
        "pipeline"
      ];
    in
    concatStringsSep "\n" lines + "\n";
  outputNames =
    let
      lines =
        map (name: "${name} package") (attrNames packages)
        ++ map (name: "${name} check") (
          attrNames gates
          ++ [
            "outputs"
            "pipeline"
          ]
        )
        ++ map (name: "${name} shell") shells;
    in
    concatStringsSep "\n" lines + "\n";
in
gates
// {
  outputs =
    let
      source = sourceTree "outputs" [ "docs/OUTPUTS.md" ];
    in
    callPackage ./outputs {
      names = outputNames;
      inherit source;
    };
  pipeline =
    let
      source = cleanSourceWith {
        name = "moon";
        src = self;
        filter =
          path: type:
          let
            relativePath = removePrefix "${self}/" (toString path);
          in
          type == "directory"
          || baseNameOf path == "moon.yml"
          || hasPrefix ".moon/tasks" relativePath
          || relativePath == "nix/flake/checks/default.nix";
      };
    in
    callPackage ./pipeline {
      names = checkNames;
      inherit source;
    };
}
