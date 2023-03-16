import { parseAbiItem } from 'viem';

export const yeeterAbi = [
  parseAbiItem('event Yeet(uint256 indexed blockNum, address indexed sender, bytes blabber)'),
] as const;
