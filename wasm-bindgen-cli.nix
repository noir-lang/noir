{ lib
, rustPlatform
, fetchCrate
, nodejs
, pkg-config
, openssl
, stdenv
, curl
, darwin
, libiconv
, runCommand
}:

rustPlatform.buildRustPackage rec {
  pname = "wasm-bindgen-cli";
  version = "0.2.86";

  src = fetchCrate {
    inherit pname version;
    sha256 = "sha256-56EOiLbdgAcoTrkyvB3t9TjtLaRvGxFUXx4haLwE2QY=";
  };

  cargoSha256 = "sha256-4CPBmz92PuPN6KeGDTdYPAf5+vTFk9EN5Cmx4QJy6yI=";

  nativeBuildInputs = [ pkg-config ];

  buildInputs = [ openssl ] ++ lib.optionals stdenv.isDarwin [
    curl
    # Need libiconv and apple Security on Darwin. See https://github.com/ipetkov/crane/issues/156
    libiconv
    darwin.apple_sdk.frameworks.Security
  ];

  doCheck = false;

  meta = with lib; {
    homepage = "https://rustwasm.github.io/docs/wasm-bindgen/";
    license = with licenses; [ asl20 /* or */ mit ];
    description = "Facilitating high-level interactions between wasm modules and JavaScript";
    maintainers = with maintainers; [ nitsky rizary ];
    mainProgram = "wasm-bindgen";
  };
}
