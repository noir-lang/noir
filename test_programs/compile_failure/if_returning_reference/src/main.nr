fn main() {
    let s1: Alias<_> = S { ref: &mut 0 };
    let s2: Alias<_> = S { ref: &mut 1 };

    let s = if false { s1 } else { s2 };
    println(s);
}

type Alias<T> = T;

struct S {
    ref: &mut u32,
}
