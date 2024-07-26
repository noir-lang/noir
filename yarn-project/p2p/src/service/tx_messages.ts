import { type Tx } from '@aztec/circuit-types';

import { type SemVer } from 'semver';

export const TX_MESSAGE_TOPIC = '';

export class AztecTxMessageCreator {
  private readonly topic: string;
  constructor(version: SemVer) {
    this.topic = `/aztec/tx/${version.toString()}`;
  }

  createTxMessage(tx: Tx) {
    const messageData = tx.toBuffer();

    return { topic: this.topic, data: messageData };
  }

  getTopic() {
    return this.topic;
  }
}
