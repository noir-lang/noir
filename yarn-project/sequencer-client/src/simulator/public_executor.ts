import { PublicContractsDB, PublicExecutor, PublicStateDB } from '@aztec/acir-simulator';
import { CircuitsWasm } from '@aztec/circuits.js';
import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { ContractDataSource, MerkleTreeId } from '@aztec/types';
import { MerkleTreeOperations, computePublicDataTreeLeafIndex } from '@aztec/world-state';

/**
 * Returns a new PublicExecutor simulator backed by the supplied merkle tree db and contract data source.
 * @param merkleTree - A merkle tree database.
 * @param contractDataSource - A contract data source.
 * @returns A new instance of a PublicExecutor.
 */
export function getPublicExecutor(merkleTree: MerkleTreeOperations, contractDataSource: ContractDataSource) {
  return new PublicExecutor(new WorldStatePublicDB(merkleTree), new ContractsDataSourcePublicDB(contractDataSource));
}

/**
 * Implements the PublicContractsDB using a ContractDataSource.
 */
class ContractsDataSourcePublicDB implements PublicContractsDB {
  constructor(private db: ContractDataSource) {}
  async getBytecode(address: AztecAddress, functionSelector: Buffer): Promise<Buffer | undefined> {
    return (await this.db.getPublicFunction(address, functionSelector))?.bytecode;
  }
  async getPortalContractAddress(address: AztecAddress): Promise<EthAddress | undefined> {
    return (await this.db.getL2ContractInfo(address))?.portalContractAddress;
  }
}

/**
 * Implements the PublicStateDB using a world-state database.
 */
class WorldStatePublicDB implements PublicStateDB {
  private writeCache: Map<bigint, Fr> = new Map();

  constructor(private db: MerkleTreeOperations) {}

  /**
   * Reads a value from public storage, returning zero if none.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @returns The current value in the storage slot.
   */
  public async storageRead(contract: AztecAddress, slot: Fr): Promise<Fr> {
    const index = computePublicDataTreeLeafIndex(contract, slot, await CircuitsWasm.get());
    const cached = this.writeCache.get(index);
    if (cached !== undefined) return cached;
    const value = await this.db.getLeafValue(MerkleTreeId.PUBLIC_DATA_TREE, index);
    return value ? Fr.fromBuffer(value) : Fr.ZERO;
  }

  /**
   * Records a write to public storage.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @param newValue - The new value to store.
   */
  public async storageWrite(contract: AztecAddress, slot: Fr, newValue: Fr): Promise<void> {
    const index = computePublicDataTreeLeafIndex(contract, slot, await CircuitsWasm.get());
    this.writeCache.set(index, newValue);
  }
}
