import { EthAddress, Header } from '@aztec/circuits.js';
import { Buffer32 } from '@aztec/foundation/buffer';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { recoverMessageAddress } from 'viem';

import { TxHash } from '../tx/tx_hash.js';
import { Gossipable } from './gossipable.js';
import { Signature } from './signature.js';
import { TopicType, createTopicString } from './topic_type.js';

export class BlockProposalHash extends Buffer32 {
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

  private sender: EthAddress | undefined;

  constructor(
    /** The block header, after execution of the below sequence of transactions */
    public readonly header: Header,

    // TODO(https://github.com/AztecProtocol/aztec-packages/pull/7727#discussion_r1713670830): temporary
    public readonly archive: Fr,
    /** The sequence of transactions in the block */
    public readonly txs: TxHash[],
    /** The signer of the BlockProposal over the header of the new block*/
    public readonly signature: Signature,
  ) {
    super();
  }

  static {
    this.p2pTopic = createTopicString(TopicType.block_proposal);
  }

  override p2pMessageIdentifier(): Buffer32 {
    return BlockProposalHash.fromField(this.archive);
  }

  /**Get Sender
   * Lazily evaluate the sender of the proposal; result is cached
   */
  async getSender() {
    if (!this.sender) {
      // performance note(): this signature method requires another hash behind the scenes
      const address = await recoverMessageAddress({
        message: { raw: this.p2pMessageIdentifier().to0xString() },
        signature: this.signature.to0xString(),
      });
      // Cache the sender for later use
      this.sender = EthAddress.fromString(address);
    }

    return this.sender;
  }

  toBuffer(): Buffer {
    return serializeToBuffer([this.header, this.archive, this.txs.length, this.txs, this.signature]);
  }

  static fromBuffer(buf: Buffer | BufferReader): BlockProposal {
    const reader = BufferReader.asReader(buf);
    return new BlockProposal(
      reader.readObject(Header),
      reader.readObject(Fr),
      reader.readArray(reader.readNumber(), TxHash),
      reader.readObject(Signature),
    );
  }
}
