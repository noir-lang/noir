import {
  type AztecNode,
  type KeyStore,
  type L2Block,
  MerkleTreeId,
  type NoteStatus,
  type NullifierMembershipWitness,
  type PublicDataWitness,
  type SiblingPath,
} from '@aztec/circuit-types';
import {
  type AztecAddress,
  type CompleteAddress,
  type EthAddress,
  type Fr,
  type FunctionSelector,
  type Header,
  type L1_TO_L2_MSG_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { computeL1ToL2MessageNullifier } from '@aztec/circuits.js/hash';
import { type FunctionArtifactWithDebugMetadata, getFunctionArtifactWithDebugMetadata } from '@aztec/foundation/abi';
import { createDebugLogger } from '@aztec/foundation/log';
import { type DBOracle, type KeyPair, MessageLoadOracleInputs } from '@aztec/simulator';
import { type ContractInstance } from '@aztec/types/contracts';

import { type ContractDataOracle } from '../contract_data_oracle/index.js';
import { type PxeDatabase } from '../database/index.js';

/**
 * A data oracle that provides information needed for simulating a transaction.
 */
export class SimulatorOracle implements DBOracle {
  constructor(
    private contractDataOracle: ContractDataOracle,
    private db: PxeDatabase,
    private keyStore: KeyStore,
    private aztecNode: AztecNode,
    private log = createDebugLogger('aztec:pxe:simulator_oracle'),
  ) {}

  async getNullifierKeyPair(accountAddress: AztecAddress, contractAddress: AztecAddress): Promise<KeyPair> {
    const accountPublicKey = (await this.db.getCompleteAddress(accountAddress))!.publicKey;
    const publicKey = await this.keyStore.getNullifierPublicKey(accountPublicKey);
    const secretKey = await this.keyStore.getSiloedNullifierSecretKey(accountPublicKey, contractAddress);
    return { publicKey, secretKey };
  }

  async getCompleteAddress(address: AztecAddress): Promise<CompleteAddress> {
    const completeAddress = await this.db.getCompleteAddress(address);
    if (!completeAddress) {
      throw new Error(
        `No public key registered for address ${address.toString()}. Register it by calling pxe.registerRecipient(...) or pxe.registerAccount(...).\nSee docs for context: https://docs.aztec.network/developers/debugging/aztecnr-errors#simulation-error-No-public-key-registered-for-address-0x0-Register-it-by-calling-pxeregisterRecipient-or-pxeregisterAccount`,
      );
    }
    return completeAddress;
  }

  async getContractInstance(address: AztecAddress): Promise<ContractInstance> {
    const instance = await this.db.getContractInstance(address);
    if (!instance) {
      throw new Error(`No contract instance found for address ${address.toString()}`);
    }
    return instance;
  }

  async getAuthWitness(messageHash: Fr): Promise<Fr[]> {
    const witness = await this.db.getAuthWitness(messageHash);
    if (!witness) {
      throw new Error(`Unknown auth witness for message hash ${messageHash.toString()}`);
    }
    return witness;
  }

  async popCapsule(): Promise<Fr[]> {
    const capsule = await this.db.popCapsule();
    if (!capsule) {
      throw new Error(`No capsules available`);
    }
    return capsule;
  }

  async getNotes(contractAddress: AztecAddress, storageSlot: Fr, status: NoteStatus) {
    const noteDaos = await this.db.getNotes({
      contractAddress,
      storageSlot,
      status,
    });
    return noteDaos.map(({ contractAddress, storageSlot, nonce, note, innerNoteHash, siloedNullifier, index }) => ({
      contractAddress,
      storageSlot,
      nonce,
      note,
      innerNoteHash,
      siloedNullifier,
      // PXE can use this index to get full MembershipWitness
      index,
    }));
  }

  async getFunctionArtifact(
    contractAddress: AztecAddress,
    selector: FunctionSelector,
  ): Promise<FunctionArtifactWithDebugMetadata> {
    const artifact = await this.contractDataOracle.getFunctionArtifact(contractAddress, selector);
    const debug = await this.contractDataOracle.getFunctionDebugMetadata(contractAddress, selector);
    return {
      ...artifact,
      debug,
    };
  }

  async getFunctionArtifactByName(
    contractAddress: AztecAddress,
    functionName: string,
  ): Promise<FunctionArtifactWithDebugMetadata | undefined> {
    const instance = await this.contractDataOracle.getContractInstance(contractAddress);
    const artifact = await this.contractDataOracle.getContractArtifact(instance.contractClassId);
    return artifact && getFunctionArtifactWithDebugMetadata(artifact, functionName);
  }

  async getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress> {
    return await this.contractDataOracle.getPortalContractAddress(contractAddress);
  }

  /**
   * Fetches a message from the db, given its key.
   * @param contractAddress - Address of a contract by which the message was emitted.
   * @param messageHash - Hash of the message.
   * @param secret - Secret used to compute a nullifier.
   * @dev Contract address and secret are only used to compute the nullifier to get non-nullified messages
   * @returns The l1 to l2 membership witness (index of message in the tree and sibling path).
   */
  async getL1ToL2MembershipWitness(
    contractAddress: AztecAddress,
    messageHash: Fr,
    secret: Fr,
  ): Promise<MessageLoadOracleInputs<typeof L1_TO_L2_MSG_TREE_HEIGHT>> {
    let nullifierIndex: bigint | undefined;
    let messageIndex = 0n;
    let startIndex = 0n;
    let siblingPath: SiblingPath<typeof L1_TO_L2_MSG_TREE_HEIGHT>;

    // We iterate over messages until we find one whose nullifier is not in the nullifier tree --> we need to check
    // for nullifiers because messages can have duplicates.
    do {
      const response = await this.aztecNode.getL1ToL2MessageMembershipWitness('latest', messageHash, startIndex);
      if (!response) {
        throw new Error(`No non-nullified L1 to L2 message found for message hash ${messageHash.toString()}`);
      }
      [messageIndex, siblingPath] = response;

      const messageNullifier = computeL1ToL2MessageNullifier(contractAddress, messageHash, secret, messageIndex);
      nullifierIndex = await this.getNullifierIndex(messageNullifier);

      startIndex = messageIndex + 1n;
    } while (nullifierIndex !== undefined);

    // Assuming messageIndex is what you intended to use for the index in MessageLoadOracleInputs
    return new MessageLoadOracleInputs(messageIndex, siblingPath);
  }

  /**
   * Gets the index of a commitment in the note hash tree.
   * @param commitment - The commitment.
   * @returns - The index of the commitment. Undefined if it does not exist in the tree.
   */
  async getCommitmentIndex(commitment: Fr) {
    return await this.aztecNode.findLeafIndex('latest', MerkleTreeId.NOTE_HASH_TREE, commitment);
  }

  async getNullifierIndex(nullifier: Fr) {
    return await this.aztecNode.findLeafIndex('latest', MerkleTreeId.NULLIFIER_TREE, nullifier);
  }

  public async findLeafIndex(blockNumber: number, treeId: MerkleTreeId, leafValue: Fr): Promise<bigint | undefined> {
    return await this.aztecNode.findLeafIndex(blockNumber, treeId, leafValue);
  }

  public async getSiblingPath(blockNumber: number, treeId: MerkleTreeId, leafIndex: bigint): Promise<Fr[]> {
    switch (treeId) {
      case MerkleTreeId.NULLIFIER_TREE:
        return (await this.aztecNode.getNullifierSiblingPath(blockNumber, leafIndex)).toFields();
      case MerkleTreeId.NOTE_HASH_TREE:
        return (await this.aztecNode.getNoteHashSiblingPath(blockNumber, leafIndex)).toFields();
      case MerkleTreeId.PUBLIC_DATA_TREE:
        return (await this.aztecNode.getPublicDataSiblingPath(blockNumber, leafIndex)).toFields();
      case MerkleTreeId.ARCHIVE:
        return (await this.aztecNode.getArchiveSiblingPath(blockNumber, leafIndex)).toFields();
      default:
        throw new Error('Not implemented');
    }
  }

  public async getNullifierMembershipWitnessAtLatestBlock(nullifier: Fr) {
    return this.getNullifierMembershipWitness(await this.getBlockNumber(), nullifier);
  }

  public getNullifierMembershipWitness(
    blockNumber: number,
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    return this.aztecNode.getNullifierMembershipWitness(blockNumber, nullifier);
  }

  public getLowNullifierMembershipWitness(
    blockNumber: number,
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    return this.aztecNode.getLowNullifierMembershipWitness(blockNumber, nullifier);
  }

  public async getBlock(blockNumber: number): Promise<L2Block | undefined> {
    return await this.aztecNode.getBlock(blockNumber);
  }

  public async getPublicDataTreeWitness(blockNumber: number, leafSlot: Fr): Promise<PublicDataWitness | undefined> {
    return await this.aztecNode.getPublicDataTreeWitness(blockNumber, leafSlot);
  }

  /**
   * Retrieve the databases view of the Block Header object.
   * This structure is fed into the circuits simulator and is used to prove against certain historical roots.
   *
   * @returns A Promise that resolves to a Header object.
   */
  getHeader(): Promise<Header> {
    return Promise.resolve(this.db.getHeader());
  }

  /**
   * Fetches the current block number.
   * @returns The block number.
   */
  public async getBlockNumber(): Promise<number> {
    return await this.aztecNode.getBlockNumber();
  }
}
