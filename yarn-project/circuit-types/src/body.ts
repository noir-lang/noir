import { ContractData, L2BlockL2Logs, PublicDataWrite, TxEffect } from '@aztec/circuit-types';
import {
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '@aztec/circuits.js';
import { sha256 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

export class Body {
  constructor(public l1ToL2Messages: Fr[], public txEffects: TxEffect[]) {}

  /**
   * Serializes a block body
   * @returns A serialized L2 block body.
   */
  toBuffer() {
    const newNoteHashes = this.txEffects.flatMap(txEffect => txEffect.newNoteHashes);
    const newNullifiers = this.txEffects.flatMap(txEffect => txEffect.newNullifiers);
    const newPublicDataWrites = this.txEffects.flatMap(txEffect => txEffect.newPublicDataWrites);
    const newL2ToL1Msgs = this.txEffects.flatMap(txEffect => txEffect.newL2ToL1Msgs);
    const newContracts = this.txEffects.flatMap(txEffect => txEffect.contractLeaves);
    const newContractData = this.txEffects.flatMap(txEffect => txEffect.contractData);
    const newL1ToL2Messages = this.l1ToL2Messages;
    const newEncryptedLogs = this.encryptedLogs;
    const newUnencryptedLogs = this.unencryptedLogs;

    return serializeToBuffer(
      newNoteHashes.length,
      newNoteHashes,
      newNullifiers.length,
      newNullifiers,
      newPublicDataWrites.length,
      newPublicDataWrites,
      newL2ToL1Msgs.length,
      newL2ToL1Msgs,
      newContracts.length,
      newContracts,
      newContractData,
      newL1ToL2Messages.length,
      newL1ToL2Messages,
      newEncryptedLogs,
      newUnencryptedLogs,
    );
  }

  /**
   * Deserializes a block from a buffer
   * @returns A deserialized L2 block.
   */
  static fromBuffer(buf: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buf);
    const newNoteHashes = reader.readVector(Fr);
    const newNullifiers = reader.readVector(Fr);
    const newPublicDataWrites = reader.readVector(PublicDataWrite);
    const newL2ToL1Msgs = reader.readVector(Fr);
    const newContracts = reader.readVector(Fr);
    const newContractData = reader.readArray(newContracts.length, ContractData);
    // TODO(sean): could an optimization of this be that it is encoded such that zeros are assumed
    // It seems the da/ tx hash would be fine, would only need to edit circuits ?
    const newL1ToL2Messages = reader.readVector(Fr);

    // Because TX's in a block are padded to nearest power of 2, this is finding the nearest nonzero tx filled with 1 nullifier
    const numberOfNonEmptyTxs = calculateNumTxsFromNullifiers(newNullifiers);

    const newEncryptedLogs = reader.readObject(L2BlockL2Logs);
    const newUnencryptedLogs = reader.readObject(L2BlockL2Logs);

    if (
      new L2BlockL2Logs(newEncryptedLogs.txLogs.slice(numberOfNonEmptyTxs)).getTotalLogCount() !== 0 ||
      new L2BlockL2Logs(newUnencryptedLogs.txLogs.slice(numberOfNonEmptyTxs)).getTotalLogCount() !== 0
    ) {
      throw new Error('Logs exist in the padded area');
    }

    const txEffects: TxEffect[] = [];

    const numberOfTxsIncludingEmpty = newNullifiers.length / MAX_NEW_NULLIFIERS_PER_TX;

    for (let i = 0; i < numberOfTxsIncludingEmpty; i += 1) {
      // TODO(#4720): this should use TxEffect.fromBuffer
      txEffects.push(
        new TxEffect(
          newNoteHashes.slice(i * MAX_NEW_NOTE_HASHES_PER_TX, (i + 1) * MAX_NEW_NOTE_HASHES_PER_TX) as Tuple<
            Fr,
            typeof MAX_NEW_NOTE_HASHES_PER_TX
          >,
          newNullifiers.slice(i * MAX_NEW_NULLIFIERS_PER_TX, (i + 1) * MAX_NEW_NULLIFIERS_PER_TX) as Tuple<
            Fr,
            typeof MAX_NEW_NULLIFIERS_PER_TX
          >,
          newL2ToL1Msgs.slice(i * MAX_NEW_L2_TO_L1_MSGS_PER_TX, (i + 1) * MAX_NEW_L2_TO_L1_MSGS_PER_TX) as Tuple<
            Fr,
            typeof MAX_NEW_L2_TO_L1_MSGS_PER_TX
          >,
          newPublicDataWrites.slice(
            i * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
            (i + 1) * MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
          ) as Tuple<PublicDataWrite, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
          newContracts.slice(i * MAX_NEW_CONTRACTS_PER_TX, (i + 1) * MAX_NEW_CONTRACTS_PER_TX) as Tuple<
            Fr,
            typeof MAX_NEW_CONTRACTS_PER_TX
          >,
          newContractData.slice(i * MAX_NEW_CONTRACTS_PER_TX, (i + 1) * MAX_NEW_CONTRACTS_PER_TX) as Tuple<
            ContractData,
            typeof MAX_NEW_CONTRACTS_PER_TX
          >,
          newEncryptedLogs.txLogs[i],
          newUnencryptedLogs.txLogs[i],
        ),
      );
    }

    return new this(newL1ToL2Messages, txEffects);
  }

  /**
   * Computes the calldata hash for the L2 block
   * This calldata hash is also computed by the rollup contract when the block is submitted,
   * and inside the circuit, it is part of the public inputs.
   * @returns The calldata hash.
   */
  getCalldataHash() {
    const computeRoot = (leafs: Buffer[]): Buffer => {
      const layers: Buffer[][] = [leafs];
      let activeLayer = 0;

      while (layers[activeLayer].length > 1) {
        const layer: Buffer[] = [];
        const layerLength = layers[activeLayer].length;

        for (let i = 0; i < layerLength; i += 2) {
          const left = layers[activeLayer][i];
          const right = layers[activeLayer][i + 1];

          layer.push(sha256(Buffer.concat([left, right])));
        }

        layers.push(layer);
        activeLayer++;
      }

      return layers[layers.length - 1][0];
    };

    const leafs: Buffer[] = this.txEffects.map(txEffect => txEffect.hash());

    return computeRoot(leafs);
  }

  get encryptedLogs(): L2BlockL2Logs {
    const logs = this.txEffects.map(txEffect => txEffect.encryptedLogs);

    return new L2BlockL2Logs(logs);
  }

  get unencryptedLogs(): L2BlockL2Logs {
    const logs = this.txEffects.map(txEffect => txEffect.unencryptedLogs);

    return new L2BlockL2Logs(logs);
  }

  get numberOfTxs() {
    // We gather all the txEffects that are not empty (the ones that have been padded by checking the first newNullifier of the txEffect);
    return this.txEffects.reduce((acc, txEffect) => (!txEffect.newNullifiers[0].equals(Fr.ZERO) ? acc + 1 : acc), 0);
  }
}

function calculateNumTxsFromNullifiers(nullifiers: Fr[]) {
  let numberOfNonEmptyTxs = 0;
  for (let i = 0; i < nullifiers.length; i += MAX_NEW_NULLIFIERS_PER_TX) {
    if (!nullifiers[i].equals(Fr.ZERO)) {
      numberOfNonEmptyTxs++;
    }
  }

  return numberOfNonEmptyTxs;
}
