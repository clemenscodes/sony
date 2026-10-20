{
  stdenvNoCC,
  names,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "pipeline";
  src = source;
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  inherit names;
  passAsFile = [ "names" ];
  checkPhase = ''
    : > projects.txt
    for file in moon.yml $(find crates -name moon.yml | sort); do
      id=$(sed --quiet "s/^id: '\(.*\)'\$/\1/p" "$file")
      language=$(sed --quiet "s/^language: '\(.*\)'\$/\1/p" "$file")
      tags=$(awk '/^tags:$/ { section = 1; next } /^[^ ]/ { section = 0 } section' "$file" \
        | sed --quiet "s/^  - '\(.*\)'\$/\1/p" | tr '\n' ',')
      printf '%s\t%s\t%s\t%s\n' "$id" "$language" "$tags" "$file" >> projects.txt
    done

    checks() {
      grep --only-matching "checks\.x86_64-linux\.[^']*" "$1" \
        | sed 's|checks\.x86_64-linux\.||' || true
    }

    exclusions() {
      awk '/^workspace:$/ { section = 1; next } section && /^[^ ]/ { section = 0 } section' "$1" \
        | sed --quiet "s/^      - '\(.*\)'\$/\1/p" || true
    }

    inheritance() {
      awk -v task="  $2:" '$0 == task { section = 1; next } section && /^  [a-z]/ { section = 0 } section' "$1" \
        | grep --only-matching "checks\.x86_64-linux\.[^']*" \
        | sed 's|checks\.x86_64-linux\.||' || true
    }

    : > candidates.txt

    checks .moon/tasks/rust.yml > pattern.txt
    while IFS="$(printf '\t')" read -r id language tags file; do
      test "$language" = "rust" || continue
      sed "s|\$project|$id|" pattern.txt >> candidates.txt
    done < projects.txt

    while IFS="$(printf '\t')" read -r id language tags file; do
      checks "$file" | sed "s|\$project|$id|" >> candidates.txt
    done < projects.txt

    : > exclusions.txt
    while IFS="$(printf '\t')" read -r id language tags file; do
      for task in $(exclusions "$file"); do
        inheritance .moon/tasks/rust.yml "$task" | sed "s|\$project|$id|" >> exclusions.txt
      done
    done < projects.txt

    sort --unique candidates.txt --output candidates.txt
    sort --unique exclusions.txt --output exclusions.txt
    comm -23 candidates.txt exclusions.txt > moonChecks.txt
    sort "$namesPath" > flakeChecks.txt

    if ! diff flakeChecks.txt moonChecks.txt > difference.txt; then
      echo "[PIPELINE] the flake and moon do not run the same checks."
      cat difference.txt
      echo "[PIPELINE] a line marked < is a check of the flake that no moon task builds."
      echo "[PIPELINE] a line marked > is a moon task naming a check the flake does not have."
      exit 1
    fi

    tab=$(printf '\t')

    : > inputs.txt
    while IFS="$tab" read -r id language tags file; do
      awk '
        /^  [a-zA-Z][a-zA-Z0-9\/._-]*:$/ { check = "" }
        /checks\.x86_64-linux\./ {
          if (match($0, /checks\.x86_64-linux\.[a-zA-Z0-9-]+/)) {
            check = substr($0, RSTART + 20, RLENGTH - 20)
          }
        }
        /^      - / {
          entry = $0
          gsub(/^      - .|.$/, "", entry)
          sub(/^!/, "", entry)
          sub(/^\//, "", entry)
          sub(/[\/*].*$/, "", entry)
          if (check != "" && entry != "") printf "%s\t%s\n", check, entry
        }
      ' "$file" >> inputs.txt
    done < projects.txt
    sort --unique inputs.txt --output inputs.txt
    cut --fields 1 inputs.txt | sort --unique > tasks.txt

    awk '
      /^ *[a-z][a-zA-Z0-9-]* =$/ { name = $1; next }
      name != "" && /sourceTree/ {
        line = $0
        sub(/.*sourceTree[[:space:]]*"[^"]*"/, "", line)
        remainder = line
        while (match(remainder, /"[^"]*"/)) {
          printf "%s\t%s\n", name, substr(remainder, RSTART + 1, RLENGTH - 2)
          remainder = substr(remainder, RSTART + RLENGTH)
        }
        if (line !~ /\]/) list = 1
        next
      }
      list {
        remainder = $0
        while (match(remainder, /"[^"]*"/)) {
          printf "%s\t%s\n", name, substr(remainder, RSTART + 1, RLENGTH - 2)
          remainder = substr(remainder, RSTART + RLENGTH)
        }
        if ($0 ~ /\]/) { list = 0; name = "" }
      }
    ' nix/flake/checks/default.nix | sort --unique > sources.txt

    : > findings.txt
    sources=0
    while IFS="$tab" read -r check path; do
      grep --quiet --line-regexp --fixed-strings "$check" tasks.txt || continue
      sources=$((sources + 1))
      root=''${path%%/*}
      grep --quiet --line-regexp --fixed-strings "$check$tab$root" inputs.txt && continue
      printf '%s reads %s and its task lists no %s\n' "$check" "$path" "$root" >> findings.txt
    done < sources.txt

    if [ -s findings.txt ]; then
      echo "[PIPELINE] a gate reads what its moon task does not list, so moon answers out of a"
      echo "[PIPELINE] cache that never saw the change. Widen the task in the same commit."
      cat findings.txt
      exit 1
    fi
    echo "[PIPELINE] $sources sources of the gates, and every one of them is a task input."
  '';
  installPhase = ''
    touch $out
  '';
}
