{
  stdenvNoCC,
  names,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "outputs";
  src = source;
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  inherit names;
  passAsFile = [ "names" ];
  checkPhase = ''
    sort "$namesPath" > flake.txt
    grep --only-matching '^| `[a-zA-Z0-9_-]*` | [a-z]*' docs/OUTPUTS.md \
      | tr --delete '|`' \
      | awk '{ print $1, $2 }' \
      | sort > docs.txt
    if ! diff flake.txt docs.txt; then
      echo "[OUTPUTS] the flake and docs/OUTPUTS.md do not agree."
      echo "[OUTPUTS] a line marked < is an output nobody wrote down."
      echo "[OUTPUTS] a line marked > is a row naming an output that is gone."
      exit 1
    fi
  '';
  installPhase = ''
    touch $out
  '';
}
