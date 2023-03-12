import { keccak256 } from '../crypto/index.js';

export function hashMessage(data: Buffer) {
  const preamble = '\x19Ethereum Signed Message:\n' + data.length;
  const preambleBuffer = Buffer.from(preamble);
  const ethMessage = Buffer.concat([preambleBuffer, data]);
  return keccak256(ethMessage);
}
