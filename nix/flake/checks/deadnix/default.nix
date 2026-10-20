{
  stdenvNoCC,
  deadnix,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "deadnix";
  src = source;
  nativeBuildInputs = [ deadnix ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    deadnix --fail .
  '';
  installPhase = ''
    touch $out
  '';
}
