use crate::module1::MyTrait;
use crate::module2::MyStruct;

// those are not the same 'Path', but they refer to the same trait & impl
// so a Duplicate error should be thrown
impl MyTrait for MyStruct {}
impl crate::module1::MyTrait for crate::module2::MyStruct { }
