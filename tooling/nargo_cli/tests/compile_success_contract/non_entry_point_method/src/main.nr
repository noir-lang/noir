contract Foo {
    struct PlaceholderStruct{x : u32 }
    
    #[contract_library_method]
    fn has_mut(_context : &mut PlaceholderStruct) {}
}