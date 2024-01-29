import { DebugLogger } from '@aztec/aztec.js';
import { BootstrapNode, P2PConfig, getP2PConfigEnvVars } from '@aztec/p2p';

import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

export const startP2PBootstrap = async (
  options: any,
  signalHandlers: (() => Promise<void>)[],
  debugLogger: DebugLogger,
) => {
  // Start a P2P bootstrap node.
  const envVars = getP2PConfigEnvVars();
  const cliOptions = parseModuleOptions(options.p2pBootstrap);
  const bootstrapNode = new BootstrapNode(debugLogger);
  const config = mergeEnvVarsAndCliOptions<P2PConfig>(envVars, cliOptions);
  await bootstrapNode.start(config);
  signalHandlers.push(bootstrapNode.stop);
};
