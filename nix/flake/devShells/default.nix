{
  callPackage,
  host,
  rustToolchain,
}:
{
  default = callPackage ./sony {
    inherit
      host
      rustToolchain
      ;
  };
}
