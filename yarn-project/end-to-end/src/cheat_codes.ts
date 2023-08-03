import { AztecAddress, CircuitsWasm, Fr } from '@aztec/circuits.js';
import { pedersenPlookupCommitInputs } from '@aztec/circuits.js/barretenberg';
import { createDebugLogger } from '@aztec/foundation/log';
import { AztecRPC } from '@aztec/types';

const toFr = (value: Fr | bigint): Fr => {
  return typeof value === 'bigint' ? new Fr(value) : value;
};

/**
 * A class that provides utility functions for interacting with the chain.
 */
export class CheatCodes {
  constructor(
    /**
     * The L1 cheat codes.
     */
    public l1: L1CheatCodes,
    /**
     * The L2 cheat codes.
     */
    public l2: L2CheatCodes,
  ) {}

  static async create(rpcUrl: string, aztecRpc: AztecRPC): Promise<CheatCodes> {
    const l1CheatCodes = new L1CheatCodes(rpcUrl);
    const l2CheatCodes = new L2CheatCodes(aztecRpc, await CircuitsWasm.get(), l1CheatCodes);
    return new CheatCodes(l1CheatCodes, l2CheatCodes);
  }
}

/**
 * A class that provides utility functions for interacting with the L1 chain.
 */
class L1CheatCodes {
  constructor(
    /**
     * The RPC client to use for interacting with the chain
     */
    public rpcUrl: string,
    /**
     * The logger to use for the l1 cheatcodes
     */
    public logger = createDebugLogger('aztec:cheat_codes:l1'),
  ) {}

  async rpcCall(method: string, params: any[]) {
    const paramsString = JSON.stringify(params);
    const content = {
      body: `{"jsonrpc":"2.0", "method": "${method}", "params": ${paramsString}, "id": 1}`,
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
    };
    return await (await fetch(this.rpcUrl, content)).json();
  }

  /**
   * Get the current blocknumber
   * @returns The current block number
   */
  public async blockNumber(): Promise<number> {
    const res = await this.rpcCall('eth_blockNumber', []);
    return parseInt(res.result, 16);
  }

  /**
   * Get the current chainId
   * @returns The current chainId
   */
  public async chainId(): Promise<number> {
    const res = await this.rpcCall('eth_chainId', []);
    return parseInt(res.result, 16);
  }

  /**
   * Get the current timestamp
   * @returns The current timestamp
   */
  public async timestamp(): Promise<number> {
    const res = await this.rpcCall('eth_getBlockByNumber', ['latest', true]);
    return parseInt(res.result.timestamp, 16);
  }

  /**
   * Get the current chainId
   * @param numberOfBlocks - The number of blocks to mine
   * @returns The current chainId
   */
  public async mine(numberOfBlocks = 1): Promise<void> {
    const res = await this.rpcCall('anvil_mine', [numberOfBlocks]);
    if (res.error) throw new Error(`Error mining: ${res.error.message}`);
    this.logger(`Mined ${numberOfBlocks} blocks`);
  }

  /**
   * Set the next block timestamp
   * @param timestamp - The timestamp to set the next block to
   */
  public async setNextBlockTimestamp(timestamp: number): Promise<void> {
    const res = await this.rpcCall('anvil_setNextBlockTimestamp', [timestamp]);
    if (res.error) throw new Error(`Error setting next block timestamp: ${res.error.message}`);
    this.logger(`Set next block timestamp to ${timestamp}`);
  }

  // Good basis for the remaining functions:
  // https://github.com/foundry-rs/foundry/blob/master/anvil/core/src/eth/mod.rs
}

/**
 * A class that provides utility functions for interacting with the L2 chain.
 */
class L2CheatCodes {
  constructor(
    /**
     * The RPC client to use for interacting with the chain
     */
    public aztecRpc: AztecRPC,
    /**
     * The circuits wasm module used for pedersen hashing
     */
    public wasm: CircuitsWasm,
    /**
     * The L1 cheat codes.
     */
    public l1: L1CheatCodes,
    /**
     * The logger to use for the l2 cheatcodes
     */
    public logger = createDebugLogger('aztec:cheat_codes:l2'),
  ) {}

  /**
   * Computes the slot value for a given map and key.
   * @param baseSlot - The base slot of the map (specified in noir contract)
   * @param key - The key to lookup in the map
   * @returns The storage slot of the value in the map
   */
  public computeSlotInMap(baseSlot: Fr | bigint, key: Fr | bigint): Fr {
    // Based on `at` function in
    // aztec3-packages/yarn-project/noir-contracts/src/contracts/noir-aztec/src/state_vars/map.nr
    return Fr.fromBuffer(
      pedersenPlookupCommitInputs(
        this.wasm,
        [toFr(baseSlot), toFr(key)].map(f => f.toBuffer()),
      ),
    );
  }

  /**
   * Get the current blocknumber
   * @returns The current block number
   */
  public async blockNumber(): Promise<number> {
    return await this.aztecRpc.getBlockNum();
  }

  /**
   * Loads the value stored at the given slot in the public storage of the given contract.
   * @param who - The address of the contract
   * @param slot - The storage slot to lookup
   * @returns The value stored at the given slot
   */
  public async loadPublic(who: AztecAddress, slot: Fr | bigint): Promise<Fr> {
    const storageValue = await this.aztecRpc.getPublicStorageAt(who, toFr(slot));
    if (storageValue === undefined) {
      throw new Error(`Storage slot ${slot} not found`);
    }
    return Fr.fromBuffer(storageValue);
  }
}
