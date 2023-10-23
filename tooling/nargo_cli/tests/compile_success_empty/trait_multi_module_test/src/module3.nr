use crate::module1::MyTrait;
use crate::module2::MyStruct;

// ensure we can implement traits that are imported with the `use` syntax
impl MyTrait for MyStruct {}
