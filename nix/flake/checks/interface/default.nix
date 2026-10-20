{
  stdenvNoCC,
  ripgrep,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "interface";
  src = source;
  nativeBuildInputs = [ ripgrep ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    refusals=0

    globs=$(rg --line-number --no-heading --glob '*.rs' \
      '^\s*(pub(\([^)]*\))?\s+)?use\s+[^;]*::\*\s*;' . \
      | rg --invert-match '^\S+:[0-9]+:\s*use super::\*;$' || true)
    if [ -n "$globs" ]; then
      echo "[INTERFACE] a glob import flattens somebody else's module into this one."
      printf '%s\n' "$globs"
      refusals=1
    fi

    exports=$(rg --line-number --no-heading --glob '*.rs' \
      '^\s*pub(\([^)]*\))?\s+use\s' . || true)

    while IFS= read -r line; do
      [ -n "$line" ] || continue
      file=$(printf '%s' "$line" | cut --delimiter=: --fields=1)
      text=$(printf '%s' "$line" | cut --delimiter=: --fields=3-)
      path=$(printf '%s' "$text" | sed 's/.*[[:space:]]use[[:space:]]\+//; s/[[:space:]]*;.*$//')
      depth=$(printf '%s' "$path" | grep --only-matching '::' | wc --lines)
      if [ "$depth" != "1" ]; then
        echo "[INTERFACE] $file reaches past its own child, $path."
        refusals=1
        continue
      fi
      if [ "$(basename "$file")" = "lib.rs" ]; then
        continue
      fi
      child=''${path%%::*}
      items=''${path##*::}
      items=$(printf '%s' "$items" | tr --delete '{}' | tr ',' ' ')
      module="$(dirname "$file")/$child/mod.rs"
      [ -f "$module" ] || continue
      for item in $items; do
        lines=$(rg --line-number --no-heading \
          "^\s*pub(\([^)]*\))?\s+use\s+[^;]*(::$item\s*;|[{,]\s*$item\s*[,}])" "$module" || true)
        if [ -n "$lines" ]; then
          echo "[INTERFACE] $file publishes $item, which $module already published."
          refusals=1
        fi
      done
    done <<< "$exports"

    if [ "$refusals" != "0" ]; then
      echo "[INTERFACE] a layer publishes its own children once, and nothing publishes twice."
      exit 1
    fi
  '';
  installPhase = ''
    touch $out
  '';
}
