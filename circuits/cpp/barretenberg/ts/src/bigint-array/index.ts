export function toBigIntBE(bytes: Uint8Array) {
  // A Buffer in node, *is* a Uint8Array. We can't refuse it's type.
  // However the algo below only works on an actual Uint8Array, hence we make a new one to be safe.
  bytes = new Uint8Array(bytes);
  let bigint = BigInt(0);
  const view = new DataView(bytes.buffer);
  for (let i = 0; i < bytes.byteLength; i++) {
    bigint = (bigint << BigInt(8)) + BigInt(view.getUint8(i));
  }
  return bigint;
}

export function toBufferBE(value: bigint, byteLength = 32) {
  const bytes = new Uint8Array(byteLength);
  const view = new DataView(bytes.buffer);
  for (let i = 0; i < byteLength; i++) {
    view.setUint8(byteLength - i - 1, Number(value & BigInt(0xff)));
    value >>= BigInt(8);
  }
  return bytes;
}
