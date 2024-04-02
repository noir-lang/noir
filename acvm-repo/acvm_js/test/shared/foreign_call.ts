import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 61, 10, 192, 48, 8, 133, 53, 133, 82, 186, 245, 38, 233, 13, 122, 153,
  14, 93, 58, 132, 144, 227, 135, 252, 41, 56, 36, 46, 201, 7, 162, 168, 200, 123, 34, 52, 142, 28, 72, 245, 38, 106, 9,
  247, 30, 202, 118, 142, 27, 215, 221, 178, 82, 175, 33, 15, 133, 189, 163, 159, 57, 197, 252, 251, 195, 235, 188, 230,
  186, 16, 65, 255, 12, 239, 92, 131, 89, 149, 198, 77, 3, 10, 9, 119, 8, 198, 242, 152, 1, 0, 0,
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
