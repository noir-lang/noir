trait MyTrait {
    fn Add10(&mut self);
}

impl MyTrait for dep::crate2::MyStruct {
    fn Add10(&mut self) {
        self.Q += 10;
    }
}
