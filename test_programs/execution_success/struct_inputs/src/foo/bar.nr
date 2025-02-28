use std::validation::AssertsIsValidInput;

global N: Field = 2;

#[derive(AssertsIsValidInput)]
struct barStruct {
    val: Field,
    array: [Field; 2],
    message: str<5>,
}
