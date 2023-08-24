import {
  CommitmentDataOracleInputs,
  CommitmentsDB,
  MessageLoadOracleInputs,
  PublicContractsDB,
  PublicExecutor,
  PublicStateDB,
} from '@aztec/acir-simulator';
import { AztecAddress, CircuitsWasm, EthAddress, Fr, FunctionSelector, HistoricBlockData } from '@aztec/circuits.js';
import { siloCommitment } from '@aztec/circuits.js/abis';
import { ContractDataSource, L1ToL2MessageSource, MerkleTreeId } from '@aztec/types';
import { MerkleTreeOperations, computePublicDataTreeLeafIndex } from '@aztec/world-state';

/**
 * Returns a new PublicExecutor simulator backed by the supplied merkle tree db and contract data source.
 * @param merkleTree - A merkle tree database.
 * @param contractDataSource - A contract data source.
 * @returns A new instance of a PublicExecutor.
 */
export function getPublicExecutor(
  merkleTree: MerkleTreeOperations,
  contractDataSource: ContractDataSource,
  l1toL2MessageSource: L1ToL2MessageSource,
  blockData: HistoricBlockData,
) {
  return new PublicExecutor(
    new WorldStatePublicDB(merkleTree),
    new ContractsDataSourcePublicDB(contractDataSource),
    new WorldStateDB(merkleTree, l1toL2MessageSource),
    blockData,
  );
}

/**
 * Implements the PublicContractsDB using a ContractDataSource.
 */
class ContractsDataSourcePublicDB implements PublicContractsDB {
  constructor(private db: ContractDataSource) {}
  async getBytecode(address: AztecAddress, selector: FunctionSelector): Promise<Buffer | undefined> {
    return (await this.db.getPublicFunction(address, selector))?.bytecode;
  }
  async getIsInternal(address: AztecAddress, selector: FunctionSelector): Promise<boolean | undefined> {
    return (await this.db.getPublicFunction(address, selector))?.isInternal;
  }
  async getPortalContractAddress(address: AztecAddress): Promise<EthAddress | undefined> {
    return (await this.db.getContractData(address))?.portalContractAddress;
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

/**
 * Implements WorldState db using a world state database.
 */
export class WorldStateDB implements CommitmentsDB {
  constructor(private db: MerkleTreeOperations, private l1ToL2MessageSource: L1ToL2MessageSource) {}

  public async getL1ToL2Message(messageKey: Fr): Promise<MessageLoadOracleInputs> {
    // todo: #697 - make this one lookup.
    const message = await this.l1ToL2MessageSource.getConfirmedL1ToL2Message(messageKey);
    const index = (await this.db.findLeafIndex(MerkleTreeId.L1_TO_L2_MESSAGES_TREE, messageKey.toBuffer()))!;
    const siblingPath = await this.db.getSiblingPath(MerkleTreeId.L1_TO_L2_MESSAGES_TREE, index);

    return {
      message: message.toFieldArray(),
      siblingPath: siblingPath.toFieldArray(),
      index,
    };
  }

  public async getCommitmentOracle(address: AztecAddress, innerCommitment: Fr): Promise<CommitmentDataOracleInputs> {
    const siloedCommitment = siloCommitment(await CircuitsWasm.get(), address, innerCommitment);
    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1386): shoild be
    // unique commitment that exists in tree (should be siloed and then made unique via
    // nonce).  Once public kernel or base rollup circuit injects nonces, this can be updated
    // to use uniqueSiloedCommitment.
    const index = (await this.db.findLeafIndex(MerkleTreeId.PRIVATE_DATA_TREE, siloedCommitment.toBuffer()))!;
    const siblingPath = await this.db.getSiblingPath(MerkleTreeId.PRIVATE_DATA_TREE, index);

    return {
      commitment: siloedCommitment,
      siblingPath: siblingPath.toFieldArray(),
      index,
    };
  }
}
