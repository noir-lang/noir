import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 49, 10, 0, 33, 12, 4, 215, 28, 28, 62, 199, 251, 193, 125, 198, 194, 198,
  66, 196, 247, 43, 168, 176, 136, 218, 232, 64, 200, 50, 69, 216, 104, 0, 10, 149, 135, 50, 211, 221, 223, 182, 57,
  227, 83, 247, 110, 25, 238, 43, 212, 85, 151, 121, 91, 118, 62, 217, 16, 119, 159, 141, 121, 234, 132, 164, 96, 77, 6,
  148, 102, 152, 53, 83, 1, 0, 0,
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
