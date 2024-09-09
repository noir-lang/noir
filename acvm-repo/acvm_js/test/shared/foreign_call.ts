import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 80, 49, 10, 192, 32, 12, 52, 45, 45, 133, 110, 190, 68, 127, 224, 103, 28, 92,
  28, 68, 124, 191, 130, 9, 4, 137, 46, 122, 16, 46, 119, 7, 33, 9, 168, 142, 175, 21, 96, 255, 32, 147, 230, 32, 207,
  33, 155, 61, 88, 56, 55, 203, 240, 125, 175, 177, 1, 110, 170, 197, 101, 55, 242, 43, 100, 132, 159, 229, 33, 22, 159,
  242, 234, 87, 51, 45, 121, 90, 200, 42, 48, 209, 35, 111, 164, 1, 0, 0,
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
