pub trait Compute {
    fn compute() -> Field;
}

pub struct Public {}

impl Compute for Public {
    fn compute() -> Field {
        1
    }
}
