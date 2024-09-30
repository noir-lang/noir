export const and_test_cases: [[string, string], string][] = [
  [
    [
      '0x0000000000000000000000000000000000000000000000000000000000000001',
      '0x0000000000000000000000000000000000000000000000000000000000000002',
    ],
    '0x0000000000000000000000000000000000000000000000000000000000000000',
  ],
  [
    [
      '0x0000000000000000000000000000000000000000000000000000000000000003',
      '0x0000000000000000000000000000000000000000000000000000000000000002',
    ],
    '0x0000000000000000000000000000000000000000000000000000000000000002',
  ],
];

export const xor_test_cases: [[string, string], string][] = [
  [
    [
      '0x0000000000000000000000000000000000000000000000000000000000000001',
      '0x0000000000000000000000000000000000000000000000000000000000000002',
    ],
    '0x0000000000000000000000000000000000000000000000000000000000000003',
  ],
  [
    [
      '0x0000000000000000000000000000000000000000000000000000000000000003',
      '0x0000000000000000000000000000000000000000000000000000000000000002',
    ],
    '0x0000000000000000000000000000000000000000000000000000000000000001',
  ],
];

export const sha256_compression_test_cases: [Uint32Array, Uint32Array, Uint32Array][] = [
  [
    Uint32Array.from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
    Uint32Array.from([1, 2, 3, 4, 5, 6, 7, 8]),
    Uint32Array.from([1862536192, 526086805, 2067405084, 593147560, 726610467, 813867028, 4091010797, 3974542186]),
  ],
];

// https://www.rfc-editor.org/rfc/rfc7693.html#appendix-B
export const blake2s256_test_cases: [Uint8Array, Uint8Array][] = [
  [
    // "abc"
    Uint8Array.from([0x61, 0x62, 0x63]),
    Uint8Array.from([
      0x50, 0x8c, 0x5e, 0x8c, 0x32, 0x7c, 0x14, 0xe2, 0xe1, 0xa7, 0x2b, 0xa3, 0x4e, 0xeb, 0x45, 0x2f, 0x37, 0x45, 0x8b,
      0x20, 0x9e, 0xd6, 0x3a, 0x29, 0x4d, 0x99, 0x9b, 0x4c, 0x86, 0x67, 0x59, 0x82,
    ]),
  ],
];

export const keccak256_test_cases: [Uint8Array, Uint8Array][] = [
  [
    Uint8Array.from([0xbd]),
    Uint8Array.from([
      0x5a, 0x50, 0x2f, 0x9f, 0xca, 0x46, 0x7b, 0x26, 0x6d, 0x5b, 0x78, 0x33, 0x65, 0x19, 0x37, 0xe8, 0x05, 0x27, 0x0c,
      0xa3, 0xf3, 0xaf, 0x1c, 0x0d, 0xd2, 0x46, 0x2d, 0xca, 0x4b, 0x3b, 0x1a, 0xbf,
    ]),
  ],
];

