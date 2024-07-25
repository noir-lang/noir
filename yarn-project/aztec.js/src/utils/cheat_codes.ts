import { type Note, type PXE } from '@aztec/circuit-types';
import { type AztecAddress, type EthAddress, Fr } from '@aztec/circuits.js';
import { toBigIntBE, toHex } from '@aztec/foundation/bigint-buffer';
import { keccak256, pedersenHash } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';

import fs from 'fs';

/**
 * A class that provides utility functions for interacting with the chain.
 */
export class CheatCodes {
  constructor(
    /**
     * The cheat codes for ethereum (L1).
     */
    public eth: EthCheatCodes,
    /**
     * The cheat codes for aztec.
     */
    public aztec: AztecCheatCodes,
  ) {}

  static create(rpcUrl: string, pxe: PXE): CheatCodes {
    const ethCheatCodes = new EthCheatCodes(rpcUrl);
    const aztecCheatCodes = new AztecCheatCodes(pxe, ethCheatCodes);
    return new CheatCodes(ethCheatCodes, aztecCheatCodes);
  }
}

/**
 * A class that provides utility functions for interacting with ethereum (L1).
 */
export class EthCheatCodes {
  constructor(
    /**
     * The RPC URL to use for interacting with the chain
     */
    public rpcUrl: string,
    /**
     * The logger to use for the eth cheatcodes
     */
    public logger = createDebugLogger('aztec:cheat_codes:eth'),
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
   * Advance the chain by a number of blocks
   * @param numberOfBlocks - The number of blocks to mine
   * @returns The current chainId
   */
  public async mine(numberOfBlocks = 1): Promise<void> {
    const res = await this.rpcCall('hardhat_mine', [numberOfBlocks]);
    if (res.error) {
      throw new Error(`Error mining: ${res.error.message}`);
    }
    this.logger.info(`Mined ${numberOfBlocks} blocks`);
  }

  /**
   * Set the next block timestamp
   * @param timestamp - The timestamp to set the next block to
   */
  public async setNextBlockTimestamp(timestamp: number): Promise<void> {
    const res = await this.rpcCall('evm_setNextBlockTimestamp', [timestamp]);
    if (res.error) {
      throw new Error(`Error setting next block timestamp: ${res.error.message}`);
    }
    this.logger.info(`Set next block timestamp to ${timestamp}`);
  }

  /**
   * Dumps the current chain state to a file.
   * @param fileName - The file name to dump state into
   */
  public async dumpChainState(fileName: string): Promise<void> {
    const res = await this.rpcCall('hardhat_dumpState', []);
    if (res.error) {
      throw new Error(`Error dumping state: ${res.error.message}`);
    }
    const jsonContent = JSON.stringify(res.result);
    fs.writeFileSync(`${fileName}.json`, jsonContent, 'utf8');
    this.logger.info(`Dumped state to ${fileName}`);
  }

  /**
   * Loads the chain state from a file.
   * @param fileName - The file name to load state from
   */
  public async loadChainState(fileName: string): Promise<void> {
    const data = JSON.parse(fs.readFileSync(`${fileName}.json`, 'utf8'));
    const res = await this.rpcCall('hardhat_loadState', [data]);
    if (res.error) {
      throw new Error(`Error loading state: ${res.error.message}`);
    }
    this.logger.info(`Loaded state from ${fileName}`);
  }

  /**
   * Load the value at a storage slot of a contract address on eth
   * @param contract - The contract address
   * @param slot - The storage slot
   * @returns - The value at the storage slot
   */
  public async load(contract: EthAddress, slot: bigint): Promise<bigint> {
    const res = await this.rpcCall('eth_getStorageAt', [contract.toString(), toHex(slot), 'latest']);
    return BigInt(res.result);
  }

  /**
   * Set the value at a storage slot of a contract address on eth
   * @param contract - The contract address
   * @param slot - The storage slot
   * @param value - The value to set the storage slot to
   */
  public async store(contract: EthAddress, slot: bigint, value: bigint): Promise<void> {
    // for the rpc call, we need to change value to be a 32 byte hex string.
    const res = await this.rpcCall('hardhat_setStorageAt', [contract.toString(), toHex(slot), toHex(value, true)]);
    if (res.error) {
      throw new Error(`Error setting storage for contract ${contract} at ${slot}: ${res.error.message}`);
    }
    this.logger.info(`Set storage for contract ${contract} at ${slot} to ${value}`);
  }

  /**
   * Computes the slot value for a given map and key.
   * @param baseSlot - The base slot of the map (specified in Aztec.nr contract)
   * @param key - The key to lookup in the map
   * @returns The storage slot of the value in the map
   */
  public keccak256(baseSlot: bigint, key: bigint): bigint {
    // abi encode (removing the 0x) - concat key and baseSlot (both padded to 32 bytes)
    const abiEncoded = toHex(key, true).substring(2) + toHex(baseSlot, true).substring(2);
    return toBigIntBE(keccak256(Buffer.from(abiEncoded, 'hex')));
  }

  /**
   * Send transactions impersonating an externally owned account or contract.
   * @param who - The address to impersonate
   */
  public async startImpersonating(who: EthAddress): Promise<void> {
    const res = await this.rpcCall('hardhat_impersonateAccount', [who.toString()]);
    if (res.error) {
      throw new Error(`Error impersonating ${who}: ${res.error.message}`);
    }
    this.logger.info(`Impersonating ${who}`);
  }

  /**
   * Stop impersonating an account that you are currently impersonating.
   * @param who - The address to stop impersonating
   */
  public async stopImpersonating(who: EthAddress): Promise<void> {
    const res = await this.rpcCall('hardhat_stopImpersonatingAccount', [who.toString()]);
    if (res.error) {
      throw new Error(`Error when stopping the impersonation of ${who}: ${res.error.message}`);
    }
    this.logger.info(`Stopped impersonating ${who}`);
  }

  /**
   * Set the bytecode for a contract
   * @param contract - The contract address
   * @param bytecode - The bytecode to set
   */
  public async etch(contract: EthAddress, bytecode: `0x${string}`): Promise<void> {
    const res = await this.rpcCall('hardhat_setCode', [contract.toString(), bytecode]);
    if (res.error) {
      throw new Error(`Error setting bytecode for ${contract}: ${res.error.message}`);
    }
    this.logger.info(`Set bytecode for ${contract} to ${bytecode}`);
  }

  /**
   * Get the bytecode for a contract
   * @param contract - The contract address
   * @returns The bytecode for the contract
   */
  public async getBytecode(contract: EthAddress): Promise<`0x${string}`> {
    const res = await this.rpcCall('eth_getCode', [contract.toString(), 'latest']);
    return res.result;
  }
}

/**
 * A class that provides utility functions for interacting with the aztec chain.
 */
export class AztecCheatCodes {
  constructor(
    /**
     * The PXE Service to use for interacting with the chain
     */
    public pxe: PXE,
    /**
     * The eth cheat codes.
     */
    public eth: EthCheatCodes,
    /**
     * The logger to use for the aztec cheatcodes
     */
    public logger = createDebugLogger('aztec:cheat_codes:aztec'),
  ) {}

