import { ContractAbi } from '@aztec/ethereum.js/contract';
export default new ContractAbi([
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
        name: 'rollupBlockNumber',
        type: 'uint256',
      },
    ],
    name: 'RollupBlockProcessed',
    type: 'event',
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
        name: '_inputs',
        type: 'bytes',
      },
    ],
    name: 'processRollup',
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
  {
    inputs: [],
    name: 'verifier',
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
]);
