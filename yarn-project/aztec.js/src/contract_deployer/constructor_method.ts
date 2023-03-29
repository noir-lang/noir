import { AztecAddress, AztecRPCClient, ContractAbi, EthAddress, Fr, FunctionType } from '@aztec/aztec-rpc';
import { ContractFunctionInteraction, SendMethodOptions } from '../contract/index.js';

export interface ConstructorOptions extends SendMethodOptions {
  portalContract?: EthAddress;
  contractAddressSalt?: Fr;
}

/**
 * Extends the SendMethodInteraction to create TxRequest for constructors (deployments).
 */
export class ConstructorMethod extends ContractFunctionInteraction {
  constructor(arc: AztecRPCClient, private abi: ContractAbi, args: any[] = []) {
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    super(arc, AztecAddress.ZERO, 'constructor', args, FunctionType.SECRET);
  }

  public async request(options: ConstructorOptions = {}) {
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

  public sign(options: ConstructorOptions = {}) {
    return super.sign(options);
  }

  public create(options: ConstructorOptions = {}) {
    return super.create(options);
  }

  public send(options: ConstructorOptions = {}) {
    return super.send(options);
  }
}
