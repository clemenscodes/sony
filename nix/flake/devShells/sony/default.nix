{
  cargo-llvm-cov,
  cargo-nextest,
  rustToolchain,
  deadnix,
  git,
  host,
  jq,
  mdbook,
  mkShellNoCC,
  moon,
  nil,
  nixfmt-tree,
  ripgrep,
  statix,
}:
mkShellNoCC {
  name = "sony";

  packages = [
    cargo-llvm-cov
    cargo-nextest
    deadnix
    git
    jq
    mdbook
    moon
    nil
    nixfmt-tree
    ripgrep
    rustToolchain
    statix
  ];

  CARGO_TERM_COLOR = "always";

  CARGO_NET_GIT_FETCH_WITH_CLI = "true";

  RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

  LLVM_COV = "${rustToolchain}/lib/rustlib/${host}/bin/llvm-cov";

  LLVM_PROFDATA = "${rustToolchain}/lib/rustlib/${host}/bin/llvm-profdata";

  shellHook = ''
    if [ -d .moon ]; then
      moon sync hooks >/dev/null
      moon sync config-schemas >/dev/null
      moon sync projects >/dev/null
    fi
  '';
}
