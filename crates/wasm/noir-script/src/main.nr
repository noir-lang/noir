// Noir Compiler version: https://github.com/noir-lang/noir/tree/noir_wasm_package_json.compiler.versionHash
// Example React Application presenting Noir source compilation, 
// Proof creation and Proof verification using Create React App (https://create-react-app.dev).
// Below Noir source code is "Hello world" example in Noir.
//
// Use "Prove" Button below (or CTRL-P | CTRL-R key combinations) to compile, prove and verify.
//
// use dep::std;

// @value x = 4
// @value y = 5
fn main(x : u64, y : pub u64) {
  assert(x < y);
}