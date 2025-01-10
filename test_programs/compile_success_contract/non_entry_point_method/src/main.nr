contract Foo {
    pub struct PlaceholderStruct{x : u32 }

    #[contract_library_method]
    pub fn has_mut(_context: &mut PlaceholderStruct) {}
}
