import { createDebugLogger } from '@aztec/foundation/log';
import { BootstrapNode, type P2PConfig } from '@aztec/p2p';

import Koa from 'koa';
import Router from 'koa-router';

const debugLogger = createDebugLogger('aztec:bootstrap_node');

const { HTTP_PORT } = process.env;

/**
 * The application entry point.
 */
async function main(config: P2PConfig, logger = debugLogger) {
  const bootstrapNode = new BootstrapNode(logger);
  await bootstrapNode.start(config);
  logger.info('DiscV5 Bootnode started');

  const httpApp = new Koa();
  const router = new Router();
  router.get('/health', (ctx: Koa.Context) => {
    ctx.status = 200;
  });

  httpApp.use(router.routes()).use(router.allowedMethods());
  httpApp.listen(HTTP_PORT, () => {
    logger.info(`HTTP server listening on port ${HTTP_PORT}`);
  });

  const stop = async () => {
    logger.debug('Stopping bootstrap node...');
    await bootstrapNode.stop();
    logger.info('Node stopped');
    process.exit(0);
  };
  process.on('SIGTERM', stop);
  process.on('SIGINT', stop);
}

export default main;
