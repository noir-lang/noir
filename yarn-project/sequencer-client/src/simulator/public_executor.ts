import { MerkleTreeId, NullifierMembershipWitness, type Tx } from '@aztec/circuit-types';
import {
  type AztecAddress,
  ContractClassRegisteredEvent,
  ContractInstanceDeployedEvent,
  type EthAddress,
  Fr,
  type FunctionSelector,
  type L1_TO_L2_MSG_TREE_HEIGHT,
  type NULLIFIER_TREE_HEIGHT,
  type NullifierLeafPreimage,
  type PublicDataTreeLeafPreimage,
} from '@aztec/circuits.js';
import { computeL1ToL2MessageNullifier, computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import { createDebugLogger } from '@aztec/foundation/log';
import { getCanonicalClassRegistererAddress } from '@aztec/protocol-contracts/class-registerer';
import {
  type CommitmentsDB,
  MessageLoadOracleInputs,
  type PublicContractsDB,
  type PublicStateDB,
} from '@aztec/simulator';
import {
  type ContractClassPublic,
  type ContractDataSource,
  type ContractInstanceWithAddress,
} from '@aztec/types/contracts';
import { type MerkleTreeOperations } from '@aztec/world-state';

/**
 * Implements the PublicContractsDB using a ContractDataSource.
 * Progressively records contracts in transaction as they are processed in a block.
 */
export class ContractsDataSourcePublicDB implements PublicContractsDB {
  private instanceCache = new Map<string, ContractInstanceWithAddress>();
  private classCache = new Map<string, ContractClassPublic>();

  private log = createDebugLogger('aztec:sequencer:contracts-data-source');

  constructor(private db: ContractDataSource) {}

  /**
   * Add new contracts from a transaction
   * @param tx - The transaction to add contracts from.
   */
  public addNewContracts(tx: Tx): Promise<void> {
    // Extract contract class and instance data from logs and add to cache for this block
    const logs = tx.unencryptedLogs.unrollLogs();
    ContractClassRegisteredEvent.fromLogs(logs, getCanonicalClassRegistererAddress()).forEach(e => {
      this.log(`Adding class ${e.contractClassId.toString()} to public execution contract cache`);
      this.classCache.set(e.contractClassId.toString(), e.toContractClassPublic());
    });
    ContractInstanceDeployedEvent.fromLogs(logs).forEach(e => {
      this.log(
        `Adding instance ${e.address.toString()} with class ${e.contractClassId.toString()} to public execution contract cache`,
      );
      this.instanceCache.set(e.address.toString(), e.toContractInstance());
    });

    return Promise.resolve();
  }

  /**
   * Removes new contracts added from transactions
   * @param tx - The tx's contracts to be removed
   */
  public removeNewContracts(tx: Tx): Promise<void> {
    // TODO(@spalladino): Can this inadvertently delete a valid contract added by another tx?
    // Let's say we have two txs adding the same contract on the same block. If the 2nd one reverts,
    // wouldn't that accidentally remove the contract added on the first one?
    const logs = tx.unencryptedLogs.unrollLogs();
    ContractClassRegisteredEvent.fromLogs(logs, getCanonicalClassRegistererAddress()).forEach(e =>
      this.classCache.delete(e.contractClassId.toString()),
    );
    ContractInstanceDeployedEvent.fromLogs(logs).forEach(e => this.instanceCache.delete(e.address.toString()));
    return Promise.resolve();
  }

  public async getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    return this.instanceCache.get(address.toString()) ?? (await this.db.getContract(address));
  }

  public async getContractClass(contractClassId: Fr): Promise<ContractClassPublic | undefined> {
    return this.classCache.get(contractClassId.toString()) ?? (await this.db.getContractClass(contractClassId));
  }

  async getBytecode(address: AztecAddress, selector: FunctionSelector): Promise<Buffer | undefined> {
    const instance = await this.getContractInstance(address);
    if (!instance) {
      throw new Error(`Contract ${address.toString()} not found`);
    }
    const contractClass = await this.getContractClass(instance.contractClassId);
    if (!contractClass) {
      throw new Error(`Contract class ${instance.contractClassId.toString()} for ${address.toString()} not found`);
    }
    return contractClass.publicFunctions.find(f => f.selector.equals(selector))?.bytecode;
  }

  async getPortalContractAddress(address: AztecAddress): Promise<EthAddress | undefined> {
    const contract = await this.getContractInstance(address);
    return contract?.portalContractAddress;
  }
}

/**
 * Implements the PublicStateDB using a world-state database.
 */
export class WorldStatePublicDB implements PublicStateDB {
  private committedWriteCache: Map<bigint, Fr> = new Map();
  private checkpointedWriteCache: Map<bigint, Fr> = new Map();
  private uncommittedWriteCache: Map<bigint, Fr> = new Map();

  constructor(private db: MerkleTreeOperations) {}

