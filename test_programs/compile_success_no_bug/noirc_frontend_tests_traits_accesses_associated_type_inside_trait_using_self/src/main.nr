
    pub trait Trait {
        let N: u32;

        fn foo() -> u32 {
            Self::N
        }
    }

    impl Trait for i32 {
        let N: u32 = 10;
    }

    fn main() {
        let _ = i32::foo();
    }
    