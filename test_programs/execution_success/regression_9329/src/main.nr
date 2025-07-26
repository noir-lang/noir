fn main(mut b: [i8; 2]) -> pub i8 {
    b[{
        let d = 1;
        for _ in 0..2 {
            b[0] = b[1] + d;
        }
        0
    }]
}
