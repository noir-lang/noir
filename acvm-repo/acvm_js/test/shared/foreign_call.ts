import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 49, 10, 64, 33, 12, 67, 99, 63, 124, 60, 142, 222, 192, 203, 56, 184, 56,
  136, 120, 126, 5, 21, 226, 160, 139, 62, 40, 13, 45, 132, 68, 3, 80, 232, 124, 164, 153, 121, 115, 99, 155, 59, 172,
  122, 231, 101, 56, 175, 80, 86, 221, 230, 31, 58, 196, 226, 83, 62, 53, 91, 16, 122, 10, 246, 84, 99, 243, 0, 30, 59,
  1, 0, 0,
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
