import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 65, 10, 192, 32, 12, 4, 215, 148, 150, 246, 212, 175, 216, 31, 244, 51,
  61, 244, 226, 65, 196, 247, 171, 24, 33, 136, 122, 209, 129, 144, 176, 132, 101, 247, 4, 160, 144, 217, 196, 45, 41,
  218, 203, 91, 207, 241, 168, 117, 94, 90, 230, 37, 238, 144, 216, 27, 249, 11, 87, 156, 131, 239, 223, 248, 207, 186,
  81, 235, 150, 67, 173, 221, 189, 95, 18, 34, 97, 64, 0, 116, 135, 40, 214, 136, 1, 0, 0,
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
