/**
 * Convert a buffer to a hex-encoded string with a '0x' prefix.
 * The conversion is done by first converting the buffer data to a hexadecimal string
 * and then appending the '0x' prefix to it. This function can be used to convert
 * binary data into a human-readable format that can be easily manipulated or displayed.
 *
 * @param b - The buffer object containing the binary data to be converted.
 * @returns A hex-encoded string with a '0x' prefix representing the input buffer data.
 */
export function bufferToHex(b: Buffer) {
  return '0x' + b.toString('hex');
}

/**
 * Converts a hex-encoded string to a Buffer object.
 * The input 'h' can be prefixed with '0x' or not, and may have an odd or even number of hex characters.
 * If the input length is odd, a leading '0' will be added before conversion.
 *
 * @param h - The hex-encoded string to be converted to a Buffer.
 * @returns A Buffer object representing the input hex-encoded string.
 */
export function hexToBuffer(h: string) {
  return Buffer.from((h.length % 2 ? '0' : '') + h.replace(/^0x/, ''), 'hex');
}

/**
 * Convert a given number to its hexadecimal representation as a string.
 * The output hex string will be prefixed with '0x'.
 *
 * @param n - The number to be converted to hex.
 * @returns The hexadecimal representation of the input number as a string.
 */
export function numberToHex(n: number) {
  return '0x' + n.toString(16);
}

/**
 * Convert a hex-encoded string to its equivalent number representation.
 * The input 'h' should be prefixed with '0x' or not, and consist of valid hex characters.
 * Note that the result might lose precision for large hex values due to JavaScript's number limitations.
 *
 * @param h - The hex-encoded string to convert to a number.
 * @returns The numeric representation of the input hex string.
 */
export function hexToNumber(h: string) {
  return Number(h);
}

/**
 * Converts a BigInt value to its corresponding hexadecimal representation.
 * The output string will be prefixed with '0x' to indicate hexadecimal notation.
 *
 * @param n - The BigInt value to be converted to hexadecimal.
 * @returns The hexadecimal representation of the input BigInt value as a string.
 */
export function bigIntToHex(n: bigint) {
  return '0x' + n.toString(16);
}

/**
 * Convert a hex-encoded string to a BigInt.
 * The input 'h' can be prefixed with '0x' or not, and it should have an even number of hex characters.
 * If the input is not valid, this function will throw a RangeError.
 *
 * @param h - The hex-encoded string representing the input value.
 * @returns A BigInt representation of the input hex value.
 */
export function hexToBigInt(h: string) {
  return BigInt(h);
}
