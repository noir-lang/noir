// comlink:
//  Dev dependency just for the somewhat complex RemoteObject type
//  This takes a {foo(): T} and makes {foo(): Promise<T>}
//  while avoiding Promise of Promise.
import { RemoteObject } from 'comlink';

import { DebugLogger, createDebugLogger } from '../../log/index.js';
import { NoRetryError, makeBackoff, retry } from '../../retry/index.js';
import { ClassConverter, JsonClassConverterInput, StringClassConverterInput } from '../class_converter.js';
import { JsonStringify, convertFromJsonObj, convertToJsonObj } from '../convert.js';

export { JsonStringify } from '../convert.js';

const debug = createDebugLogger('json-rpc:json_rpc_client');
/**
 * A normal fetch function that does not retry.
 * Alternatives are a fetch function with retries, or a mocked fetch.
 * @param host - The host URL.
 * @param method - The RPC method name.
 * @param body - The RPC payload.
 * @param noRetry - Whether to throw a `NoRetryError` in case the response is not ok and the body contains an error
 *                  message (see `retry` function for more details).
 * @returns The parsed JSON response, or throws an error.
 */
export async function defaultFetch(
  host: string,
  rpcMethod: string,
  body: any,
  useApiEndpoints: boolean,
  noRetry = false,
) {
  debug(`JsonRpcClient.fetch`, host, rpcMethod, '->', body);
  let resp: Response;
  if (useApiEndpoints) {
    resp = await fetch(`${host}/${rpcMethod}`, {
      method: 'POST',
      body: JsonStringify(body),
      headers: { 'content-type': 'application/json' },
    });
  } else {
    resp = await fetch(host, {
      method: 'POST',
      body: JsonStringify({ ...body, method: rpcMethod }),
      headers: { 'content-type': 'application/json' },
    });
  }

  let responseJson;
  try {
    responseJson = await resp.json();
  } catch (err) {
    if (!resp.ok) {
      throw new Error(resp.statusText);
    }
    throw new Error(`Failed to parse body as JSON: ${resp.text()}`);
  }
  if (!resp.ok) {
    if (noRetry) {
      throw new NoRetryError(responseJson.error);
    } else {
      throw new Error(responseJson.error);
    }
  }

  return responseJson;
}

/**
 * Makes a fetch function that retries based on the given attempts.
 * @param retries - Sequence of intervals (in seconds) to retry.
 * @param noRetry - Whether to stop retries on server errors.
 * @param log - Optional logger for logging attempts.
 * @returns A fetch function.
 */
export function makeFetch(retries: number[], noRetry: boolean, log?: DebugLogger) {
  return async (host: string, rpcMethod: string, body: any, useApiEndpoints: boolean) => {
    return await retry(
      () => defaultFetch(host, rpcMethod, body, useApiEndpoints, noRetry),
      'JsonRpcClient request',
      makeBackoff(retries),
      log,
    );
  };
}

/**
 * Creates a Proxy object that delegates over RPC and satisfies RemoteObject<T>.
 * The server should have ran new JsonRpcServer().
 */
export function createJsonRpcClient<T extends object>(
  host: string,
  stringClassMap: StringClassConverterInput,
  objectClassMap: JsonClassConverterInput,
  useApiEndpoints: boolean,
  fetch = defaultFetch,
) {
  const classConverter = new ClassConverter(stringClassMap, objectClassMap);
  let id = 0;
  const request = async (method: string, params: any[]): Promise<any> => {
    const body = {
      jsonrpc: '2.0',
      id: id++,
      method,
      params: params.map(param => convertToJsonObj(classConverter, param)),
    };
    debug(`JsonRpcClient.request`, method, '<-', params);
    const res = await fetch(host, method, body, useApiEndpoints);
    debug(`JsonRpcClient.result`, method, '->', res);
    if (res.error) {
      throw res.error;
    }
    if ([null, undefined, 'null', 'undefined'].includes(res.result)) {
      return;
    }
    return convertFromJsonObj(classConverter, res.result);
  };

  // Intercept any RPC methods with a proxy
  // This wraps 'request' with a method-call syntax wrapper
  return new Proxy(
    {},
    {
      get: (target, rpcMethod: string) => {
        if (['then', 'catch'].includes(rpcMethod)) return Reflect.get(target, rpcMethod);
        return (...params: any[]) => {
          debug(`JsonRpcClient.constructor`, 'proxy', rpcMethod, '<-', params);
          return request(rpcMethod, params);
        };
      },
    },
  ) as RemoteObject<T>;
}
