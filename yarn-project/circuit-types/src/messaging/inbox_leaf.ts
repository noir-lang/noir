import { Fr } from '@aztec/circuits.js';
import { toBigIntBE } from '@aztec/foundation/bigint-buffer';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

export class InboxLeaf {
  constructor(
    /** L2 block number in which the message will be included. */
    public readonly blockNumber: bigint,
    /** Index of the leaf in L2 block message subtree. */
    public readonly index: bigint,
    /** Leaf in the subtree/message hash. */
    public readonly leaf: Fr,
  ) {}

  toBuffer(): Buffer {
    return serializeToBuffer([this.blockNumber, this.index, this.leaf]);
  }

  fromBuffer(buffer: Buffer | BufferReader): InboxLeaf {
    const reader = BufferReader.asReader(buffer);
    const blockNumber = toBigIntBE(reader.readBytes(32));
    const index = toBigIntBE(reader.readBytes(32));
    const leaf = reader.readObject(Fr);
    return new InboxLeaf(blockNumber, index, leaf);
  }
}
