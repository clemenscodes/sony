{
  stdenvNoCC,
  statix,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "statix";
  src = source;
  nativeBuildInputs = [ statix ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    statix check .
  '';
  installPhase = ''
    touch $out
  '';
}
