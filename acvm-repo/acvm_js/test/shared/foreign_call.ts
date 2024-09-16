import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 81, 73, 10, 192, 48, 8, 140, 165, 91, 160, 183, 126, 196, 254, 160, 159, 233,
  161, 151, 30, 74, 200, 251, 19, 136, 130, 132, 196, 75, 28, 16, 199, 17, 212, 65, 112, 5, 123, 14, 32, 190, 80, 230,
  90, 130, 181, 155, 50, 142, 225, 2, 187, 89, 40, 239, 157, 106, 2, 82, 116, 138, 51, 118, 239, 171, 222, 108, 232,
  218, 139, 125, 198, 179, 113, 83, 188, 29, 57, 86, 226, 239, 23, 159, 63, 104, 63, 238, 213, 45, 237, 108, 244, 18,
  195, 174, 252, 193, 92, 2, 0, 0,
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
