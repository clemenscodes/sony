{
  ripgrep,
  stdenvNoCC,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "test";
  src = source;
  nativeBuildInputs = [ ripgrep ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    refusals=0

    echo "[TEST] this gate reads $(rg --files --glob '*.rs' ./crates | wc --lines) sources."

    while IFS= read -r source_file; do
      line=$( (rg --line-number '^mod tests \{$' "$source_file" || true) | head --lines 1 | cut --delimiter=: --fields=1)
      if [ -z "$line" ]; then
        continue
      fi
      module=$(tail --lines "+$line" "$source_file")
      functions=$(printf '%s\n' "$module" | (rg --count '^    fn ' || true))
      tests=$(printf '%s\n' "$module" | (rg --count '^    #\[test\]$' || true))
      if [ "''${functions:-0}" != "''${tests:-0}" ]; then
        surplus=$(( ''${functions:-0} - ''${tests:-0} ))
        word=functions
        if [ "$surplus" = "1" ]; then
          word=function
        fi
        echo "[TEST] $source_file keeps $surplus $word beside its tests."
        refusals=1
      fi
    done < <(rg --files --glob '*.rs' ./crates)

    if [ "$refusals" != 0 ]; then
      echo "[TEST] whatever a test needs, that test writes, because a helper hides"
      echo "[TEST] the arrangement the test exists to show."
      exit 1
    fi

    decisions=$(rg --line-number --no-heading --glob 'src/main.rs' \
      '(^|[^a-zA-Z])(if|else|match|while|for|unwrap_or|\?;)([^a-zA-Z]|$)' ./crates || true)
    if [ -n "$decisions" ]; then
      echo "[TEST] a decision inside a main. No test can call a main, so nothing decides there."
      echo "[TEST] a main parses the command line, delegates, reports and sets the status."
      printf '%s\n' "$decisions"
      exit 1
    fi

    echo "[TEST] every test module was read, and none of them keeps a function beside its tests."
  '';
  installPhase = ''
    touch $out
  '';
}
