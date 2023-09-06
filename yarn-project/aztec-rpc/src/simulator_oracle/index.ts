import {
  CommitmentDataOracleInputs,
  DBOracle,
  FunctionAbiWithDebugMetadata,
  MessageLoadOracleInputs,
} from '@aztec/acir-simulator';
import {
  AztecAddress,
  CircuitsWasm,
  CompleteAddress,
  EthAddress,
  Fr,
  FunctionSelector,
  HistoricBlockData,
  PrivateKey,
  PublicKey,
} from '@aztec/circuits.js';
import { siloCommitment } from '@aztec/circuits.js/abis';
import { DataCommitmentProvider, KeyStore, L1ToL2MessageProvider } from '@aztec/types';

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
    private l1ToL2MessageProvider: L1ToL2MessageProvider,
    private dataTreeProvider: DataCommitmentProvider,
  ) {}

  getSecretKey(_contractAddress: AztecAddress, pubKey: PublicKey): Promise<PrivateKey> {
    return this.keyStore.getAccountPrivateKey(pubKey);
  }

  async getCompleteAddress(address: AztecAddress): Promise<CompleteAddress> {
    const completeAddress = await this.db.getCompleteAddress(address);
    if (!completeAddress)
      throw new Error(
        `Unknown complete address for address ${address.toString()}. Add the information to Aztec RPC server by calling server.registerRecipient(...) or server.registerAccount(...)`,
      );
    return completeAddress;
  }

  async getAuthWitness(messageHash: Fr): Promise<Fr[]> {
    const witness = await this.db.getAuthWitness(messageHash);
    if (!witness) throw new Error(`Unknown auth witness for message hash ${messageHash.toString()}`);
    return witness;
  }

  async getNotes(contractAddress: AztecAddress, storageSlot: Fr) {
    const noteDaos = await this.db.getNoteSpendingInfo(contractAddress, storageSlot);
    return noteDaos.map(({ contractAddress, storageSlot, nonce, notePreimage, siloedNullifier, index }) => ({
      contractAddress,
      storageSlot,
      nonce,
      preimage: notePreimage.items,
      siloedNullifier,
      // RPC Client can use this index to get full MembershipWitness
      index,
    }));
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
    const messageAndIndex = await this.l1ToL2MessageProvider.getL1ToL2MessageAndIndex(msgKey);
    const message = messageAndIndex.message.toFieldArray();
    const index = messageAndIndex.index;
    const siblingPath = await this.l1ToL2MessageProvider.getL1ToL2MessagesTreePath(index);
    return {
      message,
      siblingPath: siblingPath.toFieldArray(),
      index,
    };
  }

  /**
   * Retrieves the noir oracle data required to prove existence of a given commitment.
   * @param contractAddress - The contract Address.
   * @param innerCommitment - The key of the message being fetched.
   * @returns - A promise that resolves to the commitment data, a sibling path and the
   *            index of the message in the private data tree.
   */
  async getCommitmentOracle(contractAddress: AztecAddress, innerCommitment: Fr): Promise<CommitmentDataOracleInputs> {
    const siloedCommitment = siloCommitment(await CircuitsWasm.get(), contractAddress, innerCommitment);
    const index = await this.dataTreeProvider.findCommitmentIndex(siloedCommitment.toBuffer());
    if (!index) throw new Error(`Commitment not found ${siloedCommitment.toString()}`);

    const siblingPath = await this.dataTreeProvider.getDataTreePath(index);
    return await Promise.resolve({
      commitment: siloedCommitment,
      siblingPath: siblingPath.toFieldArray(),
      index,
    });
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
