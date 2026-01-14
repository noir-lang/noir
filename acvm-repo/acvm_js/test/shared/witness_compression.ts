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
  31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 133, 204, 65, 17, 0, 32, 12, 196, 192, 107, 15, 124, 160, 150, 127, 209, 132, 176,
  58, 104, 242, 222, 137, 171, 158, 174, 254, 209, 92, 4, 146, 76, 190, 152, 201, 66, 162, 141, 196, 13, 57, 211, 99,
  86, 216, 0, 0, 0,
]);

export const expectedWitnessMap = new Map([
  [0, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [1, '0x0000000000000000000000000000000000000000000000000000000000000002'],
  [2, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [3, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [4, '0x0000000000000000000000000000000000000000000000000000000000000000'],
  [5, '0x0000000000000000000000000000000000000000000000000000000000000003'],
]);
