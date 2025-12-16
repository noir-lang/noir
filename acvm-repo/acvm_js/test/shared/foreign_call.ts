import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 144, 209, 10, 128, 32, 12, 69, 157, 149, 12, 122, 235, 11, 250, 179, 136, 232,
  33, 144, 30, 34, 250, 254, 26, 172, 216, 172, 100, 94, 152, 211, 227, 198, 174, 130, 211, 194, 43, 128, 247, 53, 103,
  112, 111, 221, 172, 119, 38, 1, 216, 107, 213, 60, 255, 44, 124, 225, 13, 141, 45, 231, 101, 61, 230, 109, 31, 166,
  49, 198, 134, 17, 138, 82, 138, 170, 192, 23, 10, 79, 133, 189, 16, 18, 32, 159, 68, 118, 131, 178, 156, 251, 241,
  191, 243, 23, 235, 18, 78, 83, 79, 252, 193, 219, 106, 242, 1, 0, 0,
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
