use std::validation::AssertsIsValidInput;

mod bar;

#[derive(AssertsIsValidInput)]
struct fooStruct {
    bar_struct: bar::barStruct,
    baz: Field,
}
