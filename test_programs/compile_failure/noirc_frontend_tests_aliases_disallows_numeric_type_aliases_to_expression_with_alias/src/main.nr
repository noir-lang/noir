
    type Double<let N: u32>: u32 = N * 2;
    type Quadruple<let N: u32>: u32 = Double::<N>+Double::<N>;
    fn main() {
        let b: [u32; 12] = foo();
        assert(b[0] == 0);
    }
    fn foo<let N:u32>() -> [u32;Quadruple::<N>] {
        let n = Double::<N>;    // To avoid the unused 'Double' error
        let mut a = [0;Quadruple::<N>];
        for i in 0..Quadruple::<N> {
            a[i] = i + n;
        }
        a
    }
    