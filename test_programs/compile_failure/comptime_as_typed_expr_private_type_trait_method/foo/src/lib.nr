pub trait Leak {
    fn raw_compute(x: Field) -> Field;
}

struct Hidden {}

impl Leak for Hidden {
    fn raw_compute(x: Field) -> Field {
        x * 1000
    }
}

pub struct PublicWrapper {
    inner: Hidden,
}
