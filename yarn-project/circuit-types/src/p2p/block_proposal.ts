import { Header } from '@aztec/circuits.js';
import { BaseHashType } from '@aztec/foundation/hash';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { TxHash } from '../index.js';
import { Gossipable } from './gossipable.js';
import { TopicType, createTopicString } from './topic_type.js';

export class BlockProposalHash extends BaseHashType {
  constructor(hash: Buffer) {
    super(hash);
  }
}

/**
 * BlockProposal
 *
 * A block proposal is created by the leader of the chain proposing a sequence of transactions to
 * be included in the head of the chain
 */
export class BlockProposal extends Gossipable {
  static override p2pTopic: string;

  constructor(
    /** The block header, after execution of the below sequence of transactions */
    public readonly header: Header,
    /** The sequence of transactions in the block */
    public readonly txs: TxHash[],
    /** The signer of the BlockProposal over the header of the new block*/
    public readonly signature: Buffer,
  ) {
    super();
  }

  static {
    this.p2pTopic = createTopicString(TopicType.block_proposal);
  }

  override p2pMessageIdentifier(): BaseHashType {
    return BlockProposalHash.fromField(this.header.hash());
  }

  toBuffer(): Buffer {
    return serializeToBuffer([this.header, this.txs.length, this.txs, this.signature.length, this.signature]);
  }

  static fromBuffer(buf: Buffer | BufferReader): BlockProposal {
    const reader = BufferReader.asReader(buf);
    return new BlockProposal(
      reader.readObject(Header),
      reader.readArray(reader.readNumber(), TxHash),
      reader.readBuffer(),
    );
  }
}
