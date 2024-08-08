export function extractECDSAPublicKeyFromBase64String(base64PublicKey: string): Buffer {
  const buffer = Buffer.from(base64PublicKey, 'base64');
  let keyOffset = 0;
  const typeLen = buffer.readUInt32BE(keyOffset);
  keyOffset += 4;
  keyOffset += typeLen;

  const curveLen = buffer.readUInt32BE(keyOffset);
  keyOffset += 4;
  keyOffset += curveLen;

  const keyLen = buffer.readUInt32BE(keyOffset);
  keyOffset += 5; // 4+1 to remove the prefix
  return buffer.subarray(keyOffset, keyOffset + keyLen - 1);
}
