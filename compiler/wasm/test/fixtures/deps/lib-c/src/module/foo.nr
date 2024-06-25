pub fn bar(param: Field) -> Field {
    std::hash::pedersen_hash([param])
}
