import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 80, 49, 10, 192, 32, 12, 52, 45, 45, 165, 155, 63, 209, 31, 248, 25, 7, 23, 7,
  17, 223, 175, 96, 2, 65, 162, 139, 30, 132, 203, 221, 65, 72, 2, 170, 227, 107, 5, 216, 63, 200, 164, 57, 200, 115,
  200, 102, 15, 22, 206, 205, 50, 124, 223, 107, 108, 128, 155, 106, 113, 217, 141, 252, 10, 25, 225, 103, 121, 136,
  197, 167, 188, 250, 213, 76, 75, 158, 22, 178, 10, 176, 188, 242, 119, 164, 1, 0, 0,
]);
export const initialWitnessMap: WitnessMap = new Map([
  [1, '0x0000000000000000000000000000000000000000000000000000000000000005'],
]);

export const oracleCallName = 'invert';
export const oracleCallInputs = [['0x0000000000000000000000000000000000000000000000000000000000000005']];

export const oracleResponse = ['0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667'];

export const expectedWitnessMap = new Map([
  [1, '0x0000000000000000000000000000000000000000000000000000000000000005'],
  [2, '0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667'],
]);
