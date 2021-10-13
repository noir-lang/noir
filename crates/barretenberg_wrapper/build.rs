fn main() {
    println!(
        "cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/crypto/blake2s"
    );
    println!(
        "cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/crypto/schnorr"
    );
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/ecc");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/env");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/srs");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/numeric");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/dsl");
    println!("cargo:rustc-link-search=/usr/lib/llvm-10/lib");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/plonk/composer");
    println!("cargo:rustc-link-lib=static=composer");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/plonk/transcript");
    println!("cargo:rustc-link-lib=static=transcript");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/stdlib/primitives");
    println!("cargo:rustc-link-lib=static=stdlib_primitives");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/stdlib/hash/sha256");
    println!("cargo:rustc-link-lib=static=stdlib_sha256");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/stdlib/hash/blake2s");
    println!("cargo:rustc-link-lib=static=stdlib_blake2s");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/stdlib/encryption/schnorr");
    println!("cargo:rustc-link-lib=static=stdlib_schnorr");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/stdlib/hash/pedersen");
    println!("cargo:rustc-link-lib=static=stdlib_pedersen");

    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/plonk/proof_system");
    println!("cargo:rustc-link-lib=static=proof_system");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/plonk/reference_string");
    println!("cargo:rustc-link-lib=static=reference_string");
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/polynomials");
    println!("cargo:rustc-link-lib=static=polynomials");

    println!("cargo:rustc-link-lib=static=blake2s");
    println!("cargo:rustc-link-lib=static=schnorr");
    println!("cargo:rustc-link-lib=static=numeric");
    println!("cargo:rustc-link-lib=static=ecc");
    println!("cargo:rustc-link-lib=static=dsl");
    println!("cargo:rustc-link-lib=static=srs");
    println!("cargo:rustc-link-lib=stdc++");
    
    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/crypto/keccak");

    println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/crypto/pedersen");
    println!("cargo:rustc-link-lib=static=pedersen");
    println!("cargo:rustc-link-lib=static=keccak");
    println!("cargo:rustc-link-lib=static=env");
    println!("cargo:rustc-link-lib=omp");
}

// /usr/lib/x86_64-linux-gnu/libpthread.so
// …/…/…/…/_deps/leveldb-build/libleveldb.a
/*
println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/proofs");
println!("cargo:rustc-link-lib=rollup_proofs");
println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/crypto/sha256");
println!("cargo:rustc-link-lib=sha256");

println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/crypto/keccak");
println!("cargo:rustc-link-lib=keccak");

println!("cargo:rustc-link-search=../aztec2-internal/barretenberg/build/src/aztec/stdlib/merkle_tree");
println!("cargo:rustc-link-lib=stdlib_merkle_tree");






*/
