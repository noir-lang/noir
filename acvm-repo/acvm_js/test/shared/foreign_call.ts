import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 65, 10, 192, 32, 12, 4, 77, 10, 165, 244, 214, 159, 216, 31, 244, 51, 61,
  120, 241, 32, 226, 251, 85, 140, 176, 136, 122, 209, 129, 144, 176, 9, 97, 151, 84, 225, 74, 69, 50, 31, 48, 35, 85,
  251, 164, 235, 53, 94, 218, 247, 75, 163, 95, 150, 12, 153, 179, 227, 191, 114, 195, 222, 216, 240, 59, 63, 75, 221,
  251, 208, 106, 207, 232, 150, 65, 100, 53, 33, 2, 22, 232, 178, 27, 144, 1, 0, 0,
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
