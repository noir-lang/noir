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
  31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 173, 206, 185, 13, 0, 48, 8, 67, 209, 144, 107, 30, 146, 44, 144, 253, 167, 162,
  65, 130, 158, 239, 198, 174, 158, 44, 45, 178, 211, 254, 222, 90, 203, 17, 206, 186, 29, 252, 53, 64, 107, 114, 150,
  46, 206, 122, 6, 24, 73, 44, 193, 220, 1, 0, 0,
]);

export const expectedWitnessMap = new Map([
  [0, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [1, '0x0000000000000000000000000000000000000000000000000000000000000002'],
  [2, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [3, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [4, '0x0000000000000000000000000000000000000000000000000000000000000000'],
  [5, '0x0000000000000000000000000000000000000000000000000000000000000003'],
]);
