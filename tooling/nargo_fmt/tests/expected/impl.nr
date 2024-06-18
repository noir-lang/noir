impl MyType {}

impl MyType<A> {}

impl MyType<A, B> {}

impl MyType {
    fn method(self) {}

    fn method(mut self) {}

    fn method(&mut self) {}
}

impl MyType {
    fn method(self) {}
}

impl MyType {
    fn method(self) {}
}
