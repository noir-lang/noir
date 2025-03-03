pub trait MyTrait {
    fn Add10(&mut self);
}

impl MyTrait for crate2::MyStruct {
    fn Add10(&mut self) {
        self.Q += 10;
    }
}
