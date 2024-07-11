fn main() {
    std::meta::unquote!(quote { assert(true); });
    assert(std::meta::unquote!(quote { true }));
}
