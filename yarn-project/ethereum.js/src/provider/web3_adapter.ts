import { EthereumProvider, RequestArguments } from './ethereum_provider.js';
import { Web3Provider } from './web3_provider.js';

/**
 * Adapts a legacy web3 provider into an EIP1193 compatible provider for injecting into the sdk.
 */
export class Web3Adapter implements EthereumProvider {
  private id = 0;

  constructor(private provider: Web3Provider) {}

  /**
   * Sends a JSON-RPC request to the legacy web3 provider and returns the result in a Promise.
   * The function constructs a payload object using the method and params provided in the RequestArguments,
   * and sends it to the provider for execution. It handles errors and responses accordingly, and
   * resolves or rejects the Promise based on the response from the provider.
   *
   * @param args - A RequestArguments object containing the JSON-RPC method and parameters.
   * @returns A Promise resolving with the result of the executed request or rejecting with an error.
   */
  public request(args: RequestArguments): Promise<any> {
    return new Promise((resolve, reject) => {
      const payload = {
        jsonrpc: '2.0',
        id: this.id++,
        method: args.method,
        params: args.params || [],
      };

      this.provider.send(payload, (err, response) => {
        if (err) {
          return reject(err);
        }
        if (!response) {
          return reject(new Error('No response.'));
        }
        resolve(response.result);
      });
    });
  }

  /**
   * Adds an event listener for the specified event on the Web3Adapter instance.
   * Please note that this method is not implemented and will throw an error when called, as events are not supported.
   *
   * @throws Will throw an error if the method is called, because events are not supported in this implementation.
   * @returns Returns the Web3Adapter instance for chaining purposes (if events were supported).
   */
  on(): this {
    throw new Error('Events not supported.');
  }

  /**
   * Remove an event listener from the Ethereum provider.
   * This method is not supported for the Web3Adapter class, and calling it will result in an error being thrown.
   *
   * @throws - An error indicating that event removal is not supported.
   * @returns  - The current instance of the Web3Adapter class.
   */
  removeListener(): this {
    throw new Error('Events not supported.');
  }
}
