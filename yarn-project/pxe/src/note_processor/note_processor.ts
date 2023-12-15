import { MAX_NEW_COMMITMENTS_PER_TX, MAX_NEW_NULLIFIERS_PER_TX, PublicKey } from '@aztec/circuits.js';
import { computeCommitmentNonce, siloNullifier } from '@aztec/circuits.js/abis';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import { AztecNode, INITIAL_L2_BLOCK_NUM, KeyStore, L1NotePayload, L2BlockContext, L2BlockL2Logs } from '@aztec/types';
import { NoteProcessorStats } from '@aztec/types/stats';

import { PxeDatabase } from '../database/index.js';
import { NoteDao } from '../database/note_dao.js';
import { getAcirSimulator } from '../simulator/index.js';

/**
 * Contains all the decrypted data in this array so that we can later batch insert it all into the database.
 */
interface ProcessedData {
  /**
   * Holds L2 block and a cache of already requested tx hashes.
   */
  blockContext: L2BlockContext;
  /**
   * DAOs of processed notes.
   */
  noteDaos: NoteDao[];
}

/**
 * NoteProcessor is responsible for decrypting logs and converting them to notes via their originating contracts
 * before storing them against their owner.
 */
export class NoteProcessor {
  /** Keeps track of processing time since an instance is created. */
  public readonly timer: Timer = new Timer();

  /** Stats accumulated for this processor. */
  public readonly stats: NoteProcessorStats = { seen: 0, decrypted: 0, failed: 0, blocks: 0, txs: 0 };

  constructor(
    /**
     * The public counterpart to the private key to be used in note decryption.
     */
    public readonly publicKey: PublicKey,
    private keyStore: KeyStore,
    private db: PxeDatabase,
    private node: AztecNode,
    private startingBlock: number = INITIAL_L2_BLOCK_NUM,
    private simulator = getAcirSimulator(db, node, keyStore),
    private log = createDebugLogger('aztec:note_processor'),
  ) {}

  /**
   * Check if the NoteProcessor is synchronized with the remote block number.
   * The function queries the remote block number from the AztecNode and compares it with the syncedToBlock value in the NoteProcessor.
   * If the values are equal, then the NoteProcessor is considered to be synchronized, otherwise not.
   *
   * @returns A boolean indicating whether the NoteProcessor is synchronized with the remote block number or not.
   */
  public async isSynchronized() {
    const remoteBlockNumber = await this.node.getBlockNumber();
    return this.getSyncedToBlock() === remoteBlockNumber;
  }

  /**
   * Returns synchronization status (ie up to which block has been synced ) for this note processor.
   */
  public get status() {
    return { syncedToBlock: this.getSyncedToBlock() };
  }

  private getSyncedToBlock(): number {
    return this.db.getSynchedBlockNumberForPublicKey(this.publicKey) ?? this.startingBlock - 1;
  }

