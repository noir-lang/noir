
    type Double<let N: u32>: u32 = N * 2;
    fn main() {
        let b: [u32; 6] = foo();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Double::<N>] {
        let mut a = [0;Double::<N>];
        for i in 0..Double::<N> {
            a[i] = i;
        }
        a
    }
    