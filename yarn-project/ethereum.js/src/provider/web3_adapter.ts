import { EthereumProvider, RequestArguments } from './ethereum_provider.js';
import { Web3Provider } from './web3_provider.js';

/**
 * Adapts a legacy web3 provider into an EIP1193 compatible provider for injecting into the sdk.
 */
export class Web3Adapter implements EthereumProvider {
  private id = 0;

  constructor(private provider: Web3Provider) {}

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

  on(): this {
    throw new Error('Events not supported.');
  }

  removeListener(): this {
    throw new Error('Events not supported.');
  }
}
