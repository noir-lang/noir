import { randomBytes } from '../crypto/index.js';
import { from2Fields, fromTruncField, to2Fields, toTruncField } from './free_funcs.js';

describe('buffer to fields and back', () => {
  it('should correctly serialize and deserialize a buffer to two fields', () => {
    // Generate a random 32-byte buffer
    const originalBuffer = randomBytes(32);

    // Serialize the buffer to two fields
    const [field1, field2] = to2Fields(originalBuffer);

    // Deserialize the fields back to a buffer
    const reconstructedBuffer = from2Fields(field1, field2);

    // Check if the original buffer and reconstructed buffer are identical
    expect(reconstructedBuffer).toEqual(originalBuffer);
  });

  it('should correctly serialize and deserialize a buffer to one truncated field', () => {
    // Generate a random 31-byte buffer padded to 32
    const originalBuffer = Buffer.concat([Buffer.alloc(1), randomBytes(31)]);

    // Serialize the buffer to one field
    const field = toTruncField(originalBuffer);

    // Deserialize the field back to a buffer
    const reconstructedBuffer = fromTruncField(field);

    // Check if the original buffer and reconstructed buffer are identical
    expect(reconstructedBuffer).toEqual(originalBuffer);
  });
});