export const ecdsa_secp256k1_test_cases: [[Uint8Array, Uint8Array, Uint8Array, Uint8Array], boolean][] = [
  [
    [
      // hashed message
      Uint8Array.from([
        0x3a, 0x73, 0xf4, 0x12, 0x3a, 0x5c, 0xd2, 0x12, 0x1f, 0x21, 0xcd, 0x7e, 0x8d, 0x35, 0x88, 0x35, 0x47, 0x69,
        0x49, 0xd0, 0x35, 0xd9, 0xc2, 0xda, 0x68, 0x06, 0xb4, 0x63, 0x3a, 0xc8, 0xc1, 0xe2,
      ]),
      // pubkey x
      Uint8Array.from([
        0xa0, 0x43, 0x4d, 0x9e, 0x47, 0xf3, 0xc8, 0x62, 0x35, 0x47, 0x7c, 0x7b, 0x1a, 0xe6, 0xae, 0x5d, 0x34, 0x42,
        0xd4, 0x9b, 0x19, 0x43, 0xc2, 0xb7, 0x52, 0xa6, 0x8e, 0x2a, 0x47, 0xe2, 0x47, 0xc7,
      ]),
      // pubkey y
      Uint8Array.from([
        0x89, 0x3a, 0xba, 0x42, 0x54, 0x19, 0xbc, 0x27, 0xa3, 0xb6, 0xc7, 0xe6, 0x93, 0xa2, 0x4c, 0x69, 0x6f, 0x79,
        0x4c, 0x2e, 0xd8, 0x77, 0xa1, 0x59, 0x3c, 0xbe, 0xe5, 0x3b, 0x03, 0x73, 0x68, 0xd7,
      ]),
      // signature
      Uint8Array.from([
        0xe5, 0x08, 0x1c, 0x80, 0xab, 0x42, 0x7d, 0xc3, 0x70, 0x34, 0x6f, 0x4a, 0x0e, 0x31, 0xaa, 0x2b, 0xad, 0x8d,
        0x97, 0x98, 0xc3, 0x80, 0x61, 0xdb, 0x9a, 0xe5, 0x5a, 0x4e, 0x8d, 0xf4, 0x54, 0xfd, 0x28, 0x11, 0x98, 0x94,
        0x34, 0x4e, 0x71, 0xb7, 0x87, 0x70, 0xcc, 0x93, 0x1d, 0x61, 0xf4, 0x80, 0xec, 0xbb, 0x0b, 0x89, 0xd6, 0xeb,
        0x69, 0x69, 0x01, 0x61, 0xe4, 0x9a, 0x71, 0x5f, 0xcd, 0x55,
      ]),
    ],
    true,
  ],
  [
    [
      // hashed message
      Uint8Array.from(Array(32).fill(0)),
      // pubkey x
      Uint8Array.from(Array(32).fill(0)),
      // pubkey y
      Uint8Array.from(Array(32).fill(0)),
      // signature
      Uint8Array.from(Array(64).fill(0)),
    ],
    false,
  ],
];

export const ecdsa_secp256r1_test_cases: [[Uint8Array, Uint8Array, Uint8Array, Uint8Array], boolean][] = [
  [
    [
      // hashed message
      Uint8Array.from([
        84, 112, 91, 163, 186, 175, 219, 223, 186, 140, 95, 154, 112, 247, 168, 155, 238, 152, 217, 6, 181, 62, 49, 7,
        77, 167, 186, 236, 220, 13, 169, 173,
      ]),
      // pubkey x
      Uint8Array.from([
        85, 15, 71, 16, 3, 243, 223, 151, 195, 223, 80, 106, 199, 151, 246, 114, 31, 177, 161, 251, 123, 143, 111, 131,
        210, 36, 73, 138, 101, 200, 142, 36,
      ]),
      // pubkey y
      Uint8Array.from([
        19, 96, 147, 215, 1, 46, 80, 154, 115, 113, 92, 189, 11, 0, 163, 204, 15, 244, 181, 192, 27, 63, 250, 25, 106,
        177, 251, 50, 112, 54, 184, 230,
      ]),
      // signature
      Uint8Array.from([
        44, 112, 168, 208, 132, 182, 43, 252, 92, 224, 54, 65, 202, 249, 247, 42, 212, 218, 140, 129, 191, 230, 236,
        148, 135, 187, 94, 27, 239, 98, 161, 50, 24, 173, 158, 226, 158, 175, 53, 31, 220, 80, 241, 82, 12, 66, 94, 155,
        144, 138, 7, 39, 139, 67, 176, 236, 123, 135, 39, 120, 193, 78, 7, 132,
      ]),
    ],
    true,
  ],
  [
    [
      // hashed message
      Uint8Array.from(Array(32).fill(0)),
      // pubkey x
      Uint8Array.from(Array(32).fill(0)),
      // pubkey y
      Uint8Array.from(Array(32).fill(0)),
      // signature
      Uint8Array.from(Array(64).fill(0)),
    ],
    false,
  ],
];
