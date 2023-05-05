fn bar(param: Field) -> Field {
  dep::std::hash::pedersen([param])[0]
}