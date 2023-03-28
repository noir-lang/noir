import { L2Block } from './l2_block.js';

describe('L2Block', () => {
  it('can encode a L2 block data object to buffer and back', () => {
    const block = L2Block.random(42);

    const buffer = block.encode();
    const recovered = L2Block.decode(buffer);

    expect(recovered).toEqual(block);
  });
});
