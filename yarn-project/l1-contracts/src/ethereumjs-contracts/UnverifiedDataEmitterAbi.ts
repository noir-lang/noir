import { ContractAbi } from '@aztec/ethereum.js/contract';
export default new ContractAbi([
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: 'bytes32',
        name: 'aztecAddress',
        type: 'bytes32',
      },
      {
        indexed: true,
        internalType: 'address',
        name: 'portalAddress',
        type: 'address',
      },
      {
        indexed: false,
        internalType: 'bytes',
        name: 'acir',
        type: 'bytes',
      },
    ],
    name: 'ContractDeployment',
    type: 'event',
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: 'uint256',
        name: 'l2BlockNum',
        type: 'uint256',
      },
      {
        indexed: true,
        internalType: 'address',
        name: 'sender',
        type: 'address',
      },
      {
        indexed: false,
        internalType: 'bytes',
        name: 'data',
        type: 'bytes',
      },
    ],
    name: 'UnverifiedData',
    type: 'event',
  },
  {
    inputs: [
      {
        internalType: 'bytes32',
        name: '_aztecAddress',
        type: 'bytes32',
      },
      {
        internalType: 'address',
        name: '_portalAddress',
        type: 'address',
      },
      {
        internalType: 'bytes',
        name: '_acir',
        type: 'bytes',
      },
    ],
    name: 'emitContractDeployment',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [
      {
        internalType: 'uint256',
        name: '_l2BlockNum',
        type: 'uint256',
      },
      {
        internalType: 'bytes',
        name: '_data',
        type: 'bytes',
      },
    ],
    name: 'emitUnverifiedData',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
]);
