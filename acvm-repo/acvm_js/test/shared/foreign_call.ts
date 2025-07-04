import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 80, 203, 10, 128, 48, 12, 91, 124, 81, 240, 230, 23, 248, 115, 30, 188, 120,
  16, 241, 251, 157, 216, 64, 29, 221, 216, 2, 37, 109, 104, 104, 40, 194, 7, 137, 5, 237, 7, 101, 206, 22, 212, 214,
  80, 5, 160, 126, 247, 119, 175, 75, 27, 88, 177, 96, 30, 149, 37, 209, 95, 238, 27, 194, 136, 115, 191, 193, 143, 201,
  17, 109, 126, 230, 154, 99, 113, 119, 63, 238, 237, 188, 188, 183, 91, 71, 110, 206, 233, 139, 163, 51, 201, 3, 1, 24,
  56, 224, 255, 1, 0, 0,
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
