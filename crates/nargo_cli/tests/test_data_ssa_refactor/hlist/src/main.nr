struct HCons<H, T> {
    head: H,
    tail: T,
}

struct HNil {}

fn mk_list() -> HCons<i32, HCons<u8, HCons<(i32, i32), HNil>>,
> {
    HCons {
        head: 42,
        tail: HCons {
            head: 42,
            tail: HCons {
                head: (42, 42),
                tail: HNil {},
            },
        },
    }
}

fn main() {
    mk_list();
}
