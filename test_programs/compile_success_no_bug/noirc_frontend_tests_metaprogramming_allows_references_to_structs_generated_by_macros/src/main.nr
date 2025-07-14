
    comptime fn make_new_struct(_s: TypeDefinition) -> Quoted {
        quote { struct Bar {} }
    }

    #[make_new_struct]
    struct Foo {}

    fn main() {
        let _ = Foo {};
        let _ = Bar {};
    }
    