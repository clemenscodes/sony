{
  stdenvNoCC,
  ripgrep,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "comment";
  src = source;
  nativeBuildInputs = [ ripgrep ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    comments=$(rg --line-number --no-heading --glob '*.rs' '^\s*(//|///|//!)' . || true)
    if [ -n "$comments" ]; then
      echo "[COMMENT] a crate contains no comment."
      echo "[COMMENT] a name states what a thing is, a type constrains it, a test proves it."
      echo "[COMMENT] where none of those states the meaning, the design is what changes."
      printf '%s\n' "$comments"
      exit 1
    fi
  '';
  installPhase = ''
    touch $out
  '';
}
