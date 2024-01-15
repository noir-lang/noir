import { randomBytes } from '../crypto/index.js';
import { from2Fields, to2Fields } from './free_funcs.js';

describe('buffer to fields and back', () => {
  it('should correctly serialize and deserialize a buffer', () => {
    // Generate a random 32-byte buffer
    const originalBuffer = randomBytes(32);

    // Serialize the buffer to two fields
    const [field1, field2] = to2Fields(originalBuffer);

    // Deserialize the fields back to a buffer
    const reconstructedBuffer = from2Fields(field1, field2);

    // Check if the original buffer and reconstructed buffer are identical
    expect(reconstructedBuffer).toEqual(originalBuffer);
  });
});
