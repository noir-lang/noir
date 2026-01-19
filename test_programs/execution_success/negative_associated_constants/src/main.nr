trait Foo {
    let N: i32;
    let M: i32;

    fn n() -> i32 {
        Self::N
    }

    fn m() -> i32 {
        Self::M
    }
}

impl Foo for i32 {
    let N: i32 = -12345;
    let M: i32 = 12345 - 20000;
}

fn main() {
    println(i32::n());
    println(i32::m());
}
