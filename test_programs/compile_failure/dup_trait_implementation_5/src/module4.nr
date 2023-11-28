// another module in the crate implements the same trait + struct
// a Duplicate error should be thrown
impl crate::module1::MyTrait for crate::module2::MyStruct { }
