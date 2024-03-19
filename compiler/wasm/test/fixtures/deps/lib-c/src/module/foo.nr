pub fn bar(param: Field) -> Field {
    dep::std::hash::pedersen_hash([param])
}
