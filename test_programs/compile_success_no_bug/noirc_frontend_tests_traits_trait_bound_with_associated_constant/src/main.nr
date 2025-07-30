
    pub trait Other {
        let N: u32;
    }

    pub trait Trait<T>
    where
        T: Other,
    {}

    impl Other for Field {
        let N: u32 = 1;
    }

    impl Trait<Field> for i32 {}

    fn main() {}
    