  /**
   * Computes the slot value for a given map and key.
   * @param baseSlot - The base slot of the map (specified in Aztec.nr contract)
   * @param key - The key to lookup in the map
   * @returns The storage slot of the value in the map
   */
  public computeSlotInMap(baseSlot: Fr | bigint, key: Fr | bigint | AztecAddress): Fr {
    // Based on `at` function in
    // aztec3-packages/aztec-nr/aztec/src/state_vars/map.nr
    return pedersenHash([new Fr(baseSlot), new Fr(key)]);
  }

  /**
   * Get the current blocknumber
   * @returns The current block number
   */
  public async blockNumber(): Promise<number> {
    return await this.pxe.getBlockNumber();
  }

  /**
   * Set time of the next execution on aztec.
   * It also modifies time on eth for next execution and stores this time as the last rollup block on the rollup contract.
   * @param to - The timestamp to set the next block to (must be greater than current time)
   */
  public async warp(to: number): Promise<void> {
    const rollupContract = (await this.pxe.getNodeInfo()).l1ContractAddresses.rollupAddress;
    await this.eth.setNextBlockTimestamp(to);
    // also store this time on the rollup contract (slot 2 tracks `lastBlockTs`).
    // This is because when the sequencer executes public functions, it uses the timestamp stored in the rollup contract.
    await this.eth.store(rollupContract, 7n, BigInt(to));
    // also store this on slot of the rollup contract (`lastWarpedBlockTs`) which tracks the last time warp was used.
    await this.eth.store(rollupContract, 8n, BigInt(to));
  }

  /**
   * Loads the value stored at the given slot in the public storage of the given contract.
   * @param who - The address of the contract
   * @param slot - The storage slot to lookup
   * @returns The value stored at the given slot
   */
  public async loadPublic(who: AztecAddress, slot: Fr | bigint): Promise<Fr> {
    const storageValue = await this.pxe.getPublicStorageAt(who, new Fr(slot));
    return storageValue;
  }

  /**
   * Loads the value stored at the given slot in the private storage of the given contract.
   * @param contract - The address of the contract
   * @param owner - The owner for whom the notes are encrypted
   * @param slot - The storage slot to lookup
   * @returns The notes stored at the given slot
   */
  public async loadPrivate(owner: AztecAddress, contract: AztecAddress, slot: Fr | bigint): Promise<Note[]> {
    const extendedNotes = await this.pxe.getIncomingNotes({
      owner,
      contractAddress: contract,
      storageSlot: new Fr(slot),
    });
    return extendedNotes.map(extendedNote => extendedNote.note);
  }
}
