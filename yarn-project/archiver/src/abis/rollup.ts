export const rollupAbi = [
  {
    name: 'L2BlockProcessed',
    type: 'event',
    inputs: [{ type: 'uint256', name: 'blockNum', indexed: true }],
  },
  {
    inputs: [],
    name: 'nextBlockNum',
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
] as const;
