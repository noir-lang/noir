// When we re-export this type from another library and then use it in
// main, we get a panic
pub struct ReExportMeFromAnotherLib {
    x : Field,
}
