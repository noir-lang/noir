import { Tx } from '../tx/tx.js';
import { BlockAttestation } from './block_attestation.js';
import { BlockProposal } from './block_proposal.js';
import { type Gossipable } from './gossipable.js';
import { TopicType } from './topic_type.js';

export interface RawGossipMessage {
  topic: string;
  data: Uint8Array;
}

// Force casts as we know that each field here extends Gossipable, and we just want types from Gossipable
export const TopicTypeMap: Record<string, typeof Gossipable> = {
  [TopicType.tx]: Tx as unknown as typeof Gossipable,
  [TopicType.block_proposal]: BlockProposal as unknown as typeof Gossipable,
  [TopicType.block_attestation]: BlockAttestation as unknown as typeof Gossipable,
};
