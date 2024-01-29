import { fileURLToPath } from '@aztec/aztec.js';
import { ServerList, createNamespacedJsonRpcServer } from '@aztec/foundation/json-rpc/server';
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';
import { readFileSync } from 'fs';
import http from 'http';
import { dirname, resolve } from 'path';

import { cliTexts } from './texts.js';
import { installSignalHandlers } from './util.js';

const { AZTEC_PORT = '8080' } = process.env;

/**
 * Returns commander program that defines the 'aztec' command line interface.
 * @param userLog - log function for logging user output.
 * @param debugLogger - logger for logging debug messages.
 */
export function getProgram(userLog: LogFn, debugLogger: DebugLogger): Command {
  const program = new Command();

  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../../package.json');
  const cliVersion: string = JSON.parse(readFileSync(packageJsonPath).toString()).version;

  program.name('aztec').description('Aztec command line interface').version(cliVersion);

  // Start Aztec modules with options
  program
    .command('start')
    .description(
      'Starts Aztec modules. Options for each module can be set as key-value pairs (e.g. "option1=value1,option2=value2") or as environment variables.',
    )
    .option('-p, --port <port>', 'Port to run Aztec on.', AZTEC_PORT)
    .option('-n, --node [options]', cliTexts.node)
    .option('-px, --pxe [options]', cliTexts.pxe)
    .option('-a, --archiver [options]', cliTexts.archiver)
    .option('-s, --sequencer [options]', cliTexts.sequencer)
    .option('-p2p, --p2p-bootstrap [options]', cliTexts.p2pBootstrap)
    .action(async options => {
      // list of 'stop' functions to call when process ends
      const signalHandlers: Array<() => Promise<void>> = [];
      let services: ServerList = [];

      // Start Aztec Node
      if (options.node) {
        const { startNode } = await import('./cmds/start_node.js');
        services = await startNode(options, signalHandlers, userLog);
      } else if (options.pxe) {
        const { startPXE } = await import('./cmds/start_pxe.js');
        services = await startPXE(options, signalHandlers, userLog);
      } else if (options.archiver) {
        const { startArchiver } = await import('./cmds/start_archiver.js');
        await startArchiver(options, signalHandlers);
      } else if (options.p2pBootstrap) {
        const { startP2PBootstrap } = await import('./cmds/start_p2p_bootstrap.js');
        await startP2PBootstrap(options, signalHandlers, debugLogger);
      }
      if (services.length) {
        const rpcServer = createNamespacedJsonRpcServer(services, debugLogger);

        const app = rpcServer.getApp();
        const httpServer = http.createServer(app.callback());
        httpServer.listen(options.port);
        userLog(`Aztec Server listening on port ${options.port}`);
      }
      installSignalHandlers(debugLogger, signalHandlers);
    });
  return program;
}
