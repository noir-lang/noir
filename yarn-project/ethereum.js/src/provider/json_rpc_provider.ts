import { EthereumProvider, RequestArguments } from './ethereum_provider.js';
import { retry } from '../retry/index.js';
import { createDebugLogger } from '../log/index.js';

const log = createDebugLogger('json_rpc_provider');

/**
 * The JsonRpcProvider class implements EthereumProvider to facilitate communication with Ethereum nodes.
 * It provides a request/response API using JSON-RPC 2.0 protocol, handling requests for blockchain data and network state.
 * This class does not support event subscriptions and will throw errors if event-related methods are called.
 * Uses fetch to perform HTTP POST requests and can be configured to automatically retry failed requests.
 */
export class JsonRpcProvider implements EthereumProvider {
  private id = 0;

  constructor(private host: string, private netMustSucceed = true) {}

  /**
   * Sends a JSON-RPC request to the Ethereum provider with the given method and parameters.
   * The 'method' should be a valid Ethereum JSON-RPC method, and 'params' should be an array of inputs required for that method.
   * Returns a promise which resolves to the result of the request, or rejects with an error if the request fails.
   *
   * @param requestArguments - An object containing 'method' and 'params' properties.
   * @param  method - The Ethereum JSON-RPC method to call.
   * @param params - The parameters required for the called method.
   * @returns A promise resolving to the result of the request, or rejecting with an error if the request fails.
   */
  public async request({ method, params }: RequestArguments): Promise<any> {
    const body = {
      jsonrpc: '2.0',
      id: this.id++,
      method,
      params,
    };
    log(`->`, body);
    const res = await this.fetch(body);
    log(`<-`, res);
    if (res.error) {
      throw res.error;
    }
    return res.result;
  }

  /**
   * Registers an event listener for the specified event on the JsonRpcProvider instance.
   * This method is not supported in the current implementation and will throw an error when called.
   *
   * @throws  An error indicating that events are not supported by the JsonRpcProvider.
   * @returns  The current JsonRpcProvider instance.
   */
  on(): this {
    throw new Error('Events not supported.');
  }

  /**
   * Remove an event listener from the Ethereum provider. This method is not supported by JsonRpcProvider
   * and will throw an error when called. To use event handling, consider using a different provider implementation.
   *
   * @throws Throws an error indicating that events are not supported by this provider.
   * @returns  Returns the current instance of the class for chaining purposes.
   */
  removeListener(): this {
    throw new Error('Events not supported.');
  }

  /**
   * Send a JSON-RPC request to the Ethereum node and return the parsed response.
   * The 'body' parameter contains the JSON-RPC request payload, including method, id, and params.
   * If 'netMustSucceed' is true, the function will be retried until the request succeeds.
   * Throws an error if the response status is not ok or if the response body cannot be parsed as JSON.
   *
   * @param body - The JSON-RPC request payload containing the method, id, and params.
   * @returns A Promise resolving to the parsed JSON-RPC response object.
   */
  private async fetch(body: any) {
    const fn = async () => {
      const resp = await fetch(this.host, {
        method: 'POST',
        body: JSON.stringify(body),
        headers: { 'content-type': 'application/json' },
      });

      if (!resp.ok) {
        throw new Error(resp.statusText);
      }

      const text = await resp.text();
      try {
        return JSON.parse(text);
      } catch (err) {
        throw new Error(`Failed to parse body as JSON: ${text}`);
      }
    };

    if (this.netMustSucceed) {
      return await retry(fn, 'JsonRpcProvider request');
    }

    return await fn();
  }
}
