use std::validation::AssertsIsValidInput;

// When we re-export this type from another library and then use it in
// main, we get a panic
#[derive(AssertsIsValidInput)]
pub struct ReExportMeFromAnotherLib {
    x: Field,
}
