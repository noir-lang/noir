import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 79, 73, 10, 128, 48, 12, 204, 40, 46, 5, 111, 126, 36, 254, 192, 207, 120,
  240, 226, 65, 196, 247, 91, 48, 129, 80, 218, 122, 48, 3, 33, 147, 9, 89, 6, 244, 98, 140, 1, 225, 157, 100, 173, 45,
  84, 91, 37, 243, 63, 44, 240, 219, 197, 246, 223, 38, 37, 176, 34, 85, 156, 169, 251, 144, 233, 183, 142, 206, 67,
  114, 215, 121, 63, 15, 84, 135, 222, 157, 98, 244, 194, 247, 227, 222, 206, 11, 31, 19, 165, 186, 164, 207, 153, 222,
  3, 91, 101, 84, 220, 120, 2, 0, 0,
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
