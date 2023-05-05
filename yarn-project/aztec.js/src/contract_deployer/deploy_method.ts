import { AztecRPCClient } from '@aztec/aztec-rpc';
import { ContractAbi, FunctionType } from '@aztec/foundation/abi';
import { ContractFunctionInteraction, SendMethodOptions } from '../contract/index.js';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { AztecAddress } from '@aztec/foundation/aztec-address';

/**
 * Options for deploying a contract on the Aztec network.
 * Allows specifying a portal contract, contract address salt, and additional send method options.
 */
export interface DeployOptions extends SendMethodOptions {
  /**
   * The Ethereum address of the Portal contract.
   */
  portalContract?: EthAddress;
  /**
   * An optional salt value used to deterministically calculate the contract address.
   */
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

  /**
   * Asynchronously create a transaction request for contract deployment using the provided options.
   * The function uses the deployment options to set up the portalContract, contractAddressSalt, and sender's address (from).
   * If no portalContract is provided, an EthAddress instance with zeroed-out bytes is used.
   * This transaction request can then be signed and sent to deploy the contract.
   *
   * @param options - An object containing optional parameters for the deployment request.
   * @returns A Promise that resolves to the created TxRequest instance.
   */
  public async request(options: DeployOptions = {}) {
    const { portalContract, contractAddressSalt, from } = options;
    this.txRequest = await this.arc.createDeploymentTxRequest(
      this.abi,
      this.args,
      portalContract || new EthAddress(Buffer.alloc(EthAddress.SIZE_IN_BYTES)),
      contractAddressSalt,
      from,
    );
    return this.txRequest;
  }

  /**
   * Signs the deployment transaction request using the provided options.
   * This function is used to generate a signature for the deployment transaction
   * before sending it to the network. The signature is derived from the caller's private key
   * and is used to authorize the deployment of the contract.
   *
   * @param options - Optional configuration object containing portalContract, contractAddressSalt, and from properties.
   * @returns A signed transaction object ready to be sent to the network.
   */
  public sign(options: DeployOptions = {}) {
    return super.sign(options);
  }

  /**
   * Create a contract deployment transaction, given the deployment options.
   * This function internally calls `request()` and `sign()` methods to prepare
   * the transaction for deployment. The resulting signed transaction can be
   * later sent using the `send()` method.
   *
   * @param options - An object containing optional deployment settings, including portalContract, contractAddressSalt, and from.
   * @returns A Promise resolving to an object containing the signed transaction data and other relevant information.
   */
  public create(options: DeployOptions = {}) {
    return super.create(options);
  }

  /**
   * Send the contract deployment transaction using the provided options.
   * This function extends the 'send' method from the ContractFunctionInteraction class,
   * allowing us to send a transaction specifically for contract deployment.
   *
   * @param options - An object containing various deployment options such as portalContract, contractAddressSalt, and from.
   * @returns A Promise that resolves to the transaction receipt upon successful deployment.
   */
  public send(options: DeployOptions = {}) {
    return super.send(options);
  }
}
