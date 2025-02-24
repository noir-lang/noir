trait Trait1 {
    fn tralala() -> Field;
}

trait Trait2 {
    fn tralala() -> Field;
}

pub struct Struct1 {}

impl Struct1 {
    fn tralala() -> Field {
        123456
    }
}

impl Trait1 for Struct1 {
    fn tralala() -> Field {
        111111
    }
}

impl Trait2 for Struct1 {
    fn tralala() -> Field {
        222222
    }
}

fn main() {
    // the struct impl takes priority over trait methods
    assert(Struct1::tralala() == 123456);
    // TODO: uncomment these, once we support the <Type as Trait>:: syntax
    //assert(<Struct1 as Trait1>::tralala() == 111111);
    //assert(<Struct1 as Trait2>::tralala() == 222222);
}
