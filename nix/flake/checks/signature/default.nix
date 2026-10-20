{
  stdenvNoCC,
  ripgrep,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "signature";
  src = source;
  nativeBuildInputs = [ ripgrep ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    : > casts.txt
    for source in $(find ./crates -name '*.rs' | sort); do
      awk '
        /^[[:space:]]*impl[[:space:]]+(From|TryFrom)</ { conversion = 1 }
        /^[[:space:]]*impl<[^>]*>[[:space:]]+(From|TryFrom)</ { conversion = 1 }
        conversion && /^}/ { conversion = 0 }
        !conversion && /[^A-Za-z0-9_]as[[:space:]]+(usize|isize|u8|u16|u32|u64|u128|i8|i16|i32|i64|i128|f32|f64)[^A-Za-z0-9_]/ {
          printf "%d:%s\n", NR, $0
        }
      ' "$source" | sed "s|^|$source:|" >> casts.txt
    done
    if [ -s casts.txt ]; then
      echo "[SIGNATURE] a numeric cast outside a From or a TryFrom."
      echo "[SIGNATURE] a conversion that can lose a value belongs in a From, where it is named."
      cat casts.txt
      exit 1
    fi

    resultFunctions=$(rg --line-number --no-heading --glob '*.rs' \
      '^\s*(pub(\([^)]*\))?\s+)?(async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*[(<].*->\s*(Result|io::Result|fmt::Result|Response)\b' ./crates \
      | rg --invert-match '\bfn\s+(try_[a-z0-9_]*|from|from_str|fmt|read|write|write_fmt|write_str|fill_buf|flush|serialize|deserialize|next_element_seed)\s*[(<]' || true)
    if [ -n "$resultFunctions" ]; then
      echo "[SIGNATURE] a function that returns a Result or a Response and does not state so in its name."
      echo "[SIGNATURE] name it try_ and the caller reads the failure before the body."
      echo "[SIGNATURE] a Response is a future of a Result, so the failure crosses the signature there as well."
      echo "[SIGNATURE] the exceptions are the methods of a trait somebody else named."
      printf '%s\n' "$resultFunctions"
      exit 1
    fi

    counts=$(rg --line-number --no-heading --glob '*.rs' \
      '^\s*(pub(\([^)]*\))?\s+)?fn\s+[a-z_][a-z0-9_]*s\s*\(\s*&self\s*\)\s*->\s*usize' ./crates \
      | rg --invert-match '\bfn\s+count[a-z0-9_]*\s*\(' || true)
    if [ -n "$counts" ]; then
      echo "[SIGNATURE] a getter named for many things that gives back one number."
      echo "[SIGNATURE] a count is named count, and the plural is the getter for the things."
      echo "[SIGNATURE] a name that already begins with count states which it is, so it is no such getter."
      printf '%s\n' "$counts"
      exit 1
    fi

    suffixedNumbers=$(rg --line-number --no-heading --glob '*.rs' \
      '\b(0[xXbBoO][0-9a-fA-F_]+|[0-9][0-9_]*)(usize|isize|u8|u16|u32|u64|u128|i8|i16|i32|i64|i128)\b|\b[0-9][0-9_]*(f32|f64)\b' ./crates || true)
    if [ -n "$suffixedNumbers" ]; then
      echo "[SIGNATURE] a number with the name of a Rust type stuck to it."
      echo "[SIGNATURE] nothing needs one. where the compiler cannot infer, that type is wrong."
      printf '%s\n' "$suffixedNumbers"
      exit 1
    fi

    separatedNumbers=$(rg --line-number --no-heading --glob '*.rs' \
      '\b(0[xXbBoO])?[0-9][0-9a-fA-F]*_' ./crates || true)
    if [ -n "$separatedNumbers" ]; then
      echo "[SIGNATURE] a number broken up by underscores."
      echo "[SIGNATURE] a number is one number, so write it as one."
      printf '%s\n' "$separatedNumbers"
      exit 1
    fi

    byteFunctions=$(rg --line-number --no-heading --glob '*.rs' \
      '^\s*(pub(\([^)]*\))?\s+)?(async\s+)?fn\s+[a-z_][a-z0-9_]*\s*[(<].*->\s*(Vec<u8>|&\[u8\]|\[u8;)' ./crates || true)
    if [ -n "$byteFunctions" ]; then
      echo "[SIGNATURE] a function that answers the bare bytes of Rust."
      echo "[SIGNATURE] bytes whose count a file settles are a Blob, and bytes of a settled count are Bytes."
      echo "[SIGNATURE] the two modules that make those are the only ones that may answer the bare thing."
      printf '%s\n' "$byteFunctions"
      exit 1
    fi

    : > prepositions.txt
    for source in $(find ./crates -name '*.rs' | sort); do
      awk '
        /#\[test\]/ { test = 1; next }
        /fn[[:space:]]+[a-z_]/ {
          if (!test && match($0, /fn[[:space:]]+(of_[a-z0-9_]*|[a-z0-9_]*_of)[[:space:]]*[(<]/)) {
            printf "%d:%s\n", NR, substr($0, RSTART, RLENGTH)
          }
          test = 0
        }
      ' "$source" | sed "s|^|$source:|" >> prepositions.txt
    done
    if [ -s prepositions.txt ]; then
      echo "[SIGNATURE] a function named with the preposition of."
      echo "[SIGNATURE] the of in a name covers either a From that nobody wrote as one,"
      echo "[SIGNATURE] or an unwrap that a caller can no longer see."
      echo "[SIGNATURE] name it for what it answers, or write the conversion the name is pretending to be."
      echo "[SIGNATURE] a test name is a sentence and may state of, which is why only the rest are read here."
      cat prepositions.txt
      exit 1
    fi
  '';
  installPhase = ''
    touch $out
  '';
}
