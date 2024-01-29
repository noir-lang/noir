import cors from '@koa/cors';
import http from 'http';
import Koa from 'koa';
import bodyParser from 'koa-bodyparser';
import compress from 'koa-compress';
import Router from 'koa-router';

import { createDebugLogger } from '../../log/index.js';
import { JsonClassConverterInput, StringClassConverterInput } from '../class_converter.js';
import { convertBigintsInObj } from '../convert.js';
import { ClassMaps, JsonProxy } from './json_proxy.js';

/**
 * JsonRpcServer.
 * Minimal, dev-friendly mechanism to create a server from an object.
 */
export class JsonRpcServer {
  /**
   * The proxy object.
   */
  public proxy: JsonProxy;
  constructor(
    private handler: object,
    private stringClassMap: StringClassConverterInput,
    private objectClassMap: JsonClassConverterInput,
    /** List of methods to disallow from calling remotely */
    public readonly disallowedMethods: string[] = [],
    private log = createDebugLogger('aztec:foundation:json-rpc:server'),
  ) {
    this.proxy = new JsonProxy(handler, stringClassMap, objectClassMap);
  }

  /**
   * Get an express app object.
   * @param prefix - Our server prefix.
   * @returns The app object.
   */
  public getApp(prefix = '') {
    const router = this.getRouter(prefix);
    const exceptionHandler = async (ctx: Koa.Context, next: () => Promise<void>) => {
      try {
        await next();
      } catch (err: any) {
        this.log.error(err);
        if (err instanceof SyntaxError) {
          ctx.status = 400;
          ctx.body = {
            jsonrpc: '2.0',
            id: null,
            error: {
              code: -32700,
              message: 'Parse error',
            },
          };
        } else {
          ctx.status = 500;
          ctx.body = {
            jsonrpc: '2.0',
            id: null,
            error: {
              code: -32603,
              message: 'Internal error',
            },
          };
        }
      }
    };
    const app = new Koa();
    app.on('error', error => {
      this.log.error(`Error on API handler: ${error}`);
    });
    app.use(exceptionHandler);
    app.use(compress({ br: false } as any));
    app.use(
      bodyParser({
        jsonLimit: '10mb',
        enableTypes: ['json'],
        detectJSON: () => true,
      }),
    );
    app.use(cors());
    app.use(router.routes());
    app.use(router.allowedMethods());

    return app;
  }

  /**
   * Get a router object wrapping our RPC class.
   * @param prefix - The server prefix.
   * @returns The router object.
   */
  private getRouter(prefix: string) {
    const router = new Router({ prefix });
    const proto = Object.getPrototypeOf(this.handler);
    // "JSON RPC mode" where a single endpoint is used and the method is given in the request body
    router.post('/', async (ctx: Koa.Context) => {
      const { params = [], jsonrpc, id, method } = ctx.request.body as any;
      // Ignore if not a function
      if (method === 'constructor' || typeof proto[method] !== 'function' || this.disallowedMethods.includes(method)) {
        ctx.status = 400;
        ctx.body = {
          jsonrpc,
          id,
          error: {
            code: -32601,
            message: `Method not found: ${method}`,
          },
        };
      } else {
        try {
          const result = await this.proxy.call(method, params);
          ctx.body = {
            jsonrpc,
            id,
            result: convertBigintsInObj(result),
          };
          ctx.status = 200;
        } catch (err: any) {
          // Propagate the error message to the client. Plenty of the errors are expected to occur (e.g. adding
          // a duplicate recipient) so this is necessary.
          ctx.status = 400;
          ctx.body = {
            jsonrpc,
            id,
            error: {
              // TODO assign error codes - https://github.com/AztecProtocol/aztec-packages/issues/2633
              code: -32000,
              message: err.message,
            },
          };
        }
      }
    });

    return router;
  }

  /**
   * Start this server with koa.
   * @param port - Port number.
   * @param prefix - Prefix string.
   */
  public start(port: number, prefix = '') {
    const httpServer = http.createServer(this.getApp(prefix).callback());
    httpServer.listen(port);
  }

