// See `pedersen_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 74, 135, 9, 0, 48, 8, 75, 171, 224, 255, 15, 139, 27, 196, 64, 200, 100, 0, 15,
  133, 80, 57, 89, 219, 127, 39, 173, 126, 235, 236, 247, 151, 48, 224, 71, 90, 33, 97, 0, 0, 0,
]);

export const initialWitnessMap = new Map([[1, '0x0000000000000000000000000000000000000000000000000000000000000001']]);

export const expectedWitnessMap = new Map([
  [1, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [2, '0x083e7911d835097629f0067531fc15cafd79a89beecb39903f69572c636f4a5a'],
  [3, '0x1a7f5efaad7f315c25a918f30cc8d7333fccab7ad7c90f14de81bcc528f9935d'],
]);
