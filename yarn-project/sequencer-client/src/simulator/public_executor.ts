import {
  ContractDataSource,
  ExtendedContractData,
  L1ToL2MessageSource,
  MerkleTreeId,
  Tx,
  UnencryptedL2Log,
} from '@aztec/circuit-types';
import {
  AztecAddress,
  ContractClassRegisteredEvent,
  ContractInstanceDeployedEvent,
  EthAddress,
  Fr,
  FunctionSelector,
  L1_TO_L2_MSG_TREE_HEIGHT,
  PublicDataTreeLeafPreimage,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import { createDebugLogger } from '@aztec/foundation/log';
import { ClassRegistererAddress } from '@aztec/protocol-contracts/class-registerer';
import { InstanceDeployerAddress } from '@aztec/protocol-contracts/instance-deployer';
import { CommitmentsDB, MessageLoadOracleInputs, PublicContractsDB, PublicStateDB } from '@aztec/simulator';
import { ContractClassPublic, ContractInstanceWithAddress } from '@aztec/types/contracts';
import { MerkleTreeOperations } from '@aztec/world-state';

/**
 * Implements the PublicContractsDB using a ContractDataSource.
 * Progressively records contracts in transaction as they are processed in a block.
 */
export class ContractsDataSourcePublicDB implements PublicContractsDB {
  private cache = new Map<string, ExtendedContractData>();
  private instanceCache = new Map<string, ContractInstanceWithAddress>();
  private classCache = new Map<string, ContractClassPublic>();

  private log = createDebugLogger('aztec:sequencer:contracts-data-source');

  constructor(private db: ContractDataSource) {}

  /**
   * Add new contracts from a transaction
   * @param tx - The transaction to add contracts from.
   */
  public addNewContracts(tx: Tx): Promise<void> {
    for (const contract of tx.newContracts) {
      const contractAddress = contract.contractData.contractAddress;

      if (contractAddress.isZero()) {
        continue;
      }

      this.cache.set(contractAddress.toString(), contract);
    }

    // Extract contract class and instance data from logs and add to cache for this block
    const logs = tx.unencryptedLogs.unrollLogs().map(UnencryptedL2Log.fromBuffer);
    ContractClassRegisteredEvent.fromLogs(logs, ClassRegistererAddress).forEach(e => {
      this.log(`Adding class ${e.contractClassId.toString()} to public execution contract cache`);
      this.classCache.set(e.contractClassId.toString(), e.toContractClassPublic());
    });
    ContractInstanceDeployedEvent.fromLogs(logs, InstanceDeployerAddress).forEach(e => {
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
    for (const contract of tx.newContracts) {
      const contractAddress = contract.contractData.contractAddress;

      if (contractAddress.isZero()) {
        continue;
      }

      this.cache.delete(contractAddress.toString());
    }

    // TODO(@spalladino): Can this inadvertently delete a valid contract added by another tx?
    // Let's say we have two txs adding the same contract on the same block. If the 2nd one reverts,
    // wouldn't that accidentally remove the contract added on the first one?
    const logs = tx.unencryptedLogs.unrollLogs().map(UnencryptedL2Log.fromBuffer);
    ContractClassRegisteredEvent.fromLogs(logs, ClassRegistererAddress).forEach(e =>
      this.classCache.delete(e.contractClassId.toString()),
    );
    ContractInstanceDeployedEvent.fromLogs(logs, InstanceDeployerAddress).forEach(e =>
      this.instanceCache.delete(e.address.toString()),
    );
    return Promise.resolve();
  }

  async getBytecode(address: AztecAddress, selector: FunctionSelector): Promise<Buffer | undefined> {
    const contract = await this.#getContract(address);
    return contract?.getPublicFunction(selector)?.bytecode;
  }

  async getIsInternal(address: AztecAddress, selector: FunctionSelector): Promise<boolean | undefined> {
    const contract = await this.#getContract(address);
    return contract?.getPublicFunction(selector)?.isInternal;
  }

  async getPortalContractAddress(address: AztecAddress): Promise<EthAddress | undefined> {
    const contract = await this.#getContract(address);
    return contract?.contractData.portalContractAddress;
  }

  async #getContract(address: AztecAddress): Promise<ExtendedContractData | undefined> {
    return (
      this.cache.get(address.toString()) ??
      (await this.#makeExtendedContractDataFor(address)) ??
      (await this.db.getExtendedContractData(address))
    );
  }

  async #makeExtendedContractDataFor(address: AztecAddress): Promise<ExtendedContractData | undefined> {
    const instance = this.instanceCache.get(address.toString());
    if (!instance) {
      return undefined;
    }

    const contractClass =
      this.classCache.get(instance.contractClassId.toString()) ??
      (await this.db.getContractClass(instance.contractClassId));
    if (!contractClass) {
      this.log.warn(
        `Contract class ${instance.contractClassId.toString()} for address ${address.toString()} not found`,
      );
      return undefined;
    }

    return ExtendedContractData.fromClassAndInstance(contractClass, instance);
  }
}

/**
 * Implements the PublicStateDB using a world-state database.
 */
export class WorldStatePublicDB implements PublicStateDB {
  private commitedWriteCache: Map<bigint, Fr> = new Map();
  private uncommitedWriteCache: Map<bigint, Fr> = new Map();

  constructor(private db: MerkleTreeOperations) {}

  /**
   * Reads a value from public storage, returning zero if none.
   * @param contract - Owner of the storage.
   * @param slot - Slot to read in the contract storage.
   * @returns The current value in the storage slot.
   */
  public async storageRead(contract: AztecAddress, slot: Fr): Promise<Fr> {
    const leafSlot = computePublicDataTreeLeafSlot(contract, slot).value;
    const uncommited = this.uncommitedWriteCache.get(leafSlot);
    if (uncommited !== undefined) {
      return uncommited;
    }
    const commited = this.commitedWriteCache.get(leafSlot);
    if (commited !== undefined) {
      return commited;
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
    this.uncommitedWriteCache.set(index, newValue);
    return Promise.resolve();
  }

  /**
   * Commit the pending changes to the DB.
   * @returns Nothing.
   */
  commit(): Promise<void> {
    for (const [k, v] of this.uncommitedWriteCache) {
      this.commitedWriteCache.set(k, v);
    }
    return this.rollback();
  }

  /**
   * Rollback the pending changes.
   * @returns Nothing.
   */
  rollback(): Promise<void> {
    this.uncommitedWriteCache = new Map<bigint, Fr>();
    return Promise.resolve();
  }
}

/**
 * Implements WorldState db using a world state database.
 */
export class WorldStateDB implements CommitmentsDB {
  constructor(private db: MerkleTreeOperations, private l1ToL2MessageSource: L1ToL2MessageSource) {}

  public async getL1ToL2Message(messageKey: Fr): Promise<MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT>> {
    // todo: #697 - make this one lookup.
    const message = await this.l1ToL2MessageSource.getConfirmedL1ToL2Message(messageKey);
    const index = (await this.db.findLeafIndex(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, messageKey.toBuffer()))!;
    const siblingPath = await this.db.getSiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>(
      MerkleTreeId.L1_TO_L2_MESSAGE_TREE,
      index,
    );

    return new MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT>(message, index, siblingPath);
  }

  public async getCommitmentIndex(commitment: Fr): Promise<bigint | undefined> {
    return await this.db.findLeafIndex(MerkleTreeId.NOTE_HASH_TREE, commitment.toBuffer());
  }

  public async getNullifierIndex(nullifier: Fr): Promise<bigint | undefined> {
    return await this.db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
  }
}
