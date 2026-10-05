pub struct Narrow {
    pub v: Field,
}

pub struct Wide {
    pub v: Field,
}

pub trait Other {
    fn validate(self) -> Field;
}

impl Other for Narrow {
    fn validate(self) -> Field {
        assert(self.v.lt(8));
        self.v
    }
}

impl Other for Wide {
    fn validate(self) -> Field {
        self.v
    }
}

pub trait Checked: Other {
    fn tag(self) -> Field {
        let _ = self;
        0
    }

    // `Wide` has no `Checked` impl. Accepting this call would bind the trait's `Self` to `Wide`
    // and make `checked` dispatch to `Wide`'s `validate` for every implementor.
    fn unused_helper() -> Field {
        g(Wide { v: 0 })
    }

    fn checked(self) -> Field {
        Other::validate(self)
    }
}

impl Checked for Narrow {}

fn g<T: Checked>(t: T) -> Field {
    t.tag()
}

pub fn verify(w: Field) -> Field {
    Narrow { v: w }.checked()
}
