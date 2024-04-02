import { type DebugLogger } from '@aztec/aztec.js';
import { type LogFn } from '@aztec/foundation/log';
import { BootstrapNode, type P2PConfig, getP2PConfigEnvVars } from '@aztec/p2p';

import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

export const startP2PBootstrap = async (
  options: any,
  signalHandlers: (() => Promise<void>)[],
  userLog: LogFn,
  debugLogger: DebugLogger,
) => {
  // Start a P2P bootstrap node.
  const envVars = getP2PConfigEnvVars();
  const cliOptions = parseModuleOptions(options.p2pBootstrap);
  const bootstrapNode = new BootstrapNode(debugLogger);
  const config = mergeEnvVarsAndCliOptions<P2PConfig>(envVars, cliOptions);
  await bootstrapNode.start(config);
  userLog(`P2P bootstrap node started on ${config.tcpListenIp}:${config.tcpListenPort}`);
  signalHandlers.push(bootstrapNode.stop);
};
