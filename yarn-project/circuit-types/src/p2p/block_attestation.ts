import { Header } from '@aztec/circuits.js';
import { BaseHashType } from '@aztec/foundation/hash';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { Gossipable } from './gossipable.js';
import { TopicType, createTopicString } from './topic_type.js';

export class BlockAttestationHash extends BaseHashType {
  constructor(hash: Buffer) {
    super(hash);
  }
}

/**
 * BlockAttestation
 *
 * A validator that has attested to seeing the contents of a block
 * will produce a block attestation over the header of the block
 */
export class BlockAttestation extends Gossipable {
  static override p2pTopic: string;

  constructor(
    /** The block header the attestation is made over */
    public readonly header: Header,
    /** The signature of the block attester */
    public readonly signature: Buffer,
  ) {
    super();
  }

  static {
    this.p2pTopic = createTopicString(TopicType.block_attestation);
  }

  override p2pMessageIdentifier(): BaseHashType {
    return BlockAttestationHash.fromField(this.header.hash());
  }

  toBuffer(): Buffer {
    return serializeToBuffer([this.header, this.signature.length, this.signature]);
  }

  static fromBuffer(buf: Buffer | BufferReader): BlockAttestation {
    const reader = BufferReader.asReader(buf);
    return new BlockAttestation(reader.readObject(Header), reader.readBuffer());
  }
}
