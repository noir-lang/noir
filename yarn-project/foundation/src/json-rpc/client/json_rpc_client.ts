// comlink:
//  Dev dependency just for the somewhat complex RemoteObject type
//  This takes a {foo(): T} and makes {foo(): Promise<T>}
//  while avoiding Promise of Promise.
import { RemoteObject } from 'comlink';
import { createDebugLogger } from '../../log/index.js';
import { retry } from '../../retry/index.js';
import { ClassConverter, ClassConverterInput } from '../class_converter.js';
import { convertFromJsonObj, convertToJsonObj } from '../convert.js';

const debug = createDebugLogger('json-rpc:json_rpc_client');
/**
 * A normal fetch function that does not retry.
 * Alternatives are a fetch function with retries, or a mocked fetch.
 * @param host - The host URL.
 * @param method - The RPC method name.
 * @param body - The RPC payload.
 * @returns The parsed JSON response, or throws an error.
 */
export async function defaultFetch(host: string, rpcMethod: string, body: any) {
  debug(`JsonRpcClient.fetch`, host, rpcMethod, '<-', body);
  const resp = await fetch(`${host}/${rpcMethod}`, {
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
}

/**
 * A fetch function with retries.
 */
export async function mustSucceedFetch(host: string, rpcMethod: string, body: any) {
  return await retry(() => defaultFetch(host, rpcMethod, body), 'JsonRpcClient request');
}

/**
 * Creates a Proxy object that delegates over RPC and satisfies RemoteObject<T>.
 * The server should have ran new JsonRpcServer().
 */
export function createJsonRpcClient<T extends object>(
  host: string,
  classMap: ClassConverterInput,
  fetch = defaultFetch,
) {
  const classConverter = new ClassConverter(classMap);
  let id = 0;
  const request = async (method: string, params: any[]): Promise<any> => {
    const body = {
      jsonrpc: '2.0',
      id: id++,
      method,
      params: params.map(param => convertToJsonObj(classConverter, param)),
    };
    debug(`JsonRpcClient.request`, method, '<-', params);
    const res = await fetch(host, method, body);
    debug(`JsonRpcClient.request`, method, '->', res);
    if (res.error) {
      throw res.error;
    }
    return convertFromJsonObj(classConverter, res.result);
  };

  // Intercept any RPC methods with a proxy
  // This wraps 'request' with a method-call syntax wrapper
  return new Proxy(
    {},
    {
      get:
        (_, rpcMethod: string) =>
        (...params: any[]) => {
          debug(`JsonRpcClient.constructor`, 'proxy', rpcMethod, '<-', params);
          return request(rpcMethod, params);
        },
    },
  ) as RemoteObject<T>;
}
