fn main() {
    let a = id([1, 2]);
    let b = id([1, 2, 3]);

    let itWorks1 = MyStruct { data: a };
    constrain itWorks1.data[1] == 2;
    let itWorks2 = MyStruct { data: b };
    constrain itWorks2.data[1] == 2;

    let c = [1, 2];
    let itAlsoWorks = MyStruct { data: c };
    constrain itAlsoWorks.data[1] == 2;
}

fn id<I>(x: [Field; I]) -> [Field; I] {
    x
}

struct MyStruct<S> {
    data: [Field; S]
}