  /**
   * Process the given L2 block contexts and encrypted logs to update the note processor.
   * It synchronizes the user's account by decrypting the encrypted logs and processing
   * the transactions and auxiliary data associated with them.
   * Throws an error if the number of block contexts and encrypted logs do not match.
   *
   * @param l2BlockContexts - An array of L2 block contexts to be processed.
   * @param encryptedL2BlockLogs - An array of encrypted logs associated with the L2 block contexts.
   * @returns A promise that resolves once the processing is completed.
   */
  public async process(l2BlockContexts: L2BlockContext[], encryptedL2BlockLogs: L2BlockL2Logs[]): Promise<void> {
    if (l2BlockContexts.length !== encryptedL2BlockLogs.length) {
      throw new Error(
        `Number of blocks and EncryptedLogs is not equal. Received ${l2BlockContexts.length} blocks, ${encryptedL2BlockLogs.length} encrypted logs.`,
      );
    }
    if (!l2BlockContexts.length) {
      return;
    }

    const blocksAndNotes: ProcessedData[] = [];
    const curve = new Grumpkin();

    // Iterate over both blocks and encrypted logs.
    for (let blockIndex = 0; blockIndex < encryptedL2BlockLogs.length; ++blockIndex) {
      this.stats.blocks++;
      const { txLogs } = encryptedL2BlockLogs[blockIndex];
      const blockContext = l2BlockContexts[blockIndex];
      const block = blockContext.block;
      const dataStartIndexForBlock = block.startNoteHashTreeSnapshot.nextAvailableLeafIndex;

      // We are using set for `userPertainingTxIndices` to avoid duplicates. This would happen in case there were
      // multiple encrypted logs in a tx pertaining to a user.
      const noteDaos: NoteDao[] = [];
      const privateKey = await this.keyStore.getAccountPrivateKey(this.publicKey);

      // Iterate over all the encrypted logs and try decrypting them. If successful, store the note.
      for (let indexOfTxInABlock = 0; indexOfTxInABlock < txLogs.length; ++indexOfTxInABlock) {
        this.stats.txs++;
        const dataStartIndexForTx = dataStartIndexForBlock + indexOfTxInABlock * MAX_NEW_COMMITMENTS_PER_TX;
        const newCommitments = block.newCommitments.slice(
          indexOfTxInABlock * MAX_NEW_COMMITMENTS_PER_TX,
          (indexOfTxInABlock + 1) * MAX_NEW_COMMITMENTS_PER_TX,
        );
        const newNullifiers = block.newNullifiers.slice(
          indexOfTxInABlock * MAX_NEW_NULLIFIERS_PER_TX,
          (indexOfTxInABlock + 1) * MAX_NEW_NULLIFIERS_PER_TX,
        );
        // Note: Each tx generates a `TxL2Logs` object and for this reason we can rely on its index corresponding
        //       to the index of a tx in a block.
        const txFunctionLogs = txLogs[indexOfTxInABlock].functionLogs;
        const excludedIndices: Set<number> = new Set();
        for (const functionLogs of txFunctionLogs) {
          for (const logs of functionLogs.logs) {
            this.stats.seen++;
            const payload = L1NotePayload.fromEncryptedBuffer(logs, privateKey, curve);
            if (payload) {
              // We have successfully decrypted the data.
              try {
                const { commitmentIndex, nonce, innerNoteHash, siloedNullifier } = await this.findNoteIndexAndNullifier(
                  newCommitments,
                  newNullifiers[0],
                  payload,
                  excludedIndices,
                );
                const index = BigInt(dataStartIndexForTx + commitmentIndex);
                excludedIndices.add(commitmentIndex);
                noteDaos.push(
                  new NoteDao(
                    payload.note,
                    payload.contractAddress,
                    payload.storageSlot,
                    blockContext.getTxHash(indexOfTxInABlock),
                    nonce,
                    innerNoteHash,
                    siloedNullifier,
                    index,
                    this.publicKey,
                  ),
                );
                this.stats.decrypted++;
              } catch (e) {
                this.stats.failed++;
                this.log.warn(`Could not process note because of "${e}". Skipping note...`);
              }
            }
          }
        }
      }

      blocksAndNotes.push({
        blockContext: l2BlockContexts[blockIndex],
        noteDaos,
      });
    }

    await this.processBlocksAndNotes(blocksAndNotes);

    const syncedToBlock = l2BlockContexts[l2BlockContexts.length - 1].block.number;
    await this.db.setSynchedBlockNumberForPublicKey(this.publicKey, syncedToBlock);

    this.log(`Synched block ${syncedToBlock}`);
  }

