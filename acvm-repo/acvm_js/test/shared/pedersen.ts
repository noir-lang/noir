// See `pedersen_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 9, 10, 0, 64, 8, 2, 103, 15, 232, 255, 31, 142, 138, 10, 34, 65, 84, 198,
  15, 28, 82, 145, 178, 182, 86, 191, 238, 183, 24, 131, 205, 79, 203, 0, 166, 242, 158, 93, 92, 0, 0, 0,
]);

export const initialWitnessMap = new Map([[1, '0x0000000000000000000000000000000000000000000000000000000000000001']]);

export const expectedWitnessMap = new Map([
  [1, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [2, '0x083e7911d835097629f0067531fc15cafd79a89beecb39903f69572c636f4a5a'],
  [3, '0x1a7f5efaad7f315c25a918f30cc8d7333fccab7ad7c90f14de81bcc528f9935d'],
]);
