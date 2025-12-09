// Solved witness for noir program (x = 1, y = 2)
//
// fn main(x : Field, y : pub Field) -> pub Field {
//   assert(x != y);
//   x + y
// }
//
// Regenerate this byte array by going to the Noir integration tests and running `/test_programs/execution_success/witness_compression`
// after recompiling Noir to print the witness byte array to be written to file after execution

export const expectedCompressedWitnessMap = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 133, 204, 193, 13, 0, 17, 16, 70, 225, 49, 118, 251, 216, 214, 54, 56, 136, 196,
  101, 36, 92, 57, 184, 43, 66, 52, 162, 48, 29, 248, 223, 249, 203, 227, 58, 37, 253, 38, 140, 54, 125, 180, 174, 208,
  202, 62, 69, 39, 210, 105, 127, 116, 79, 41, 72, 152, 241, 69, 99, 242, 64, 66, 47, 36, 250, 0, 73, 112, 132, 122,
  236, 0, 0, 0,
]);

export const expectedWitnessMap = new Map([
  [0, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [1, '0x0000000000000000000000000000000000000000000000000000000000000002'],
  [2, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [3, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [4, '0x0000000000000000000000000000000000000000000000000000000000000000'],
  [5, '0x0000000000000000000000000000000000000000000000000000000000000003'],
]);
