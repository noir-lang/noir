/**
 * Rollup ABI for viem.
 */
export const RollupAbi = [
  {
    inputs: [],
    stateMutability: 'nonpayable',
    type: 'constructor',
  },
  {
    inputs: [],
    name: 'InvalidProof',
    type: 'error',
  },
  {
    inputs: [
      {
        internalType: 'bytes32',
        name: 'expected',
        type: 'bytes32',
      },
      {
        internalType: 'bytes32',
        name: 'actual',
        type: 'bytes32',
      },
    ],
    name: 'InvalidStateHash',
    type: 'error',
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: 'uint256',
        name: 'blockNum',
        type: 'uint256',
      },
    ],
    name: 'L2BlockProcessed',
    type: 'event',
  },
  {
    inputs: [],
    name: 'VERIFIER',
    outputs: [
      {
        internalType: 'contract MockVerifier',
        name: '',
        type: 'address',
      },
    ],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [
      {
        internalType: 'bytes',
        name: '_proof',
        type: 'bytes',
      },
      {
        internalType: 'bytes',
        name: '_l2Block',
        type: 'bytes',
      },
    ],
    name: 'process',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [],
    name: 'rollupStateHash',
    outputs: [
      {
        internalType: 'bytes32',
        name: '',
        type: 'bytes32',
      },
    ],
    stateMutability: 'view',
    type: 'function',
  },
] as const;
