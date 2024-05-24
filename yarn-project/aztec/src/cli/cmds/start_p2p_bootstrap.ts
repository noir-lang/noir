import { type DebugLogger } from '@aztec/aztec.js';
import { type LogFn } from '@aztec/foundation/log';
import { type P2PConfig, getP2PConfigEnvVars } from '@aztec/p2p';
import runBootstrapNode from '@aztec/p2p-bootstrap';

import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

export const startP2PBootstrap = async (options: any, userLog: LogFn, debugLogger: DebugLogger) => {
  // Start a P2P bootstrap node.
  const envVars = getP2PConfigEnvVars();
  const cliOptions = parseModuleOptions(options.p2pBootstrap);
  const config = mergeEnvVarsAndCliOptions<P2PConfig>(envVars, cliOptions);
  await runBootstrapNode(config, debugLogger);
  userLog(`P2P bootstrap node started on ${config.udpListenIp}:${config.udpListenPort}`);
};
