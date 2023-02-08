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

    constrain foo(itWorks2).data[0] == itWorks2.data[0] + 1;
}

fn id<I>(x: [Field; I]) -> [Field; I] {
    x
}

struct MyStruct<S> {
    data: [Field; S]
}

fn foo(mut s: MyStruct<2+1>) -> MyStruct<10/2-2> {
    s.data[0] = s.data[0] + 1;
    s
}
