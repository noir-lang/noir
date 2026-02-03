pub struct MyStruct {
    pub Q: Field,
}

impl crate1::MyTrait for MyStruct {
    fn Add10(&mut self) {
        self.Q += 10;
    }
}
