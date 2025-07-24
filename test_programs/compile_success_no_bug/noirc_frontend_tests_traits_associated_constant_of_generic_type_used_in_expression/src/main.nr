
    trait Serialize {
        let N: u32;
    }

    impl<let M: u32> Serialize for [Field; M] {
        let N: u32 = M;
    }

    fn main() {
        let _ = <[Field; 3] as Serialize>::N;
    }
    