  /**
   * Get a list of methods.
   * @returns A list of methods.
   */
  public getMethods(): string[] {
    return Object.getOwnPropertyNames(Object.getPrototypeOf(this.handler));
  }

  /**
   * Gets the class maps that were used to create the proxy.
   * @returns The string & object class maps.
   */
  public getClassMaps(): ClassMaps {
    return { stringClassMap: this.stringClassMap, objectClassMap: this.objectClassMap };
  }

  /**
   * Call an RPC method.
   * @param methodName - The RPC method.
   * @param jsonParams - The RPG parameters.
   * @param skipConversion - Whether to skip conversion of the parameters.
   * @returns The remote result.
   */
  public async call(methodName: string, jsonParams: any[] = [], skipConversion: boolean) {
    return await this.proxy.call(methodName, jsonParams, skipConversion);
  }
}

/**
 * Creates a router for handling a plain status request that will return 200 status when running.
 * @param apiPrefix - The prefix to use for all api requests
 * @returns - The router for handling status requests.
 */
export function createStatusRouter(apiPrefix = '') {
  const router = new Router({ prefix: `${apiPrefix}` });
  router.get('/status', (ctx: Koa.Context) => {
    ctx.status = 200;
  });
  return router;
}

/**
 * Creates an http server that forwards calls to the underlying instance and starts it on the given port.
 * @param instance - Instance to wrap in a JSON-RPC server.
 * @param jsonRpcFactoryFunc - Function that wraps the instance in a JSON-RPC server.
 * @param port - Port to listen in.
 * @returns A running http server.
 */
export function startHttpRpcServer<T>(
  name: string,
  instance: T,
  jsonRpcFactoryFunc: (instance: T) => JsonRpcServer,
  port: string | number,
): http.Server {
  const rpcServer = jsonRpcFactoryFunc(instance);

  const namespacedServer = createNamespacedJsonRpcServer([{ [name]: rpcServer }]);

  const app = namespacedServer.getApp();

  const httpServer = http.createServer(app.callback());
  httpServer.listen(port);

  return httpServer;
}
/**
 * List of namespace to server instance.
 */
export type ServerList = {
  /** name of the service to be used for namespacing */
  [name: string]: JsonRpcServer;
}[];

/**
 * Creates a single JsonRpcServer from multiple servers.
 * @param servers - List of servers to be combined into a single server, passed as ServerList.
 * @returns A single JsonRpcServer with namespaced methods.
 */
export function createNamespacedJsonRpcServer(
  servers: ServerList,
  log = createDebugLogger('aztec:foundation:json-rpc:multi-server'),
): JsonRpcServer {
  const handler = {} as any;
  const disallowedMethods: string[] = [];
  const classMapsArr: ClassMaps[] = [];

  for (const serverEntry of servers) {
    const [namespace, server] = Object.entries(serverEntry)[0];
    const serverMethods = server.getMethods();

    for (const method of serverMethods) {
      const namespacedMethod = `${namespace}_${method}`;

      handler[namespacedMethod] = (...args: any[]) => {
        return server.call(method, args, true);
      };
    }

    // get the combined disallowed methods from all servers.
    disallowedMethods.push(...server.disallowedMethods.map(method => `${namespace}_${method}`));
    // get the combined classmaps from all servers.
    const classMap = server.getClassMaps();
    classMapsArr.push({
      stringClassMap: classMap.stringClassMap,
      objectClassMap: classMap.objectClassMap,
    });
  }

  // Get the combined stringClassMap & objectClassMap from all servers
  const classMaps = classMapsArr.reduce(
    (acc, curr) => {
      return {
        stringClassMap: { ...acc.stringClassMap, ...curr.stringClassMap },
        objectClassMap: { ...acc.objectClassMap, ...curr.objectClassMap },
      };
    },
    { stringClassMap: {}, objectClassMap: {} } as ClassMaps,
  );

  return new JsonRpcServer(Object.create(handler), classMaps.stringClassMap, classMaps.objectClassMap, [], log);
}
