import { bufferAsFields, bufferFromFields } from './buffer.js';

describe('buffer', () => {
  it('converts buffer back and forth from fields', () => {
    const buffer = Buffer.from('1234567890abcdef'.repeat(10), 'hex');
    const fields = bufferAsFields(buffer, 20);
    expect(bufferFromFields(fields).toString('hex')).toEqual(buffer.toString('hex'));
  });

  it('throws if max length is exceeded', () => {
    const buffer = Buffer.from('1234567890abcdef'.repeat(10), 'hex');
    expect(() => bufferAsFields(buffer, 3)).toThrow(/exceeds maximum size/);
  });
});
