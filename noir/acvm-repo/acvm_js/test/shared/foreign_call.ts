import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 177, 10, 192, 32, 12, 68, 207, 148, 150, 118, 234, 175, 216, 63, 232,
  207, 116, 232, 226, 32, 226, 247, 171, 24, 225, 6, 113, 209, 7, 33, 199, 5, 194, 221, 9, 192, 160, 178, 145, 102, 154,
  247, 234, 182, 115, 60, 102, 221, 47, 203, 121, 69, 59, 20, 246, 78, 254, 198, 149, 231, 80, 253, 187, 248, 249, 48,
  106, 205, 220, 189, 187, 144, 33, 24, 144, 0, 93, 119, 243, 238, 108, 1, 0, 0,
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
