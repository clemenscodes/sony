{
  stdenvNoCC,
  rustToolchain,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "rustfmt";
  src = source;
  nativeBuildInputs = [ rustToolchain ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    cargo fmt --all --check
  '';
  installPhase = ''
    touch $out
  '';
}
