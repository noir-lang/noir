import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 81, 203, 10, 128, 48, 12, 179, 243, 193, 192, 155, 95, 178, 253, 129, 63, 227,
  193, 139, 7, 17, 191, 223, 137, 45, 4, 201, 188, 216, 64, 73, 27, 182, 146, 108, 210, 60, 136, 165, 68, 251, 78, 217,
  102, 132, 105, 179, 114, 250, 135, 44, 126, 187, 18, 250, 13, 239, 70, 80, 252, 8, 214, 195, 131, 160, 126, 115, 235,
  104, 54, 18, 127, 142, 251, 243, 64, 50, 6, 146, 119, 44, 101, 103, 215, 237, 92, 246, 131, 125, 59, 222, 168, 205,
  53, 125, 34, 186, 57, 185, 0, 144, 108, 110, 185, 127, 2, 0, 0,
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
