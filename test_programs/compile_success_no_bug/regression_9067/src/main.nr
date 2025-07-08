fn main(c: bool) -> pub Field {
    let mut s = 0;
    if c {
        for i in 0..150 {
            for j in 0..150 {
                if c {
                    s += (i as Field);
                } else {
                    s += (j as Field);
                }
            }
        }
    }
    s
}
