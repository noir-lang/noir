import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 177, 10, 192, 32, 12, 68, 207, 148, 150, 118, 234, 175, 216, 63, 232,
  207, 116, 232, 210, 161, 136, 223, 175, 98, 132, 27, 212, 69, 31, 132, 28, 23, 8, 119, 59, 0, 131, 204, 66, 154, 41,
  222, 173, 219, 142, 113, 153, 121, 191, 44, 231, 21, 237, 144, 88, 43, 249, 11, 71, 156, 77, 245, 251, 249, 231, 119,
  189, 214, 204, 89, 187, 11, 25, 130, 54, 1, 36, 1, 124, 242, 107, 1, 0, 0,
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
