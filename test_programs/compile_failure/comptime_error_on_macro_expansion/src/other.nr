pub comptime fn expr() -> Quoted {
    quote { 1 + "a" }
}