  /**
   * Find the index of the note in the note hash tree by computing the note hash with different nonce and see which
   * commitment for the current tx matches this value.
   * Compute a nullifier for a given l1NotePayload.
   * The nullifier is calculated using the private key of the account,
   * contract address, and the note associated with the l1NotePayload.
   * This method assists in identifying spent commitments in the private state.
   * @param commitments - Commitments in the tx. One of them should be the note's commitment.
   * @param firstNullifier - First nullifier in the tx.
   * @param l1NotePayload - An instance of l1NotePayload.
   * @param excludedIndices - Indices that have been assigned a note in the same tx. Notes in a tx can have the same
   * l1NotePayload. We need to find a different index for each replicate.
   * @returns Information for a decrypted note, including the index of its commitment, nonce, inner note
   * hash, and the siloed nullifier. Throw if cannot find the nonce for the note.
   */
  private async findNoteIndexAndNullifier(
    commitments: Fr[],
    firstNullifier: Fr,
    { contractAddress, storageSlot, note }: L1NotePayload,
    excludedIndices: Set<number>,
  ) {
    let commitmentIndex = 0;
    let nonce: Fr | undefined;
    let innerNoteHash: Fr | undefined;
    let siloedNoteHash: Fr | undefined;
    let uniqueSiloedNoteHash: Fr | undefined;
    let innerNullifier: Fr | undefined;
    for (; commitmentIndex < commitments.length; ++commitmentIndex) {
      if (excludedIndices.has(commitmentIndex)) {
        continue;
      }

      const commitment = commitments[commitmentIndex];
      if (commitment.equals(Fr.ZERO)) {
        break;
      }

      const expectedNonce = computeCommitmentNonce(firstNullifier, commitmentIndex);
      ({ innerNoteHash, siloedNoteHash, uniqueSiloedNoteHash, innerNullifier } =
        await this.simulator.computeNoteHashAndNullifier(contractAddress, expectedNonce, storageSlot, note));
      if (commitment.equals(uniqueSiloedNoteHash)) {
        nonce = expectedNonce;
        break;
      }
    }

    if (!nonce) {
      let errorString;
      if (siloedNoteHash == undefined) {
        errorString = 'Cannot find a matching commitment for the note.';
      } else {
        errorString = `We decrypted a log, but couldn't find a corresponding note in the tree.
This might be because the note was nullified in the same tx which created it.
In that case, everything is fine. To check whether this is the case, look back through
the logs for a notification
'important: chopped commitment for siloed inner hash note
${siloedNoteHash.toString()}'.
If you can see that notification. Everything's fine.
If that's not the case, and you can't find such a notification, something has gone wrong.
There could be a problem with the way you've defined a custom note, or with the way you're
serializing / deserializing / hashing / encrypting / decrypting that note.
Please see the following github issue to track an improvement that we're working on:
https://github.com/AztecProtocol/aztec-packages/issues/1641`;
      }

      throw new Error(errorString);
    }

    return {
      commitmentIndex,
      nonce,
      innerNoteHash: innerNoteHash!,
      siloedNullifier: siloNullifier(contractAddress, innerNullifier!),
    };
  }

  /**
   * Process the given blocks and their associated transaction auxiliary data.
   * This function updates the database with information about new transactions,
   * user-pertaining transaction indices, and auxiliary data. It also removes nullified
   * transaction auxiliary data from the database. This function keeps track of new nullifiers
   * and ensures all other transactions are updated with newly settled block information.
   *
   * @param blocksAndNotes - Array of objects containing L2BlockContexts, user-pertaining transaction indices, and NoteDaos.
   */
  private async processBlocksAndNotes(blocksAndNotes: ProcessedData[]) {
    const noteDaos = blocksAndNotes.flatMap(b => b.noteDaos);
    if (noteDaos.length) {
      await this.db.addNotes(noteDaos);
      noteDaos.forEach(noteDao => {
        this.log(
          `Added note for contract ${noteDao.contractAddress} at slot ${
            noteDao.storageSlot
          } with nullifier ${noteDao.siloedNullifier.toString()}`,
        );
      });
    }

    const newNullifiers: Fr[] = blocksAndNotes.flatMap(b => b.blockContext.block.newNullifiers);
    const removedNotes = await this.db.removeNullifiedNotes(newNullifiers, this.publicKey);
    removedNotes.forEach(noteDao => {
      this.log(
        `Removed note for contract ${noteDao.contractAddress} at slot ${
          noteDao.storageSlot
        } with nullifier ${noteDao.siloedNullifier.toString()}`,
      );
    });
  }
}
