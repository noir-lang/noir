fn main() {
    let mut x = 2;
    add1(&mut x);
    assert(x == 3);

    let mut s = S { y: x };
    s.add2();
    assert(s.y == 5);
}

fn add1(x: &mut Field) {
    *x += 1;
}

struct S { y: Field }

impl S {
    fn add2(&mut self) {
        self.y += 2;
    }
}
