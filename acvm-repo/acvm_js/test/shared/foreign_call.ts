import { WitnessMap } from '@noir-lang/acvm_js';

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 61, 10, 192, 32, 12, 133, 19, 11, 165, 116, 235, 77, 236, 13, 122, 153,
  14, 93, 58, 136, 120, 124, 241, 47, 129, 12, 42, 130, 126, 16, 18, 146, 16, 222, 11, 66, 225, 136, 129, 84, 111, 162,
  150, 112, 239, 161, 172, 231, 184, 113, 221, 45, 45, 245, 42, 242, 144, 216, 43, 250, 153, 83, 204, 191, 223, 189,
  198, 246, 92, 39, 60, 244, 63, 195, 59, 87, 99, 150, 165, 113, 83, 193, 0, 1, 19, 247, 29, 5, 160, 1, 0, 0,
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
