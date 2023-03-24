/**
 * Yeeter ABI for viem.
 */
export const YeeterAbi = [
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
    name: 'ContractDeploymentYeet',
    type: 'event',
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: true,
        internalType: 'uint256',
        name: 'l2blockNum',
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
        name: 'blabber',
        type: 'bytes',
      },
    ],
    name: 'Yeet',
    type: 'event',
  },
  {
    inputs: [
      {
        internalType: 'uint256',
        name: '_l2blockNum',
        type: 'uint256',
      },
      {
        internalType: 'bytes',
        name: '_blabber',
        type: 'bytes',
      },
    ],
    name: 'yeet',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
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
    name: 'yeetContractDeployment',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
] as const;
