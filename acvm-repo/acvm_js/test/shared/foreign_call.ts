import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 149, 81, 237, 10, 128, 32, 12, 116, 246, 193, 160, 127, 61, 65, 111, 22, 17, 253,
  8, 164, 31, 17, 61, 127, 69, 91, 204, 156, 48, 7, 58, 61, 239, 240, 142, 129, 139, 11, 239, 5, 116, 174, 169, 131, 75,
  139, 177, 193, 153, 10, 192, 206, 141, 254, 243, 223, 70, 15, 222, 32, 236, 168, 175, 219, 185, 236, 199, 56, 79, 33,
  52, 4, 225, 143, 250, 244, 170, 192, 27, 74, 95, 229, 122, 104, 21, 80, 70, 146, 17, 152, 251, 198, 208, 166, 32, 21,
  185, 123, 14, 239, 21, 156, 157, 92, 163, 94, 232, 115, 22, 2, 0, 0,
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
