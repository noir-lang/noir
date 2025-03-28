import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 81, 203, 10, 128, 48, 12, 179, 243, 57, 240, 230, 143, 108, 127, 224, 207,
  120, 240, 226, 65, 196, 239, 119, 98, 11, 101, 100, 94, 214, 64, 73, 26, 88, 73, 24, 53, 31, 166, 52, 196, 186, 99,
  150, 93, 67, 188, 149, 57, 212, 33, 146, 221, 173, 160, 243, 186, 92, 144, 54, 127, 138, 245, 204, 62, 243, 95, 110,
  13, 195, 122, 144, 207, 240, 126, 28, 65, 71, 7, 250, 206, 105, 6, 214, 251, 113, 111, 231, 133, 190, 93, 191, 40,
  237, 37, 127, 1, 190, 36, 121, 0, 128, 254, 118, 42, 127, 2, 0, 0,
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
