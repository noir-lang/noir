import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 193, 10, 192, 32, 8, 134, 117, 99, 99, 236, 182, 55, 105, 111, 176, 151,
  217, 161, 75, 135, 136, 30, 63, 42, 82, 144, 8, 47, 245, 65, 252, 230, 47, 162, 34, 52, 174, 242, 144, 226, 131, 148,
  255, 18, 206, 125, 164, 102, 142, 23, 215, 245, 50, 114, 222, 173, 15, 80, 38, 65, 217, 108, 39, 61, 7, 30, 115, 11,
  223, 186, 248, 251, 160, 221, 170, 146, 64, 191, 39, 215, 60, 3, 47, 3, 99, 171, 188, 84, 164, 1, 0, 0,
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
