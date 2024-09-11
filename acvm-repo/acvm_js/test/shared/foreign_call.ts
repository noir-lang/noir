import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 81, 49, 10, 128, 48, 12, 108, 196, 138, 224, 230, 75, 226, 15, 252, 140, 131,
  139, 131, 136, 239, 111, 161, 9, 28, 165, 205, 210, 28, 132, 36, 119, 16, 114, 9, 133, 130, 53, 7, 73, 29, 37, 107,
  143, 80, 238, 148, 204, 99, 56, 200, 111, 22, 227, 190, 83, 93, 16, 146, 193, 112, 22, 225, 34, 168, 205, 142, 174,
  241, 218, 206, 179, 121, 49, 188, 109, 57, 84, 191, 159, 255, 122, 63, 235, 199, 189, 190, 197, 237, 13, 45, 1, 20,
  245, 146, 30, 92, 2, 0, 0,
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
