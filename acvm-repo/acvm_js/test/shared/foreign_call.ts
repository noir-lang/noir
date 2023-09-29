import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 81, 10, 0, 16, 16, 68, 199, 42, 57, 14, 55, 112, 25, 31, 126, 124, 72,
  206, 79, 161, 86, 225, 135, 87, 219, 78, 187, 53, 205, 104, 0, 2, 29, 201, 52, 103, 222, 220, 216, 230, 13, 43, 254,
  121, 25, 158, 151, 54, 153, 117, 27, 53, 116, 136, 197, 167, 124, 107, 184, 64, 236, 73, 56, 83, 1, 18, 139, 122, 157,
  67, 1, 0, 0,
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
