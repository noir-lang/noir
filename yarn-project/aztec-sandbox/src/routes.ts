import Koa from 'koa';
import Router from 'koa-router';

/**
 * Creates a router for helper API endpoints of the Private eXecution Environment (PXE).
 * @param aztecNode - An instance of the aztec node.
 * @param config - The aztec node's configuration variables.
 */
export function createApiRouter() {
  const router = new Router({ prefix: '/api' });
  router.get('/status', (ctx: Koa.Context) => {
    // TODO: add `status` to Aztec node.
    ctx.status = 200;
  });

  return router;
}
