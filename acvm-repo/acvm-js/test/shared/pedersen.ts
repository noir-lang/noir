// See `pedersen_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 9, 10, 0, 64, 8, 2, 103, 15, 250, 255, 139, 163, 162, 130, 72, 16, 149,
  241, 3, 135, 84, 164, 172, 173, 213, 175, 251, 45, 198, 96, 243, 211, 50, 152, 67, 220, 211, 92, 0, 0, 0,
]);

export const initialWitnessMap = new Map([[1, '0x0000000000000000000000000000000000000000000000000000000000000001']]);

export const expectedWitnessMap = new Map([
  [1, '0x0000000000000000000000000000000000000000000000000000000000000001'],
  [2, '0x09489945604c9686e698cb69d7bd6fc0cdb02e9faae3e1a433f1c342c1a5ecc4'],
  [3, '0x24f50d25508b4dfb1e8a834e39565f646e217b24cb3a475c2e4991d1bb07a9d8'],
]);
