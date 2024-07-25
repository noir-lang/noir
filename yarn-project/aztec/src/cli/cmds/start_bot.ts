import { type BotConfig, BotRunner, createBotRunnerRpcServer, getBotConfigFromEnv } from '@aztec/bot';
import { type PXE } from '@aztec/circuit-types';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { type LogFn } from '@aztec/foundation/log';

import { mergeEnvVarsAndCliOptions, parseModuleOptions } from '../util.js';

export async function startBot(
  options: any,
  signalHandlers: (() => Promise<void>)[],
  userLog: LogFn,
): Promise<ServerList> {
  // Services that will be started in a single multi-rpc server
  const services: ServerList = [];

  const { proverNode, archiver, sequencer, p2pBootstrap, txe, prover } = options;
  if (proverNode || archiver || sequencer || p2pBootstrap || txe || prover) {
    userLog(
      `Starting a bot with --prover-node, --prover, --archiver, --sequencer, --p2p-bootstrap, or --txe is not supported.`,
    );
    process.exit(1);
  }

  await addBot(options, services, signalHandlers);
  return services;
}

export function addBot(
  options: any,
  services: ServerList,
  signalHandlers: (() => Promise<void>)[],
  deps: { pxe?: PXE } = {},
) {
  const envVars = getBotConfigFromEnv();
  const cliOptions = parseModuleOptions(options.bot);
  const config = mergeEnvVarsAndCliOptions<BotConfig>(envVars, cliOptions);

  const botRunner = new BotRunner(config, { pxe: deps.pxe });
  const botServer = createBotRunnerRpcServer(botRunner);
  if (!config.noStart) {
    void botRunner.start(); // Do not block since bot setup takes time
  }
  services.push({ bot: botServer });
  signalHandlers.push(botRunner.stop);
  return Promise.resolve();
}
