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
  31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 141, 204, 177, 9, 192, 32, 16, 70, 225, 223, 51, 217, 35, 171, 133, 196, 66, 4,
  155, 19, 180, 213, 194, 222, 33, 196, 165, 28, 198, 13, 60, 95, 253, 241, 40, 119, 14, 239, 231, 90, 233, 214, 255,
  38, 97, 68, 27, 188, 97, 174, 152, 120, 176, 79, 41, 217, 16, 29, 124, 244, 129, 185, 100, 131, 91, 54, 122, 1, 182,
  39, 151, 52, 242, 0, 0, 0,
]);

export const expectedWitnessMap = new Map([
  [0, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [1, '0x0000000000000000000000000000000000000000000000000000000000000002'],
  [2, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [3, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [4, '0x0000000000000000000000000000000000000000000000000000000000000000'],
  [5, '0x0000000000000000000000000000000000000000000000000000000000000003'],
]);
