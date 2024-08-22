import { type BotConfig, BotRunner, botConfigMappings, createBotRunnerRpcServer } from '@aztec/bot';
import { type AztecNode, type PXE } from '@aztec/circuit-types';
import { type ServerList } from '@aztec/foundation/json-rpc/server';
import { type LogFn } from '@aztec/foundation/log';

import { extractRelevantOptions } from '../util.js';

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
  // Start a PXE client that is used by the bot if required
  let pxe: PXE | undefined;
  if (options.pxe) {
    const { addPXE } = await import('./start_pxe.js');
    pxe = await addPXE(options, services, signalHandlers, userLog);
  }

  await addBot(options, services, signalHandlers, { pxe });
  return services;
}

export function addBot(
  options: any,
  services: ServerList,
  signalHandlers: (() => Promise<void>)[],
  deps: { pxe?: PXE; node?: AztecNode } = {},
) {
  const config = extractRelevantOptions<BotConfig>(options, botConfigMappings, 'bot');

  const botRunner = new BotRunner(config, deps);
  const botServer = createBotRunnerRpcServer(botRunner);
  if (!config.noStart) {
    void botRunner.start(); // Do not block since bot setup takes time
  }
  services.push({ bot: botServer });
  signalHandlers.push(botRunner.stop);
  return Promise.resolve();
}
