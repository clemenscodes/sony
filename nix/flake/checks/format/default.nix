{
  stdenvNoCC,
  nixfmt-tree,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "format";
  src = source;
  nativeBuildInputs = [ nixfmt-tree ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    treefmt --ci --tree-root .
  '';
  installPhase = ''
    touch $out
  '';
}
