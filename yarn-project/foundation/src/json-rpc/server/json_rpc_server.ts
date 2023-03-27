import http from 'http';
import Router from 'koa-router';
import cors from '@koa/cors';
import compress from 'koa-compress';
import { ClassConverterInput } from '../class_converter.js';
import Koa from 'koa';
import bodyParser from 'koa-bodyparser';
import { JsonProxy } from './json_proxy.js';
import { createDebugLogger } from '../../log/index.js';

const debug = createDebugLogger('json-rpc:json_rpc_server');

/**
 * JsonRpcServer.
 * Minimal, dev-friendly mechanism to create a server from an object.
 */
export class JsonRpcServer {
  proxy: JsonProxy;
  constructor(private handler: object, input: ClassConverterInput) {
    this.proxy = new JsonProxy(handler, input);
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
        console.log(err);
        ctx.status = 400;
        ctx.body = { error: err.message };
      }
    };
    const app = new Koa();
    app.on('error', error => {
      console.log(`KOA app-level error. ${JSON.stringify({ error })}`);
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
    for (const method of Object.getOwnPropertyNames(proto)) {
      // Ignore if not a function
      if (method === 'constructor' || typeof proto[method] !== 'function') {
        continue;
      }
      router.post(`/${method}`, async (ctx: Koa.Context) => {
        const { params = [], jsonrpc, id } = ctx.request.body as any;
        debug('JsonRpcServer:getRouter', method, '<-', params);
        const result = await this.proxy.call(method, params);
        ctx.body = { jsonrpc, id, result };
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
