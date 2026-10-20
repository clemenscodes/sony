{
  cargo-nextest,
  craneLib,
  name,
  source,
}:
let
  workspace = craneLib.crateNameFromCargoToml { cargoToml = "${source}/Cargo.toml"; };
in
craneLib.buildPackage {
  inherit (workspace) version;
  pname = name;
  src = source;
  strictDeps = true;
  doCheck = true;
  nativeBuildInputs = [ cargo-nextest ];
  cargoExtraArgs = "--package ${name}";
  env.NEXTEST_SHOW_PROGRESS = "counter";
  cargoTestCommand = "cargo nextest run --cargo-profile release --profile release --no-tests=warn";
  extraDummyScript = ''
    mkdir --parents $out/.config
    cp ${source}/.config/nextest.toml $out/.config/nextest.toml
  '';
}
