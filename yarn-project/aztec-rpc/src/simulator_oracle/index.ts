import { DBOracle, NoteLoadOracleInputs } from '@aztec/acir-simulator';
import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { FunctionAbi } from '@aztec/noir-contracts';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { Database } from '../database/index.js';
import { KeyPair } from '../key_store/index.js';

export class SimulatorOracle implements DBOracle {
  constructor(private contractDataOracle: ContractDataOracle, private db: Database, private keyPair: KeyPair) {}

  getSecretKey(_: AztecAddress, address: AztecAddress): Promise<Buffer> {
    if (!address.equals(this.keyPair.getPublicKey().toAddress())) {
      throw new Error('Only allow access to the secret keys of the tx creator.');
    }
    return this.keyPair.getPrivateKey();
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
    return await this.contractDataOracle.getFunctionAbi(contractAddress, functionSelector);
  }

  async getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress> {
    return await this.contractDataOracle.getPortalContractAddress(contractAddress);
  }
}
