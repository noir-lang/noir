import { ContractAbi } from '@aztec/ethereum.js/contract';
export default new ContractAbi([
  {
    inputs: [
      {
        internalType: 'bytes',
        name: '_l2Block',
        type: 'bytes',
      },
    ],
    name: 'computeDiffRootAndMessagesHash',
    outputs: [
      {
        internalType: 'bytes32',
        name: '',
        type: 'bytes32',
      },
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
    inputs: [
      {
        internalType: 'bytes',
        name: '_l2Block',
        type: 'bytes',
      },
    ],
    name: 'decode',
    outputs: [
      {
        internalType: 'uint256',
        name: '',
        type: 'uint256',
      },
      {
        internalType: 'bytes32',
        name: '',
        type: 'bytes32',
      },
      {
        internalType: 'bytes32',
        name: '',
        type: 'bytes32',
      },
      {
        internalType: 'bytes32',
        name: '',
        type: 'bytes32',
      },
    ],
    stateMutability: 'view',
    type: 'function',
  },
]);
