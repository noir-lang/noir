
    pub trait Trait {
        let N: u32;
    }

    impl Trait for Field {
        let N: u32 = 1;
    }

    impl Trait for i32 {
        let N: u32 = 999;
    }

    pub fn load<T>()
    where
        T: Trait,
    {
        let _ = <T as Trait>::N;
    }

    fn main() {
        let _ = load::<Field>();
        let _ = load::<i32>();
    }
    