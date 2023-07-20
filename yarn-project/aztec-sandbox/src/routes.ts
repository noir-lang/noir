import { DeployL1Contracts } from '@aztec/ethereum';

import Koa from 'koa';
import Router from 'koa-router';

/**
 * Creates a router for helper API endpoints of the Aztec RPC Server.
 * @param aztecNode - An instance of the aztec node.
 * @param config - The aztec node's configuration variables.
 */
export function createApiRouter(l1Contracts: DeployL1Contracts) {
  const router = new Router({ prefix: '/api' });
  router.get('/status', (ctx: Koa.Context) => {
    // TODO: add `status` to Aztec node.
    ctx.status = 200;
  });

  router.get('/l1-contract-addresses', (ctx: Koa.Context) => {
    ctx.body = {
      rollup: l1Contracts.rollupAddress.toString(),
      contractDeploymentEmitter: l1Contracts.contractDeploymentEmitterAddress.toString(),
      inbox: l1Contracts.inboxAddress.toString(),
      outbox: l1Contracts.outboxAddress.toString(),
      decoderHelper: l1Contracts.decoderHelperAddress?.toString(),
      registry: l1Contracts.registryAddress.toString(),
    };
    ctx.status = 200;
  });

  return router;
}
