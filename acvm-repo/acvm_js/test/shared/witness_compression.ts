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
  31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 149, 204, 185, 17, 0, 0, 8, 2, 65, 240, 171, 195, 254, 171, 52, 212, 16, 47, 34,
  216, 129, 216, 234, 236, 134, 20, 169, 91, 179, 199, 175, 63, 108, 232, 22, 169, 91, 31, 143, 90, 63, 92, 28, 1, 0, 0,
]);

export const expectedWitnessMap = new Map([
  [0, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [1, '0x0000000000000000000000000000000000000000000000000000000000000002'],
  [2, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [3, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [4, '0x0000000000000000000000000000000000000000000000000000000000000000'],
  [5, '0x0000000000000000000000000000000000000000000000000000000000000003'],
]);
