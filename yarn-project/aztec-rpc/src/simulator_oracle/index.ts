import { CommitmentDataOracleInputs, DBOracle, MessageLoadOracleInputs } from '@aztec/acir-simulator';
import {
  AztecAddress,
  CircuitsWasm,
  EthAddress,
  Fr,
  PartialContractAddress,
  Point,
  PrivateHistoricTreeRoots,
} from '@aztec/circuits.js';
import { siloCommitment } from '@aztec/circuits.js/abis';
import { FunctionAbi } from '@aztec/foundation/abi';
import { KeyStore, MerkleTreeId } from '@aztec/types';
import { DataCommitmentProvider, L1ToL2MessageProvider } from '@aztec/types';

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

  /**
   * Retrieve the secret key associated with a specific public key.
   * The function only allows access to the secret keys of the transaction creator,
   * and throws an error if the address does not match the public key address of the key pair.
   *
   * @param _contractAddress - The contract address. Ignored here. But we might want to return different keys for different contracts.
   * @param pubKey - The public key of an account.
   * @returns A Promise that resolves to the secret key as a Buffer.
   * @throws An Error if the input address does not match the public key address of the key pair.
   */
  getSecretKey(_contractAddress: AztecAddress, pubKey: Point): Promise<Buffer> {
    return this.keyStore.getAccountPrivateKey(pubKey);
  }

  /**
   * Retrieve the public key associated to a given address.
   * @param address - Address to fetch the pubkey for.
   * @returns A public key and the corresponding partial contract address, such that the hash of the two resolves to the input address.
   */
  async getPublicKey(address: AztecAddress): Promise<[Point, PartialContractAddress]> {
    const result = await this.db.getPublicKey(address);
    if (!result) throw new Error(`Unknown public key for address ${address.toString()}`);
    return result;
  }

  /**
   * Retrieves a set of notes stored in the database for a given contract address and storage slot.
   * The query result is paginated using 'limit' and 'offset' values.
   * Returns an object containing an array of note data, including preimage, nonce, and index for each note.
   *
   * @param contractAddress - The AztecAddress instance representing the contract address.
   * @param storageSlot - The Fr instance representing the storage slot of the notes.
   * @param sortBy - An array of indices of the fields to sort.
   * @param sortOrder - The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * @param limit - The number of notes to retrieve per query (pagination limit).
   * @returns A Promise that resolves to an array of note data.
   */
  async getNotes(contractAddress: AztecAddress, storageSlot: Fr, sortBy: number[], sortOrder: number[], limit: number) {
    const noteDaos = await this.db.getNoteSpendingInfo(contractAddress, storageSlot, {
      sortBy,
      sortOrder,
      limit,
    });
    return noteDaos.map(({ nonce, notePreimage, index }) => ({
      nonce,
      preimage: notePreimage.items,
      // RPC Client can use this index to get full MembershipWitness
      index,
    }));
  }

  /**
   * Retrieve the ABI information of a specific function within a contract.
   * The function is identified by its selector, which is a unique identifier generated from the function signature.
   *
   * @param contractAddress - The contract address.
   * @param functionSelector - The Buffer containing the function selector bytes.
   * @returns A Promise that resolves to a FunctionAbi object containing the ABI information of the target function.
   */
  async getFunctionABI(contractAddress: AztecAddress, functionSelector: Buffer): Promise<FunctionAbi> {
    return await this.contractDataOracle.getFunctionAbi(contractAddress, functionSelector);
  }

  /**
   * Retrieves the portal contract address associated with the given contract address.
   * Throws an error if the input contract address is not found or invalid.
   *
   * @param contractAddress - The address of the contract whose portal address is to be fetched.
   * @returns A Promise that resolves to an EthAddress instance, representing the portal contract address.
   */
  async getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress> {
    return await this.contractDataOracle.getPortalContractAddress(contractAddress);
  }

  /**
   * Retreives the L1ToL2Message associated with a specific message key
   * Throws an error if the message key is not found
   *
   * @param msgKey - The key of the message to be retreived
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
   * @param commitment - The key of the message being fetched.
   * @returns - A promise that resolves to the commitment data, a sibling path and the
   *            index of the message in the private data tree.
   */
  async getCommitmentOracle(contractAddress: AztecAddress, commitment: Fr): Promise<CommitmentDataOracleInputs> {
    const siloedCommitment = siloCommitment(await CircuitsWasm.get(), contractAddress, commitment);
    const index = await this.dataTreeProvider.findCommitmentIndex(siloedCommitment.toBuffer());
    if (!index) throw new Error('Commitment not found');

    const siblingPath = await this.dataTreeProvider.getDataTreePath(index);
    return await Promise.resolve({
      commitment: siloedCommitment,
      siblingPath: siblingPath.toFieldArray(),
      index,
    });
  }

  getTreeRoots(): PrivateHistoricTreeRoots {
    const roots = this.db.getTreeRoots();

    return PrivateHistoricTreeRoots.from({
      privateKernelVkTreeRoot: Fr.ZERO,
      privateDataTreeRoot: roots[MerkleTreeId.PRIVATE_DATA_TREE],
      contractTreeRoot: roots[MerkleTreeId.CONTRACT_TREE],
      nullifierTreeRoot: roots[MerkleTreeId.NULLIFIER_TREE],
      l1ToL2MessagesTreeRoot: roots[MerkleTreeId.L1_TO_L2_MESSAGES_TREE],
    });
  }
}
