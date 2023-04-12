export function hexStringToBuffer(hex: string): Buffer {
  if (!/^(0x)?[a-fA-F0-9]+$/.test(hex)) throw new Error(`Invalid format for hex string: "${hex}"`);
  if (hex.length % 2 === 1) throw new Error(`Invalid length for hex string: "${hex}"`);
  return Buffer.from(hex.replace(/^0x/, ''), 'hex');
}

export function sleep<T>(ms: number, retval: T): Promise<T> {
  return new Promise(resolve => setTimeout(() => resolve(retval), ms));
}

/**
 * Returns the lowest power of two that is greater of equal to the input.
 * @param num - The input.
 */
export function ceilPowerOfTwo(num: number): number {
  return 2 ** Math.ceil(Math.log2(num));
}
