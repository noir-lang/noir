export function hexToBuffer(h: string) {
  return Buffer.from((h.length % 2 ? '0' : '') + h.replace(/^0x/, ''), 'hex');
}
