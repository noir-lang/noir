/**
 * UnverifiedDataEmitter ABI for viem.
 */
export const UnverifiedDataEmitterAbi = [
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
        internalType: 'uint256',
        name: '_l2BlockNum',
        type: 'uint256',
      },
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
] as const;
/**
 * UnverifiedDataEmitter Bytecode for viem.
 */
export const UnverifiedDataEmitterBytecode =
  '0x608060405234801561001057600080fd5b50610268806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80631beadbf61461003b578063ec1c72ff14610050575b600080fd5b61004e610049366004610141565b610063565b005b61004e61005e3660046101b7565b6100af565b826001600160a01b031684867fc6da1a507344f9e421450cb8075906fe777b6411c1918306f6018ebb4d6b7e3785856040516100a0929190610203565b60405180910390a45050505050565b336001600160a01b0316837fb72b85ef2a843244f5ea1955248b0ac363732d2e6a98cc3641084dd5718ad8b584846040516100eb929190610203565b60405180910390a3505050565b60008083601f84011261010a57600080fd5b50813567ffffffffffffffff81111561012257600080fd5b60208301915083602082850101111561013a57600080fd5b9250929050565b60008060008060006080868803121561015957600080fd5b853594506020860135935060408601356001600160a01b038116811461017e57600080fd5b9250606086013567ffffffffffffffff81111561019a57600080fd5b6101a6888289016100f8565b969995985093965092949392505050565b6000806000604084860312156101cc57600080fd5b83359250602084013567ffffffffffffffff8111156101ea57600080fd5b6101f6868287016100f8565b9497909650939450505050565b60208152816020820152818360408301376000818301604090810191909152601f909201601f1916010191905056fea2646970667358221220444c01fee3697b99865e7d7c596fde10e007e4e7e2a1ea003b06c9290c8b67c564736f6c63430008120033';
