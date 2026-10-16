pub struct Capped<let MAX: u32, let SCALE: u32> {
    v: u32,
}

impl<let MAX: u32, let SCALE: u32> Capped<MAX, SCALE> {
    fn check(self) -> u32 {
        assert(self.v <= MAX, "value above cap");
        self.v * SCALE
    }
}

type Scaled<let SCALE: u32, let MAX: u32> = Capped<MAX, SCALE>;

fn main(v: u32) -> pub u32 {
    // `Scaled<1000, 8>` is `Capped<8, 1000>`, so `v = 500` fails the cap.
    let c = Scaled::<1000, 8> { v };
    c.check()
}
