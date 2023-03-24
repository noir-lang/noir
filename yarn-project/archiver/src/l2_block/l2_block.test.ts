import { mockRandomL2Block } from '../index.js';
import { L2Block } from './l2_block.js';

describe('L2Block', () => {
  it('can encode a L2 block data object to buffer and back', () => {
    const block = mockRandomL2Block(42);

    const buffer = block.encode();
    const recovered = L2Block.decode(buffer);

    expect(recovered).toEqual(block);
  });
});
