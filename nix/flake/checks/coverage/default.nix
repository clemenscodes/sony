{
  cargo-llvm-cov,
  craneLib,
  host,
  jq,
  name,
  source,
  rustToolchain,
}:
let
  workspace = craneLib.crateNameFromCargoToml { cargoToml = "${source}/Cargo.toml"; };
  tools = "${rustToolchain}/lib/rustlib/${host}/bin";
  shared = {
    inherit (workspace) version;
    pname = "coverage-${name}";
    src = source;
    strictDeps = true;
    cargoExtraArgs = "--package ${name}";
  };
  cargoArtifacts = craneLib.buildDepsOnly (shared // { doCheck = false; });
in
craneLib.buildPackage (
  shared
  // {
    inherit cargoArtifacts;
    doCheck = true;
    nativeBuildInputs = [
      cargo-llvm-cov
      jq
    ];
    env = {
      LLVM_COV = "${tools}/llvm-cov";
      LLVM_PROFDATA = "${tools}/llvm-profdata";
    };
    preCheck = ''
      measure() {
        manifest=$(cargo metadata --no-deps --format-version 1 \
          | jq --raw-output --arg name '${name}' '.packages[] | select(.name == $name) | .manifest_path')
        directory=$(dirname "$manifest")

        targets='--bins'
        if [ -f "$directory/src/lib.rs" ]; then
          targets='--lib'
        fi

        code=0
        while IFS= read -r source_file; do
          case "$source_file" in
            */src/main.rs) continue ;;
          esac
          if grep --quiet '[^[:space:]]' "$source_file"; then
            code=1
          fi
        done < <(find "$directory/src" -name '*.rs')

        if [ "$code" = 0 ]; then
          echo "[COVERAGE] ${name} keeps no code of its own, so there is nothing for a test to read."
          return 0
        fi

        exceptions='src/main\.rs$'

        cargo llvm-cov clean --workspace
        cargo llvm-cov --package ${name} $targets \
          --ignore-filename-regex "$exceptions" \
          --json --output-path coverage.json

        jq --raw-output '
          .data[0].totals
          | "[COVERAGE] ${name} reads \(.regions.covered) of \(.regions.count) regions, \(.functions.covered) of \(.functions.count) functions, \(.lines.covered) of \(.lines.count) lines."
        ' coverage.json

        coverage=$(jq '
          .data[0].totals
          | .regions.covered == .regions.count
            and .functions.covered == .functions.count
            and .lines.covered == .lines.count
        ' coverage.json)

        if [ "$coverage" != "true" ]; then
          echo "[COVERAGE] a line no test reaches is a claim nothing proves."
          jq --raw-output '
            .data[0].files[]
            | select(.summary.regions.covered != .summary.regions.count
                  or .summary.functions.covered != .summary.functions.count
                  or .summary.lines.covered != .summary.lines.count)
            | "[COVERAGE] \(.filename) regions \(.summary.regions.covered)/\(.summary.regions.count) functions \(.summary.functions.covered)/\(.summary.functions.count) lines \(.summary.lines.covered)/\(.summary.lines.count)"
          ' coverage.json

          jq --raw-output '
            .data[0].files[]
            | .filename as $file
            | .segments[]
            | select(.[2] == 0 and .[4] == true)
            | "[COVERAGE] \($file):\(.[0]):\(.[1]) is a region no test reaches"
          ' coverage.json
          exit 1
        fi

        echo "[COVERAGE] ${name} is read through, every region, every function, every line."
      }
    '';
    cargoTestCommand = "measure";
  }
)
