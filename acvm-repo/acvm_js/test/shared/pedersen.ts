// See `pedersen_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 74, 9, 10, 0, 0, 4, 115, 149, 255, 127, 88, 8, 133, 213, 218, 137, 80, 144, 32,
  182, 79, 213, 151, 173, 61, 5, 121, 245, 91, 103, 255, 191, 3, 7, 16, 26, 112, 158, 113, 0, 0, 0,
]);

export const initialWitnessMap = new Map([[1, '0x0000000000000000000000000000000000000000000000000000000000000001']]);

export const expectedWitnessMap = new Map([
  [1, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [2, '0x083e7911d835097629f0067531fc15cafd79a89beecb39903f69572c636f4a5a'],
  [3, '0x1a7f5efaad7f315c25a918f30cc8d7333fccab7ad7c90f14de81bcc528f9935d'],
]);
