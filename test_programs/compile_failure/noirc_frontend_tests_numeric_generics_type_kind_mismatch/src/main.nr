
    fn foo<let N: u32>() -> u16 {
        N as u16
    }

    global J: u16 = 10;

    fn bar<let N: u16>() -> u16 {
        foo::<J>()
    }

    global M: u16 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    