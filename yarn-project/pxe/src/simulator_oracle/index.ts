import { DBOracle, FunctionAbiWithDebugMetadata, MessageLoadOracleInputs } from '@aztec/acir-simulator';
import {
  AztecAddress,
  CompleteAddress,
  EthAddress,
  Fr,
  FunctionSelector,
  GrumpkinPrivateKey,
  HistoricBlockData,
  PublicKey,
} from '@aztec/circuits.js';
import { KeyStore, MerkleTreeId, StateInfoProvider } from '@aztec/types';

import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { Database } from '../database/index.js';

/**
 * A data oracle that provides information needed for simulating a transaction.
 */
export class SimulatorOracle implements DBOracle {
  constructor(
    private contractDataOracle: ContractDataOracle,
    private db: Database,
    private keyStore: KeyStore,
    private stateInfoProvider: StateInfoProvider,
  ) {}

  getSecretKey(_contractAddress: AztecAddress, pubKey: PublicKey): Promise<GrumpkinPrivateKey> {
    return this.keyStore.getAccountPrivateKey(pubKey);
  }

  async getCompleteAddress(address: AztecAddress): Promise<CompleteAddress> {
    const completeAddress = await this.db.getCompleteAddress(address);
    if (!completeAddress)
      throw new Error(
        `Unknown complete address for address ${address.toString()}. Add the information to PXE Service by calling server.registerRecipient(...) or server.registerAccount(...)`,
      );
    return completeAddress;
  }

  async getAuthWitness(messageHash: Fr): Promise<Fr[]> {
    const witness = await this.db.getAuthWitness(messageHash);
    if (!witness) throw new Error(`Unknown auth witness for message hash ${messageHash.toString(true)}`);
    return witness;
  }

  async getNotes(contractAddress: AztecAddress, storageSlot: Fr) {
    const noteDaos = await this.db.getNoteSpendingInfo(contractAddress, storageSlot);
    return noteDaos.map(
      ({ contractAddress, storageSlot, nonce, notePreimage, innerNoteHash, siloedNullifier, index }) => ({
        contractAddress,
        storageSlot,
        nonce,
        preimage: notePreimage.items,
        innerNoteHash,
        siloedNullifier,
        // PXE can use this index to get full MembershipWitness
        index,
      }),
    );
  }

  async getFunctionABI(
    contractAddress: AztecAddress,
    selector: FunctionSelector,
  ): Promise<FunctionAbiWithDebugMetadata> {
    const abi = await this.contractDataOracle.getFunctionAbi(contractAddress, selector);
    const debug = await this.contractDataOracle.getFunctionDebugMetadata(contractAddress, selector);
    return {
      ...abi,
      debug,
    };
  }

  async getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress> {
    return await this.contractDataOracle.getPortalContractAddress(contractAddress);
  }

  /**
   * Retrieves the L1ToL2Message associated with a specific message key
   * Throws an error if the message key is not found
   *
   * @param msgKey - The key of the message to be retrieved
   * @returns A promise that resolves to the message data, a sibling path and the
   *          index of the message in the l1ToL2MessagesTree
   */
  async getL1ToL2Message(msgKey: Fr): Promise<MessageLoadOracleInputs> {
    const messageAndIndex = await this.stateInfoProvider.getL1ToL2MessageAndIndex(msgKey);
    const message = messageAndIndex.message.toFieldArray();
    const index = messageAndIndex.index;
    const siblingPath = await this.stateInfoProvider.getL1ToL2MessagesTreePath(index);
    return {
      message,
      siblingPath: siblingPath.toFieldArray(),
      index,
    };
  }

  /**
   * Gets the index of a commitment in the private data tree.
   * @param commitment - The commitment.
   * @returns - The index of the commitment. Undefined if it does not exist in the tree.
   */
  async getCommitmentIndex(commitment: Fr) {
    return await this.stateInfoProvider.findLeafIndex(MerkleTreeId.PRIVATE_DATA_TREE, commitment.toBuffer());
  }

  /**
   * Retrieve the databases view of the Historic Block Data object.
   * This structure is fed into the circuits simulator and is used to prove against certain historic roots.
   *
   * @returns A Promise that resolves to a HistoricBlockData object.
   */
  getHistoricBlockData(): Promise<HistoricBlockData> {
    return Promise.resolve(this.db.getHistoricBlockData());
  }
}
