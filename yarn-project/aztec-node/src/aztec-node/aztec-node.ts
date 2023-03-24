import { default as levelup } from 'levelup';
import { default as memdown } from 'memdown';
import { L2BlockSource, Archiver } from '@aztec/archiver';
import { P2P, P2PClient, Tx } from '@aztec/p2p';
import { MerkleTrees, WorldStateSynchroniser, ServerWorldStateSynchroniser } from '@aztec/world-state';
import { SequencerClient } from '@aztec/sequencer-client';
import { AztecNodeConfig } from './config.js';

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore
export const createMemDown = () => memdown();

/**
 * The aztec node.
 */
export class AztecNode {
  constructor(
    private p2pClient?: P2P,
    private blockSource?: L2BlockSource,
    private merkleTreeDB?: MerkleTrees,
    private worldStateSynchroniser?: WorldStateSynchroniser,
    private sequencer?: SequencerClient,
  ) {}

  /**
   * Initialises the Aztec Node, wait for component to sync.
   * @param config - The configuration to be used by the aztec node.
   * @returns - A fully synced Aztec Node for use in development/testing.
   */
  public static async createAndSync(config: AztecNodeConfig) {
    // first create and sync the archiver
    const blockSource = await Archiver.createAndSync(config);

    // give the block source to the P2P network
    const p2pClient = new P2PClient(blockSource);

    // now create the merkle trees and the world state syncher
    const merkleTreeDB = await MerkleTrees.new(levelup(createMemDown()));
    const worldStateSynchroniser = new ServerWorldStateSynchroniser(merkleTreeDB, blockSource);

    // start both and wait for them to sync from the block source
    await Promise.all([p2pClient.start(), worldStateSynchroniser.start()]);

    // now create the sequencer
    const sequencer = await SequencerClient.new(config, p2pClient, worldStateSynchroniser);
    return new AztecNode(p2pClient, blockSource, merkleTreeDB, worldStateSynchroniser, sequencer);
  }

  /**
   * Method to determine if the node is ready to accept transactions.
   * @returns - Flag indicating the readiness for tx submission.
   */
  public async isReady() {
    return (await this.p2pClient?.isReady()) ?? false;
  }

  /**
   * Method to request blocks. Will attempt to return all requested blocks but will return only those available.
   * @param from - The start of the range of blocks to return.
   * @param take - The number of blocks desired.
   * @returns The blocks requested.
   */
  public async getBlocks(from: number, take: number) {
    this.verifyInitialised();
    return (await this.blockSource?.getL2Blocks(from, take)) ?? [];
  }

  /**
   * Method to submit a transaction to the p2p pool.
   * @param tx - The transaction to be submitted.
   */
  public async sendTx(tx: Tx) {
    this.verifyInitialised();
    await this.p2pClient!.sendTx(tx);
  }

  /**
   * Method to stop the aztec node.
   */
  public async stop() {
    await this.p2pClient?.stop();
    await this.worldStateSynchroniser?.stop();
    await this.merkleTreeDB?.stop();
    await this.sequencer?.stop();
    await this.blockSource?.stop();
  }

  /**
   * Method to retrieve pending txs.
   * @returns - The pending txs.
   */
  public async getTxs() {
    return await this.p2pClient!.getTxs();
  }

  /**
   * Method to verify that we are initialised, throws if not.
   */
  private verifyInitialised() {
    const invalid = [
      this.blockSource,
      this.merkleTreeDB,
      this.p2pClient,
      this.worldStateSynchroniser,
      this.sequencer,
    ].findIndex(x => !x);
    if (invalid != -1) {
      throw new Error('Aztec Node not initialised');
    }
  }
}
