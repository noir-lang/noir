import {
  EncryptedL2BlockL2Logs,
  EncryptedNoteL2BlockL2Logs,
  TxEffect,
  UnencryptedL2BlockL2Logs,
} from '@aztec/circuit-types';
import { padArrayEnd } from '@aztec/foundation/collection';
import { sha256Trunc } from '@aztec/foundation/crypto';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { inspect } from 'util';

export class Body {
  constructor(public txEffects: TxEffect[]) {
    txEffects.forEach(txEffect => {
      if (txEffect.isEmpty()) {
        throw new Error('Empty tx effect not allowed in Body');
      }
    });
  }

  /**
   * Serializes a block body
   * @returns A serialized L2 block body.
   */
  toBuffer() {
    return serializeToBuffer(this.txEffects.length, this.txEffects);
  }

  /**
   * Deserializes a block from a buffer
   * @returns A deserialized L2 block.
   */
  static fromBuffer(buf: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buf);

    return new this(reader.readVector(TxEffect));
  }

  [inspect.custom]() {
    return `Body {
  txEffects: ${inspect(this.txEffects)},
  emptyTxEffectsCount: ${this.numberOfTxsIncludingPadded},
  emptyTxEffectHash: ${TxEffect.empty().hash().toString('hex')},
  txsEffectsHash: ${this.getTxsEffectsHash().toString('hex')},
}`;
  }

  /**
   * Computes the transactions effects hash for the L2 block
   * This hash is also computed in the `AvailabilityOracle`.
   * @returns The txs effects hash.
   */
  getTxsEffectsHash() {
    // Adapted from proving-state.ts -> findMergeLevel and unbalanced_tree.ts
    // Calculates the tree upwards layer by layer until we reach the root
    // The L1 calculation instead computes the tree from right to left (slightly cheaper gas)
    // TODO: A more thorough investigation of which method is cheaper, then use that method everywhere
    const computeRoot = (leaves: Buffer[]): Buffer => {
      const depth = Math.ceil(Math.log2(leaves.length));
      let [layerWidth, nodeToShift] =
        leaves.length & 1 ? [leaves.length - 1, leaves[leaves.length - 1]] : [leaves.length, Buffer.alloc(0)];
      // Allocate this layer's leaves and init the next layer up
      let thisLayer = leaves.slice(0, layerWidth);
      let nextLayer = [];
      for (let i = 0; i < depth; i++) {
        for (let j = 0; j < layerWidth; j += 2) {
          // Store the hash of each pair one layer up
          nextLayer[j / 2] = sha256Trunc(Buffer.concat([thisLayer[j], thisLayer[j + 1]]));
        }
        layerWidth /= 2;
        if (layerWidth & 1) {
          if (nodeToShift.length) {
            // If the next layer has odd length, and we have a node that needs to be shifted up, add it here
            nextLayer.push(nodeToShift);
            layerWidth += 1;
            nodeToShift = Buffer.alloc(0);
          } else {
            // If we don't have a node waiting to be shifted, store the next layer's final node to be shifted
            layerWidth -= 1;
            nodeToShift = nextLayer[layerWidth];
          }
        }
        // reset the layers
        thisLayer = nextLayer;
        nextLayer = [];
      }
      // return the root
      return thisLayer[0];
    };

    const emptyTxEffectHash = TxEffect.empty().hash();
    let leaves: Buffer[] = this.txEffects.map(txEffect => txEffect.hash());
    if (leaves.length < 2) {
      leaves = padArrayEnd(leaves, emptyTxEffectHash, 2);
    }
    return computeRoot(leaves);
  }

  get noteEncryptedLogs(): EncryptedNoteL2BlockL2Logs {
    const logs = this.txEffects.map(txEffect => txEffect.noteEncryptedLogs);

    return new EncryptedNoteL2BlockL2Logs(logs);
  }

  get encryptedLogs(): EncryptedL2BlockL2Logs {
    const logs = this.txEffects.map(txEffect => txEffect.encryptedLogs);

    return new EncryptedL2BlockL2Logs(logs);
  }

  get unencryptedLogs(): UnencryptedL2BlockL2Logs {
    const logs = this.txEffects.map(txEffect => txEffect.unencryptedLogs);

    return new UnencryptedL2BlockL2Logs(logs);
  }

  /**
   * Computes the number of transactions in the block including padding transactions.
   * @dev Modified code from TxsDecoder.computeNumTxEffectsToPad
   */
  get numberOfTxsIncludingPadded() {
    const numTxEffects = this.txEffects.length;

    // 2 is the minimum number of tx effects
    if (numTxEffects <= 2) {
      return 2;
    }

    return numTxEffects;
  }

  static random(
    txsPerBlock = 4,
    numPrivateCallsPerTx = 2,
    numPublicCallsPerTx = 3,
    numEncryptedLogsPerCall = 2,
    numUnencryptedLogsPerCall = 1,
  ) {
    const txEffects = [...new Array(txsPerBlock)].map(_ =>
      TxEffect.random(numPrivateCallsPerTx, numPublicCallsPerTx, numEncryptedLogsPerCall, numUnencryptedLogsPerCall),
    );

    return new Body(txEffects);
  }

  static empty() {
    return new Body([]);
  }
}
