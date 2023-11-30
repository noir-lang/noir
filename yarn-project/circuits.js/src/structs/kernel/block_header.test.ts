import { BlockHeader } from './block_header.js';

describe('BlockHeader', () => {
  it('serializes to buffer and back', () => {
    const blockHeader = BlockHeader.random();
    const serialized = blockHeader.toBuffer();
    const deserialized = BlockHeader.fromBuffer(serialized);
    expect(deserialized).toEqual(blockHeader);
  });

  it('serializes to string and back', () => {
    const blockHeader = BlockHeader.random();
    const serialized = blockHeader.toString();
    const deserialized = BlockHeader.fromString(serialized);
    expect(deserialized).toEqual(blockHeader);
  });
});
