import { DBOracle } from '@aztec/acir-simulator';
import { AztecNode } from '@aztec/aztec-node';
import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { KeyPair } from '@aztec/key-store';
import { FunctionAbi } from '@aztec/foundation/abi';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { Database } from '../database/index.js';

/**
 * A data oracle that provides information needed for simulating a transaction.
 */
export class SimulatorOracle implements DBOracle {
  constructor(
    private contractDataOracle: ContractDataOracle,
    private db: Database,
    private keyPair: KeyPair,
    private node: AztecNode,
  ) {}

  /**
   * Retrieve the secret key associated with a specific address.
   * The function only allows access to the secret keys of the transaction creator,
   * and throws an error if the address does not match the public key address of the key pair.
   *
   * @param _ - The contract address. Ignored here. But we might want to return different keys for different contracts.
   * @param address - The address of an account.
   * @returns A Promise that resolves to the secret key as a Buffer.
   * @throws An Error if the input address does not match the public key address of the key pair.
   */
  getSecretKey(_: AztecAddress, address: AztecAddress): Promise<Buffer> {
    if (!address.equals(this.keyPair.getPublicKey().toAddress())) {
      throw new Error('Only allow access to the secret keys of the tx creator.');
    }
    return this.keyPair.getPrivateKey();
  }

  /**
   * Retrieves a set of notes stored in the database for a given contract address and storage slot.
   * The query result is paginated using 'limit' and 'offset' values.
   * Returns an object containing the total count of notes and an array of note data, including preimage,
   * sibling path, and index for each note.
   *
   * @param contractAddress - The AztecAddress instance representing the contract address.
   * @param storageSlot - The Fr instance representing the storage slot of the notes.
   * @param limit - The number of notes to retrieve per query (pagination limit).
   * @param offset - The starting index for pagination.
   * @returns A Promise that resolves to an object with properties 'count' and 'notes'.
   */
  async getNotes(contractAddress: AztecAddress, storageSlot: Fr, limit: number, offset: number) {
    const noteDaos = await this.db.getTxAuxData(contractAddress, storageSlot);
    return {
      count: noteDaos.length,
      notes: await Promise.all(
        noteDaos.slice(offset, offset + limit).map(async noteDao => {
          const path = await this.node.getDataTreePath(noteDao.index);
          return {
            preimage: noteDao.notePreimage.items,
            siblingPath: path.data.map(buf => Fr.fromBuffer(buf)),
            index: noteDao.index,
          };
        }),
      ),
    };
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
}