  /**
   * Reads a value from public storage, returning zero if none.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @returns The current value in the storage slot.
   */
  public async storageRead(contract: AztecAddress, slot: Fr): Promise<Fr> {
    const leafSlot = computePublicDataTreeLeafSlot(contract, slot).value;
    const uncommitted = this.uncommittedWriteCache.get(leafSlot);
    if (uncommitted !== undefined) {
      return uncommitted;
    }
    const checkpointed = this.checkpointedWriteCache.get(leafSlot);
    if (checkpointed !== undefined) {
      return checkpointed;
    }
    const committed = this.committedWriteCache.get(leafSlot);
    if (committed !== undefined) {
      return committed;
    }

    const lowLeafResult = await this.db.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot);
    if (!lowLeafResult || !lowLeafResult.alreadyPresent) {
      return Fr.ZERO;
    }

    const preimage = (await this.db.getLeafPreimage(
      MerkleTreeId.PUBLIC_DATA_TREE,
      lowLeafResult.index,
    )) as PublicDataTreeLeafPreimage;

    return preimage.value;
  }

  /**
   * Records a write to public storage.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @param newValue - The new value to store.
   */
  public storageWrite(contract: AztecAddress, slot: Fr, newValue: Fr): Promise<void> {
    const index = computePublicDataTreeLeafSlot(contract, slot).value;
    this.uncommittedWriteCache.set(index, newValue);
    return Promise.resolve();
  }

  /**
   * Commit the pending changes to the DB.
   * @returns Nothing.
   */
  commit(): Promise<void> {
    for (const [k, v] of this.checkpointedWriteCache) {
      this.committedWriteCache.set(k, v);
    }
    // uncommitted writes take precedence over checkpointed writes
    // since they are the most recent
    for (const [k, v] of this.uncommittedWriteCache) {
      this.committedWriteCache.set(k, v);
    }
    return this.rollbackToCommit();
  }

  /**
   * Rollback the pending changes.
   * @returns Nothing.
   */
  async rollbackToCommit(): Promise<void> {
    await this.rollbackToCheckpoint();
    this.checkpointedWriteCache = new Map<bigint, Fr>();
    return Promise.resolve();
  }

  checkpoint(): Promise<void> {
    for (const [k, v] of this.uncommittedWriteCache) {
      this.checkpointedWriteCache.set(k, v);
    }
    return this.rollbackToCheckpoint();
  }

  rollbackToCheckpoint(): Promise<void> {
    this.uncommittedWriteCache = new Map<bigint, Fr>();
    return Promise.resolve();
  }
}

/**
 * Implements WorldState db using a world state database.
 */
export class WorldStateDB implements CommitmentsDB {
  constructor(private db: MerkleTreeOperations) {}

  public async getNullifierMembershipWitnessAtLatestBlock(
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    const index = await this.db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    if (!index) {
      return undefined;
    }

    const leafPreimagePromise = this.db.getLeafPreimage(MerkleTreeId.NULLIFIER_TREE, index);
    const siblingPathPromise = this.db.getSiblingPath<typeof NULLIFIER_TREE_HEIGHT>(
      MerkleTreeId.NULLIFIER_TREE,
      BigInt(index),
    );

    const [leafPreimage, siblingPath] = await Promise.all([leafPreimagePromise, siblingPathPromise]);

    if (!leafPreimage) {
      return undefined;
    }

    return new NullifierMembershipWitness(BigInt(index), leafPreimage as NullifierLeafPreimage, siblingPath);
  }

  public async getL1ToL2MembershipWitness(
    contractAddress: AztecAddress,
    messageHash: Fr,
    secret: Fr,
  ): Promise<MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT>> {
    let nullifierIndex: bigint | undefined;
    let messageIndex: bigint | undefined;
    let startIndex = 0n;

    // We iterate over messages until we find one whose nullifier is not in the nullifier tree --> we need to check
    // for nullifiers because messages can have duplicates.
    do {
      messageIndex = (await this.db.findLeafIndexAfter(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, messageHash, startIndex))!;
      if (messageIndex === undefined) {
        throw new Error(`No non-nullified L1 to L2 message found for message hash ${messageHash.toString()}`);
      }

      const messageNullifier = computeL1ToL2MessageNullifier(contractAddress, messageHash, secret, messageIndex);
      nullifierIndex = await this.getNullifierIndex(messageNullifier);

      startIndex = messageIndex + 1n;
    } while (nullifierIndex !== undefined);

    const siblingPath = await this.db.getSiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      messageIndex,
    );

    return new MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT>(messageIndex, siblingPath);
  }

  public async getCommitmentIndex(commitment: Fr): Promise<bigint | undefined> {
    return await this.db.findLeafIndex(MerkleTreeId.NOTE_HASH_TREE, commitment);
  }

  public async getNullifierIndex(nullifier: Fr): Promise<bigint | undefined> {
    return await this.db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
  }
}
