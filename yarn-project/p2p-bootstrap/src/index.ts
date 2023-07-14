import { createDebugLogger } from '@aztec/foundation/log';
import { BootstrapNode, getP2PConfigEnvVars } from '@aztec/p2p';

import 'dotenv/config';

const logger = createDebugLogger('aztec:bootstrap_node');

/**
 * The application entry point.
 */
async function main() {
  const config = getP2PConfigEnvVars();
  const bootstrapNode = new BootstrapNode(logger);
  await bootstrapNode.start(config);
  logger('Node started');

  const stop = async () => {
    logger('Stopping bootstrap node...');
    await bootstrapNode.stop();
    logger('Node stopped');
    process.exit(0);
  };
  process.on('SIGTERM', stop);
  process.on('SIGINT', stop);
}

main().catch(err => {
  logger(err);
});
