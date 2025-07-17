
    pub trait Trait {
        fn foo<let U: u32>(self, _msg: str<U>) { 
            let _ = self; 
        }

        fn bar<let U: u32>(self, _msg: str<U>) {
            let _ = self;
        }
    }

    pub struct Struct {}

    impl Trait for Struct {}

    pub fn main() {
        Struct {}.bar("Hello");
    }
    