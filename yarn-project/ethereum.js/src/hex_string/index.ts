export function bufferToHex(b: Buffer) {
  return '0x' + b.toString('hex');
}

export function hexToBuffer(h: string) {
  return Buffer.from((h.length % 2 ? '0' : '') + h.replace(/^0x/, ''), 'hex');
}

export function numberToHex(n: number) {
  return '0x' + n.toString(16);
}

export function hexToNumber(h: string) {
  return Number(h);
}

export function bigIntToHex(n: bigint) {
  return '0x' + n.toString(16);
}

export function hexToBigInt(h: string) {
  return BigInt(h);
}
