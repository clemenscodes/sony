{
  stdenvNoCC,
  ripgrep,
  source,
}:
stdenvNoCC.mkDerivation {
  name = "word";
  src = source;
  nativeBuildInputs = [ ripgrep ];
  dontConfigure = true;
  dontBuild = true;
  doCheck = true;
  checkPhase = ''
    refusals=0

    verbLines=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])(carr(y|ies|ied|ying)|stands?|standing|stood|lies|lie|lying|holds?|holding|held|sits?|sitting|sat|says?|said|saying|speaks?|spoke|spoken|speaking|aims?|aimed|aiming)([^a-zA-Z]|$)' . || true)
    if [ -n "$verbLines" ]; then
      echo "[WORD] one of the refused verbs. State where a thing is and what it does."
      echo "[WORD] nothing here speaks. A program requests and responds."
      echo "[WORD] a name states a meaning, a type constrains a value, a test proves a behaviour."
      echo "[WORD] a pointer points. It never aims."
      printf '%s\n' "$verbLines"
      refusals=1
    fi

    noughts=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])noughts?([^a-zA-Z]|$)' . || true)
    if [ -n "$noughts" ]; then
      echo "[WORD] nought is no word of this repository. Write zero."
      printf '%s\n' "$noughts"
      refusals=1
    fi

    corpus=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])corpus([^a-zA-Z]|$)' . || true)
    if [ -n "$corpus" ]; then
      echo "[WORD] corpus is no word of this repository. Most sentences are better without it."
      echo "[WORD] where all of the material needs a name, it is the assets or the disc."
      printf '%s\n' "$corpus"
      refusals=1
    fi

    hexadecimalLines=$(rg --hidden --line-number --no-heading \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '0x[0-9A-F]*[a-f][0-9a-fA-F]*' . || true)
    if [ -n "$hexadecimalLines" ]; then
      echo "[WORD] hexadecimal this repository forms is written in upper case."
      printf '%s\n' "$hexadecimalLines"
      refusals=1
    fi

    addressLines=$(rg --hidden --line-number --no-heading --pcre2 --glob '*.md' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      --glob '!crates/sony/docs/RUNNING.md' \
      '(^|[^0-9a-zA-Z_])(?=[0-9a-f]{8}([^0-9a-zA-Z_…]|$))[0-9a-f]*[a-f]' . || true)
    if [ -n "$addressLines" ]; then
      echo "[WORD] an address this repository writes down is written in upper case."
      printf '%s\n' "$addressLines"
      refusals=1
    fi

    shape=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])shapes?d?([^a-zA-Z]|$)|(^|[^a-zA-Z])shaping([^a-zA-Z]|$)' . || true)
    if [ -n "$shape" ]; then
      echo "[WORD] shape is no word of this repository. A thing laid out is a structure."
      printf '%s\n' "$shape"
      refusals=1
    fi

    placeLines=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])places?([^a-zA-Z]|$)' . || true)
    if [ -n "$placeLines" ]; then
      echo "[WORD] place is no word of this repository, nowhere and never."
      echo "[WORD] a value that says where a thing is names that thing. An offset, an address,"
      echo "[WORD] a position, a storage, a slot, a directory. A place names none of them."
      printf '%s\n' "$placeLines"
      refusals=1
    fi

    entiretyLines=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])whole([^a-zA-Z]|$)' . || true)
    if [ -n "$entiretyLines" ]; then
      echo "[WORD] whole is no word of this repository. Say the thing without it."
      echo "[WORD] one value, a number of Rust, and where all of a thing is meant, all of it."
      printf '%s\n' "$entiretyLines"
      refusals=1
    fi

    refusedWords="written|given|at|of|over|wrote|number|found|one|other|later|behind|rest"
    participles="taken|waiting|wanted|shown|kept|settled|made|describes|covers|mapped|readable"
    stopgaps="util|utils|helper|helpers|common|misc|shared"
    emptyWords="$refusedWords|$participles|$stopgaps"
    emptyWordNames=$(rg --hidden --line-number --no-heading \
      --glob '*.rs' \
      -e "^[[:space:]]*(let|const|static)[[:space:]]+(mut[[:space:]]+)?($emptyWords)\b" \
      -e "^[[:space:]]*($emptyWords)[[:space:]]*:" \
      -e "[(,][[:space:]]*($emptyWords)[[:space:]]*:" \
      -e "^[[:space:]]*([A-Za-z_][A-Za-z0-9_:<>*&]*[[:space:]]+){1,2}[*&[:space:]]*($emptyWords)[[:space:]]*[=;,)]" \
      . || true)
    if [ -n "$emptyWordNames" ]; then
      echo "[WORD] a binding, a field, an argument or a constant named with an empty word."
      echo "[WORD] docs/RUST_STYLE.md refuses written, given, at, of, over, wrote, number,"
      echo "[WORD] found, one, other, later, behind and rest outright."
      echo "[WORD] a value is a noun for what it is, never for what an action left behind,"
      echo "[WORD] so taken, waiting, wanted, shown, kept, settled and made name nothing."
      echo "[WORD] util, utils, helper, helpers, common, misc and shared mark a concept"
      echo "[WORD] that nobody has named yet."
      echo "[WORD] prose keeps the word, so a message and a test sentence are not read here."
      printf '%s\n' "$emptyWordNames"
      refusals=1
    fi

    runLines=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!**/RUNNING.md' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])(an?|the|one|every|each|its|their|no|unbroken|first|last)[\s_]+runs?([^a-zA-Z]|$)|(^|[^a-zA-Z])runs?[\s_]+of([^a-zA-Z]|$)' . || true)
    if [ -n "$runLines" ]; then
      echo "[WORD] run is a verb. A process runs and nothing else does."
      echo "[WORD] bytes are bytes, pages are pages, and neither of them is a run."
      printf '%s\n' "$runLines"
      refusals=1
    fi

    gerunds=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '^\s*let\s+(mut\s+)?[a-z_]*ing\s*[:=]' \
      --regexp '\|\s*(mut\s+)?[a-z_]*ing\s*[,|]' . \
      | rg --invert-match '(\blet\s+|\|\s*)(mut\s+)?([a-z_]*_)?(padding|packing|spelling|string|king|plumbing|tooling|ceiling|findings?)\s*[,:=|]' || true)
    if [ -n "$gerunds" ]; then
      echo "[WORD] a binding named for the act that made it. Say what the value is."
      printf '%s\n' "$gerunds"
      refusals=1
    fi

    acts=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '^\s*let\s+(mut\s+)?(read|walked|gathered|folded|trimmed|satisfied|finished|annotated|marked|padded|named|kept|opens|kill|resolve|cut|there|here)\s*[:=]' \
      --regexp '\|\s*(mut\s+)?(read|walked|gathered|folded|trimmed|satisfied|finished|annotated|marked|padded|named|kept|opens|kill|resolve|cut|there|here)\s*[,|]' . || true)
    if [ -n "$acts" ]; then
      echo "[WORD] a binding named for the act that made it. Say what the value is."
      printf '%s\n' "$acts"
      refusals=1
    fi

    actFields=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '(^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*)(mut\s+)?\b(read|walked|gathered|folded|trimmed|satisfied|finished|annotated|marked|padded|named|kept|opens|kill|resolve|cut|there|here)\s*:' . || true)
    if [ -n "$actFields" ]; then
      echo "[WORD] an argument or a field named for the act that made it. Say what the value is."
      printf '%s\n' "$actFields"
      refusals=1
    fi

    pastTenseNames=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '^\s*let\s+(mut\s+)?[a-z0-9_]*[a-z]ed\s*[:=]' \
      --regexp '\|\s*(mut\s+)?[a-z0-9_]*[a-z]ed\s*[,|]' \
      --regexp '(^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*)(mut\s+)?[a-z][a-z0-9_]*[a-z]ed\s*:' \
      --regexp '^\s*(pub(\([^)]*\))?\s+)?(const|static)\s+[A-Z][A-Z0-9_]*[A-Z]ED\s*:' ./crates \
      | rg --invert-match --ignore-case '(^|[^a-zA-Z])(shared|extended|seeds?)([^a-zA-Z]|$)' || true)
    if [ -n "$pastTenseNames" ]; then
      echo "[WORD] a name in the past tense. Say what the value is, not what was done to it."
      echo "[WORD] the act is a noun, a decryption. The thing is a decrypted_executable."
      echo "[WORD] a bare decrypted names nothing, because it never says what was decrypted."
      printf '%s\n' "$pastTenseNames"
      refusals=1
    fi

    typedLiterals=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '(^|[^a-zA-Z])(0[xXbBoO][0-9a-fA-F_]+|[0-9][0-9_]*)(u8|u16|u32|u64|u128|usize|i8|i16|i32|i64|i128|isize)([^a-zA-Z]|$)' . || true)
    if [ -n "$typedLiterals" ]; then
      echo "[WORD] a literal wearing its type. Name the value or put the type on the call."
      printf '%s\n' "$typedLiterals"
      refusals=1
    fi

    prepositions=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '^\s*let\s+(mut\s+)?([a-z0-9_]*_)?behind(_[a-z0-9_]*)?\s*[:=]' \
      --regexp '(^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*)(mut\s+)?([a-z0-9_]*_)?behind(_[a-z0-9_]*)?\s*:' \
      --regexp '^\s*(pub(\([^)]*\))?\s+)?(const|static)\s+([A-Z0-9_]*_)?BEHIND(_[A-Z0-9_]*)?\s*:' \
      --regexp '(^|[^a-zA-Z0-9_])(byte|short|int|long|float|double|char|boolean|var|[A-Z][A-Za-z0-9_]*(<[^<>]*>)?(\[\])?)\s+([a-z0-9]+_)?[bB]ehind([A-Z][A-Za-z0-9]*)?\s*[=;,)]' \
      . || true)
    if [ -n "$prepositions" ]; then
      echo "[WORD] behind is a preposition and names nothing. Say what the value is."
      echo "[WORD] an index that follows the digits of an address is an index, and it is"
      echo "[WORD] named for what it indexes. The rest of a text is no answer either,"
      echo "[WORD] because the rest of what is the question the name has to answer."
      printf '%s\n' "$prepositions"
      refusals=1
    fi

    bindings=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '^\s*let\s+(mut\s+)?([a-z0-9_]*_)?(written|given|at|of|over|wrote|numbers?)(_[a-z0-9_]*)?\s*[:=]' ./crates || true)
    if [ -n "$bindings" ]; then
      echo "[WORD] written, given, at, of, over, wrote and number name nothing. Say what the value is."
      echo "[WORD] a definition may say them, a binding may not."
      printf '%s\n' "$bindings"
      refusals=1
    fi

    fields=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '(^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*)(mut\s+)?([a-z0-9_]*_)?(written|given|at|of|over|wrote|numbers?)(_[a-z0-9_]*)?\s*:' ./crates || true)
    if [ -n "$fields" ]; then
      echo "[WORD] written, given, at, of, over, wrote and number name nothing. Say what the value is."
      echo "[WORD] a definition may say them, a field or an argument may not."
      printf '%s\n' "$fields"
      refusals=1
    fi

    constants=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '^\s*(pub(\([^)]*\))?\s+)?(const|static)\s+([A-Z0-9_]*_)?(WRITTEN|GIVEN|AT|OF|OVER|WROTE|NUMBERS?)(_[A-Z0-9_]*)?\s*:' ./crates || true)
    if [ -n "$constants" ]; then
      echo "[WORD] written, given, at, of, over, wrote and number name nothing. Say what the value is."
      echo "[WORD] a definition may say them, a constant may not."
      printf '%s\n' "$constants"
      refusals=1
    fi

    bare_values=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '(^\s*let\s+(mut\s+)?|^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*(mut\s+)?)(first|last|start|end|inside)\s*:' ./crates || true)
    if [ -n "$bare_values" ]; then
      echo "[WORD] first, last, start, end and inside name a position and never a thing."
      echo "[WORD] say first or last of what, start or end of what, inside of what."
      printf '%s\n' "$bare_values"
      refusals=1
    fi

    bare_bindings=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '^\s*let\s+(mut\s+)?(first|last|start|end|inside)\s*=' ./crates || true)
    if [ -n "$bare_bindings" ]; then
      echo "[WORD] first, last, start, end and inside name a position and never a thing."
      echo "[WORD] say first or last of what, start or end of what, inside of what."
      printf '%s\n' "$bare_bindings"
      refusals=1
    fi

    bare_methods=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '^\s*(pub(\([^)]*\))?\s+)?fn\s+(first|last|end|inside)\s*[(<]' ./crates || true)
    if [ -n "$bare_methods" ]; then
      echo "[WORD] first, last, end and inside name a position and never a thing."
      echo "[WORD] say first or last of what, end of what, inside of what."
      printf '%s\n' "$bare_methods"
      refusals=1
    fi

    bare_constants=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      '^\s*(pub(\([^)]*\))?\s+)?(const|static)\s+(FIRST|LAST|START|END|INSIDE)\s*:' ./crates || true)
    if [ -n "$bare_constants" ]; then
      echo "[WORD] first, last, start, end and inside name a position and never a thing."
      echo "[WORD] say first or last of what, start or end of what, inside of what."
      printf '%s\n' "$bare_constants"
      refusals=1
    fi

    empty_bindings=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '^\s*let\s+(mut\s+)?(found|refusal)\s*[:=]' \
      --regexp '^\s*let\s+(mut\s+)?([a-z0-9]+_)*(one|other|later)(_[a-z0-9_]*)?\s*[:=]' ./crates || true)
    if [ -n "$empty_bindings" ]; then
      echo "[WORD] found, one and other name nothing. Say what the value is."
      echo "[WORD] found says that a search ended. One and other say that there are two."
      echo "[WORD] neither says more inside a compound. No name has one of them in it."
      printf '%s\n' "$empty_bindings"
      refusals=1
    fi

    empty_fields=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '(^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*)(mut\s+)?(found|refusal)\s*:' \
      --regexp '(^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*)(mut\s+)?([a-z0-9]+_)*(one|other|later)(_[a-z0-9_]*)?\s*:' ./crates || true)
    if [ -n "$empty_fields" ]; then
      echo "[WORD] found, one and other name nothing. Say what the value is."
      echo "[WORD] a field and an argument answer the question what is this. None of them does."
      echo "[WORD] neither says more inside a compound. No name has one of them in it."
      printf '%s\n' "$empty_fields"
      refusals=1
    fi

    empty_constants=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '^\s*(pub(\([^)]*\))?\s+)?(const|static)\s+(FOUND|REFUSAL)\s*:' \
      --regexp '^\s*(pub(\([^)]*\))?\s+)?(const|static)\s+([A-Z0-9]+_)*(ONE|OTHER|LATER)(_[A-Z0-9_]*)?\s*:' ./crates || true)
    if [ -n "$empty_constants" ]; then
      echo "[WORD] found, one and other name nothing. Say what the value is."
      echo "[WORD] a constant is named for what it is. None of them says it."
      echo "[WORD] neither says more inside a compound. No name has one of them in it."
      printf '%s\n' "$empty_constants"
      refusals=1
    fi

    counts=$(rg --hidden --line-number --no-heading --glob '*.rs' \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --regexp '^\s*let\s+(mut\s+)?(two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand|million)\s*[:=]' \
      --regexp '(^\s*(pub(\([^)]*\))?\s+)?|[(,]\s*)(mut\s+)?(two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand|million)\s*:' \
      --regexp '^\s*(pub(\([^)]*\))?\s+)?(const|static)\s+(TWO|THREE|FOUR|FIVE|SIX|SEVEN|EIGHT|NINE|TEN|ELEVEN|TWELVE|THIRTEEN|FOURTEEN|FIFTEEN|SIXTEEN|SEVENTEEN|EIGHTEEN|NINETEEN|TWENTY|THIRTY|FORTY|FIFTY|SIXTY|SEVENTY|EIGHTY|NINETY|HUNDRED|THOUSAND|MILLION)\s*:' ./crates || true)
    if [ -n "$counts" ]; then
      echo "[WORD] a number counts. It never says what the value is."
      echo "[WORD] say what is counted. Four bytes are bytes, and the mask they meet is a mask."
      echo "[WORD] an ordinal before a noun stays. twenty_eighth_unread names an offset."
      printf '%s\n' "$counts"
      refusals=1
    fi

    refusalLines=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])refus(e|es|ed|ing|al|als)([^a-zA-Z]|$)' ./crates || true)
    if [ -n "$refusalLines" ]; then
      echo "[WORD] a refusal in this code is an error. Say error."
      echo "[WORD] the type is Error, the variant is Error::Language, the field is error."
      echo "[WORD] a test that proves one of them ends in is_an_error."
      printf '%s\n' "$refusalLines"
      refusals=1
    fi

    readings=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])readings?([^a-zA-Z]|$)' . || true)
    if [ -n "$readings" ]; then
      echo "[WORD] reading is the act and never the thing. Name what came out of it."
      echo "[WORD] a disassembly, a finding, a report, a measurement, the bytes."
      echo "[WORD] where the act is meant, a program reads, and the sentence has a subject."
      printf '%s\n' "$readings"
      refusals=1
    fi

    everywhere=$(rg --hidden --line-number --no-heading --ignore-case \
      --glob '!nix/flake/checks/word/**' \
      --glob '!crates/*/assets/**' \
      --glob '!crates/*/docs/UBIQUITOUS_LANGUAGE.md' \
      '(^|[^a-zA-Z])written([^a-zA-Z]|$)' . || true)
    if [ -n "$everywhere" ]; then
      echo "[WORD] written names nothing, anywhere, in any language, in code and in prose."
      echo "[WORD] a value is a noun for what it is and never for what an action left behind."
      printf '%s\n' "$everywhere"
      refusals=1
    fi

    if [ "$refusals" != 0 ]; then
      exit 1
    fi

    echo "[WORD] every rule of this repository was read, and none of them refused a line."
  '';
  installPhase = ''
    touch $out
  '';
}
