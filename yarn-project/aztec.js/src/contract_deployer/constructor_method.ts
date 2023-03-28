import { AztecRPCClient, ContractAbi } from '@aztec/aztec-rpc';
import { EthAddress, Fr } from '@aztec/circuits.js';
import { AztecAddress, randomBytes } from '@aztec/foundation';
import { ContractFunction, SendMethod, SendMethodOptions } from '../contract/index.js';

export interface ConstructorOptions extends SendMethodOptions {
  portalContract?: EthAddress;
  contractAddressSalt?: Fr;
}

/**
 * Extends the SendMethodInteraction to create TxRequest for constructors (deployments).
 */
export class ConstructorMethod extends SendMethod {
  constructor(
    arc: AztecRPCClient,
    private abi: ContractAbi,
    args: any[] = [],
    defaultOptions: ConstructorOptions = {},
  ) {
    const constructorAbi = abi.functions.find(f => f.name === 'constructor');
    if (!constructorAbi) {
      throw new Error('Cannot find constructor in the ABI.');
    }

    super(arc, AztecAddress.ZERO, new ContractFunction(constructorAbi), args, defaultOptions);
  }

  public async request(options: ConstructorOptions = {}) {
    const { portalContract, contractAddressSalt, from } = { ...this.defaultOptions, ...options };
    this.txRequest = await this.arc.createDeploymentTxRequest(
      this.abi,
      this.entry.encodeParameters(this.args).map(p => new Fr(p)),
      portalContract || new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES)),
      contractAddressSalt || new Fr(randomBytes(Fr.SIZE_IN_BYTES)),
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
