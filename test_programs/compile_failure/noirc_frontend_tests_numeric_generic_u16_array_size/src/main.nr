
    fn len<let N: u32>(_arr: [Field; N]) -> u32 {
        N
    }

    pub fn foo<let N: u16>() -> u32 {
        let fields: [Field; N] = [0; N];
        len(fields)
    }
    