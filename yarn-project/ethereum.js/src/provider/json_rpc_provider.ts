import { EthereumProvider, RequestArguments } from './ethereum_provider.js';
import { retry } from '../retry/index.js';
import { createDebugLogger } from '../log/index.js';

const log = createDebugLogger('json_rpc_provider');

export class JsonRpcProvider implements EthereumProvider {
  private id = 0;

  constructor(private host: string, private netMustSucceed = true) {}

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

  on(): this {
    throw new Error('Events not supported.');
  }

  removeListener(): this {
    throw new Error('Events not supported.');
  }

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
