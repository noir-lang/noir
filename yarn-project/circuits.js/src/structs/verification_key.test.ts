import { VerificationKey } from './verification_key.js';

describe('structs/verification_key', () => {
  // The VK below was grabbed from a Noir build output, which is currently failing to deserialize
  // apparently due to a mismatch in the VK format used across teams. Once Noir moves to the same
  // format we're using (or the other way around), we can grab a new VK from a build artifact,
  // paste it here, and check that it deserializes properly.
  it.skip(`can deserialize vk built by noir`, () => {
    const serialized = `0000000100000100000000300000000b00000003515f310c94cc788117a46753f05f3ca63659fe689529657338e0ac526d0b405a26d4090b00afd09d53eb6e37a91f7e7eba68d98653aca455d7f05003a8d9fcf5c75fff00000003515f3206852757d96b9a6063ba1f3a37dc369df432f440c63f452976d86f962b64761b2e313905d40943c8f134ec0faa000c9facc670d2acaa3affa69941ff0a3e5f3f00000003515f330709e3c949775037b37630dd2845ecacb5c4da4d04d094f65414edebe9292b262e2f9a2189107a99dad5259914f87419e4fc9c35023e8b9694f901c53cfc99c800000003515f3406852757d96b9a6063ba1f3a37dc`;
    const vk = VerificationKey.fromBuffer(Buffer.from(serialized, 'hex'));
    expect(vk.circuitSize).toBeGreaterThan(100);
  });
});
