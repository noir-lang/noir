import { EncryptedL2BlockL2Logs, TxEffect, UnencryptedL2BlockL2Logs } from '@aztec/circuit-types';
import { padArrayEnd } from '@aztec/foundation/collection';
import { sha256 } from '@aztec/foundation/crypto';
import { BufferReader, serializeToBuffer, truncateAndPad } from '@aztec/foundation/serialize';

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
}`;
  }

  /**
   * Computes the transactions effects hash for the L2 block
   * This hash is also computed in the `AvailabilityOracle` and the `Circuit`.
   * @returns The txs effects hash.
   */
  getTxsEffectsHash() {
    const computeRoot = (leaves: Buffer[]): Buffer => {
      const layers: Buffer[][] = [leaves];
      let activeLayer = 0;

      while (layers[activeLayer].length > 1) {
        const layer: Buffer[] = [];
        const layerLength = layers[activeLayer].length;

        for (let i = 0; i < layerLength; i += 2) {
          const left = layers[activeLayer][i];
          const right = layers[activeLayer][i + 1];

          layer.push(truncateAndPad(sha256(Buffer.concat([left, right]))));
        }

        layers.push(layer);
        activeLayer++;
      }

      return layers[layers.length - 1][0];
    };

    const emptyTxEffectHash = TxEffect.empty().hash();
    const leaves: Buffer[] = padArrayEnd(
      this.txEffects.map(txEffect => txEffect.hash()),
      emptyTxEffectHash,
      this.numberOfTxsIncludingPadded,
    );

    return computeRoot(leaves);
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

    // Note that the following could be implemented in a more simple way as "2 ** Math.ceil(Math.log2(numTxEffects));"
    // but we want to keep the same logic as in Solidity and there we don't have the math functions.
    let v = numTxEffects;

    // The following rounds numTxEffects up to the next power of 2 (works only for 4 bytes value!)
    v--;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v++;

    return v;
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
