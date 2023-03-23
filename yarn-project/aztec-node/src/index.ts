import { default as levelup } from 'levelup';
import { default as memdown } from 'memdown';
import { L2BlockSource, Archiver } from '@aztec/archiver';
import { P2P, P2PCLient } from '@aztec/p2p';
import { MerkleTrees, WorldStateSynchroniser, ServerWorldStateSynchroniser } from '@aztec/world-state';
import { EthAddress } from '@aztec/ethereum.js/eth_address';
import { Tx } from '@aztec/p2p';

/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-ignore
export const createMemDown = () => memdown();

/**
 * The public client.
 */
export class AztecNode {
  private p2pClient?: P2P;
  private blockSource?: L2BlockSource;
  private merkleTreeDB?: MerkleTrees;
  private worldStateSynchroniser?: WorldStateSynchroniser;

  constructor() {}

  /**
   * Initialises the Aztec Node, wait for component to sync.
   * @param rpcUrl - The URL of an Ethereum RPC node.
   * @param rollupAddress - The rollup contract address.
   * @param yeeterAddress - The yeeter contract address.
   */
  public async init(rpcUrl: string, rollupAddress: EthAddress, yeeterAddress: EthAddress) {
    // first configure the block source
    this.blockSource = Archiver.new(rpcUrl, rollupAddress, yeeterAddress);

    await this.blockSource.start();

    // give the block source to the P2P network and the world state synchroniser
    this.p2pClient = new P2PCLient(this.blockSource);
    const db = levelup(createMemDown());
    this.merkleTreeDB = await MerkleTrees.new(db);
    this.worldStateSynchroniser = new ServerWorldStateSynchroniser(this.merkleTreeDB, this.blockSource);

    // start both and wait for them to sync from the block source
    const p2pSyncPromise = this.p2pClient.start();
    const worldStateSyncPromise = this.worldStateSynchroniser.start();
    await Promise.all([p2pSyncPromise, worldStateSyncPromise]);

    // create and start the sequencer
    // new Sequencer(this.blockSource, this.p2pClient, this.merkleTreeDB, this.publisher);
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
    this.verifyInitialised();
    await this.p2pClient!.stop();
    await this.worldStateSynchroniser!.stop();
    await this.merkleTreeDB!.stop();
    await this.blockSource!.stop();
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
    const invalid = [this.blockSource, this.merkleTreeDB, this.p2pClient, this.worldStateSynchroniser].filter(x => !x);
    if (invalid.length) {
      throw new Error('Aztec Node not initialised');
    }
  }
}
