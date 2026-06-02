const circuit = {
  noir_version: '0.33.0+325dac54efb6f99201de9fdeb0a507d45189607d',
  hash: 13910613032272493000,
  abi: {
    parameters: [
      {
        name: 'device_secret',
        type: {
          kind: 'field',
        },
        visibility: 'private',
      },
      {
        name: 'commitment',
        type: {
          kind: 'field',
        },
        visibility: 'public',
      },
      {
        name: 'challenge',
        type: {
          kind: 'field',
        },
        visibility: 'public',
      },
      {
        name: 'user_id_hash',
        type: {
          kind: 'field',
        },
        visibility: 'public',
      },
    ],
    return_type: {
      abi_type: {
        kind: 'field',
      },
      visibility: 'public',
    },
    error_types: {},
  },
  bytecode:
    'H4sIAAAAAAAA/62SwQ3DMAhFYycDgYEYbl2lVp39R2irEtVKcgtPspA5fMETafqxfF6e/iSvD69wD9xz8yGXYGXutXQkfEKxpgIsbVVUFJVXUaKurNWaVTBk6riJ0ebBKXDGOS4Lrhze3XX3F+0wcG9cAh0mv8vR5Zd56GX/p+GOT7wB254FMOUCAAA=',
  debug_symbols:
    'NYxJCoAwDEXvkrULFRzoVUQkapVCSUsHQUrvbqq4++9PCXa5xnNRdBgPYkqgzYZBGWJKUL+Wt0iFfEAXQLTtWIGknVXf5AoOpSWIbsgzw4VO4aplmZcs0va/MYbbfgl3Hw==',
  file_map: {
    57: {
      source:
        'fn main(\r\n    device_secret: Field,\r\n    commitment: pub Field,\r\n    challenge: pub Field,\r\n    user_id_hash: pub Field,\r\n) -> pub Field {\r\n    let computed_commitment = device_secret * device_secret + user_id_hash;\r\n    assert(computed_commitment == commitment);\r\n\r\n    device_secret * challenge + user_id_hash\r\n}\r\n',
      path: '/src/main.nr',
    },
  },
  names: ['main'],
};

export default circuit;
