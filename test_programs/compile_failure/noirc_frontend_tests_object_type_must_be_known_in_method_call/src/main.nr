
    pub fn foo<let N: u32>() -> [Field; N] {
        let array = [];
        let mut bar = array[0];
        let _ = bar.len();
        bar
    }

    fn main() {}
    