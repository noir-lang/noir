import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 177, 10, 192, 32, 16, 67, 227, 21, 74, 233, 212, 79, 177, 127, 208, 159,
  233, 224, 226, 32, 226, 247, 139, 168, 16, 68, 93, 244, 45, 119, 228, 142, 144, 92, 0, 20, 50, 7, 237, 76, 213, 190,
  50, 245, 26, 175, 218, 231, 165, 57, 175, 148, 14, 137, 179, 147, 191, 114, 211, 221, 216, 240, 59, 63, 107, 221, 115,
  104, 181, 103, 244, 43, 36, 10, 38, 68, 108, 25, 253, 238, 136, 1, 0, 0,
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
