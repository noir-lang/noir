import { AztecAddress, AztecRPCClient, EthAddress, Fr } from '@aztec/aztec-rpc';
import { ContractAbi, FunctionType } from '@aztec/noir-contracts';
import { ContractFunctionInteraction, SendMethodOptions } from '../contract/index.js';

export interface DeployOptions extends SendMethodOptions {
  portalContract?: EthAddress;
  contractAddressSalt?: Fr;
}

/**
 * Creates a TxRequest from a contract ABI, for contract deployment.
 * Extends the ContractFunctionInteraction class.
 */
export class DeployMethod extends ContractFunctionInteraction {
  constructor(arc: AztecRPCClient, private abi: ContractAbi, args: any[] = []) {
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    super(arc, AztecAddress.ZERO, 'constructor', args, FunctionType.SECRET);
  }

  public async request(options: DeployOptions = {}) {
    const { portalContract, contractAddressSalt, from } = options;
    this.txRequest = await this.arc.createDeploymentTxRequest(
      this.abi,
      [],
      portalContract || new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES)),
      contractAddressSalt || Fr.random(),
      from || AztecAddress.ZERO,
    );
    return this.txRequest;
  }

  public sign(options: DeployOptions = {}) {
    return super.sign(options);
  }

  public create(options: DeployOptions = {}) {
    return super.create(options);
  }

  public send(options: DeployOptions = {}) {
    return super.send(options);
  }
}
