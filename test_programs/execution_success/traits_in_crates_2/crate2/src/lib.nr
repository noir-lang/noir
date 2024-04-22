struct MyStruct {
    Q: field,
}

impl dep::crate1::MyTrait for MyStruct {
    fn Add10(&mut self) {
        self.Q += 10;
    }
}
