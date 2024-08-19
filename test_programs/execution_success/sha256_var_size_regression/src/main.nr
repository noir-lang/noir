fn main(foo: [u8; 95], toggle: bool) {
    let size: Field = 93 + toggle as Field * 2;
    let hash = std::sha256::sha256_var(foo, size as u64);
    println(f"{hash}");
}  