trait Serialize {
    let Size: u32;

    fn serialize(self);
}

impl Serialize for Field {
    let Size: u32 = 1;

    fn serialize(self) {}
}

impl<A> Serialize for (A,)
where
    A: Serialize,
{
    let Size: u32 = <A as Serialize>::Size;

    fn serialize(self: Self) {
        self.0.serialize();
    }
}

fn main() {
    let x = ((1,),);
    x.serialize();
}
