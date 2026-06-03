// Regression test for https://github.com/noir-lang/noir/issues/11551
//
// A trait method whose return type chains associated-type projections
// (`<Self::Target as Mappable>::Target`) must resolve and execute for a concrete impl.
trait Mappable {
    type Target;
    fn map_to(self) -> Self::Target;
}

trait Chainable: Mappable {
    fn chain(self) -> <Self::Target as Mappable>::Target
    where
        Self::Target: Mappable;
}

impl Mappable for Field {
    type Target = bool;
    fn map_to(self) -> Self::Target {
        self != 0
    }
}

impl Mappable for bool {
    type Target = u32;
    fn map_to(self) -> Self::Target {
        if self {
            1
        } else {
            0
        }
    }
}

impl Chainable for Field {
    fn chain(self) -> <Self::Target as Mappable>::Target
    where
        Self::Target: Mappable,
    {
        self.map_to().map_to()
    }
}

fn main(x: Field) {
    let result: u32 = x.chain();
    assert(result == 1);
}
