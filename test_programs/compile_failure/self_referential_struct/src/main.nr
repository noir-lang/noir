struct Option2<T> {
    _is_some: bool,
    _value: T,
}

struct SelfReferential
{
    prop : Option2<SelfReferential>
}

fn main(x: SelfReferential) { assert(x._is_some); }
