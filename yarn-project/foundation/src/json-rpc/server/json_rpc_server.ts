import cors from '@koa/cors';
import http from 'http';
import Koa from 'koa';
import bodyParser from 'koa-bodyparser';
import compress from 'koa-compress';
import Router from 'koa-router';

import { createDebugLogger } from '../../log/index.js';
import { JsonClassConverterInput, StringClassConverterInput } from '../class_converter.js';
import { convertBigintsInObj } from '../convert.js';
import { JsonProxy } from './json_proxy.js';

/**
 * JsonRpcServer.
 * Minimal, dev-friendly mechanism to create a server from an object.
 */
export class JsonRpcServer {
  proxy: JsonProxy;
  constructor(
    private handler: object,
    stringClassMap: StringClassConverterInput,
    objectClassMap: JsonClassConverterInput,
    private createApi: boolean,
    private disallowedMethods: string[] = [],
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
        this.log(err);
        ctx.status = 400;
        ctx.body = { error: err.message };
      }
    };
    const app = new Koa();
    app.on('error', error => {
      this.log(`KOA app-level error. ${convertBigintsInObj({ error })}`);
    });
    app.use(compress({ br: false } as any));
    app.use(bodyParser());
    app.use(cors());
    app.use(exceptionHandler);
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
    // Find all our endpoints from the handler methods

    if (this.createApi) {
      // "API mode" where an endpoint is created for each method
      for (const method of Object.getOwnPropertyNames(proto)) {
        // Ignore if not a function or function is not allowed
        if (
          method === 'constructor' ||
          typeof proto[method] !== 'function' ||
          this.disallowedMethods.includes(method)
        ) {
          continue;
        }
        router.post(`/${method}`, async (ctx: Koa.Context) => {
          const { params = [], jsonrpc, id } = ctx.request.body as any;
          const result = await this.proxy.call(method, params);
          ctx.body = {
            jsonrpc,
            id,
            result: convertBigintsInObj(result),
          };
          ctx.status = 200;
        });
      }
    } else {
      // "JSON RPC mode" where a single endpoint is used and the method is given in the request body
      router.post('/', async (ctx: Koa.Context) => {
        const { params = [], jsonrpc, id, method } = ctx.request.body as any;
        // Ignore if not a function
        if (
          method === 'constructor' ||
          typeof proto[method] !== 'function' ||
          this.disallowedMethods.includes(method)
        ) {
          ctx.status = 400;
          ctx.body = { error: `Invalid method name: ${method}` };
        }
        const result = await this.proxy.call(method, params);

        ctx.body = {
          jsonrpc,
          id,
          result: convertBigintsInObj(result),
        };
        ctx.status = 200;
      });
    }

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
}
