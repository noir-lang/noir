import { DBOracle } from '@aztec/acir-simulator';
import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { FunctionAbi } from '@aztec/noir-contracts';
import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { Database } from '../database/index.js';
import { KeyPair } from '../key_store/index.js';
import { AztecNode } from '@aztec/aztec-node';

export class SimulatorOracle implements DBOracle {
  constructor(
    private contractDataOracle: ContractDataOracle,
    private db: Database,
    private keyPair: KeyPair,
    private node: AztecNode,
  ) {}

  getSecretKey(_: AztecAddress, address: AztecAddress): Promise<Buffer> {
    if (!address.equals(this.keyPair.getPublicKey().toAddress())) {
      throw new Error('Only allow access to the secret keys of the tx creator.');
    }
    return this.keyPair.getPrivateKey();
  }

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

  async getFunctionABI(contractAddress: AztecAddress, functionSelector: Buffer): Promise<FunctionAbi> {
    return await this.contractDataOracle.getFunctionAbi(contractAddress, functionSelector);
  }

  async getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress> {
    return await this.contractDataOracle.getPortalContractAddress(contractAddress);
  }
}
