import { DBOracle, NoteLoadOracleInputs } from '@aztec/acir-simulator';
import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { Database } from '../database/database.js';
import { KeyStore } from '../key_store/key_store.js';
import { FunctionAbi } from '@aztec/noir-contracts';

export class SimulatorOracle implements DBOracle {
  constructor(private db: Database, private keyStore: KeyStore) {}

  getSecretKey(_: AztecAddress, address: AztecAddress): Promise<Buffer> {
    return this.keyStore.getAccountPrivateKey(address);
  }

  async getNotes(contractAddress: AztecAddress, storageSlot: Fr, n: number): Promise<NoteLoadOracleInputs[]> {
    const noteDaos = await this.db.getTxAuxData(contractAddress, storageSlot);
    return noteDaos.slice(0, n).map(noteDao => ({
      preimage: noteDao.notePreimage.items,
      siblingPath: [], // TODO get this from node
      index: noteDao.index,
    }));
  }

  async getFunctionABI(contractAddress: AztecAddress, functionSelector: Buffer): Promise<FunctionAbi> {
    const contract = await this.db.getContract(contractAddress);
    if (!contract) {
      throw new Error(`Contract ${contractAddress} not found`);
    }

    const storedFunction = contract.functions.find(f => f.selector === functionSelector);
    if (!storedFunction) {
      throw new Error(`Function ${functionSelector} not found`);
    }

    return storedFunction;
  }

  async getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress> {
    const contract = await this.db.getContract(contractAddress);
    if (!contract) {
      throw new Error(`Contract ${contractAddress} not found`);
    }
    return contract.portalContract;
  }
}
