import { deployInitialTestAccounts } from '@aztec/accounts/testing';
import { createAztecNodeRpcServer } from '@aztec/aztec-node';
import { type ServerList, createNamespacedJsonRpcServer, createStatusRouter } from '@aztec/foundation/json-rpc/server';
import { type DebugLogger, type LogFn } from '@aztec/foundation/log';
import { createPXERpcServer } from '@aztec/pxe';

import { type Command } from 'commander';
import http from 'http';

import { createSandbox } from '../sandbox.js';
import { github, splash } from '../splash.js';
import { cliTexts } from './texts.js';
import { createAccountLogs, installSignalHandlers } from './util.js';

const { AZTEC_PORT = '8080', API_PREFIX = '', TEST_ACCOUNTS = 'true', ENABLE_GAS = '' } = process.env;

/**
 * Returns commander program that defines the 'aztec' command line interface.
 * @param userLog - log function for logging user output.
 * @param debugLogger - logger for logging debug messages.
 */
export function injectAztecCommands(program: Command, userLog: LogFn, debugLogger: DebugLogger) {
  // Start Aztec modules with options
  program
    .command('start')
    .description(
      'Starts Aztec modules. Options for each module can be set as key-value pairs (e.g. "option1=value1,option2=value2") or as environment variables.',
    )
    .option('-sb, --sandbox', 'Starts Aztec Sandbox.')
    .option('-p, --port <port>', 'Port to run Aztec on.', AZTEC_PORT)
    .option('-n, --node [options]', cliTexts.node)
    .option('-px, --pxe [options]', cliTexts.pxe)
    .option('-a, --archiver [options]', cliTexts.archiver)
    .option('-s, --sequencer [options]', cliTexts.sequencer)
    .option('-r, --prover [options]', cliTexts.prover)
    .option('-p2p, --p2p-bootstrap [options]', cliTexts.p2pBootstrap)
    .option('-t, --txe [options]', cliTexts.txe)
    .action(async options => {
      // list of 'stop' functions to call when process ends
      const signalHandlers: Array<() => Promise<void>> = [];
      let services: ServerList = [];

      if (options.sandbox) {
        // If no CLI arguments were provided, run aztec full node for sandbox usage.
        userLog(`${splash}\n${github}\n\n`);
        userLog(`Setting up Aztec Sandbox, please stand by...`);
        const { aztecNodeConfig, node, pxe, stop } = await createSandbox({
          enableGas: ['true', '1'].includes(ENABLE_GAS),
        });

        // Deploy test accounts by default
        if (TEST_ACCOUNTS === 'true') {
          if (aztecNodeConfig.p2pEnabled) {
            userLog(`Not setting up test accounts as we are connecting to a network`);
          } else {
            userLog('Setting up test accounts...');
            const accounts = await deployInitialTestAccounts(pxe);
            const accLogs = await createAccountLogs(accounts, pxe);
            userLog(accLogs.join(''));
          }
        }

        // Start Node and PXE JSON-RPC server
        const nodeServer = createAztecNodeRpcServer(node);
        const pxeServer = createPXERpcServer(pxe);
        signalHandlers.push(stop);
        services = [{ node: nodeServer }, { pxe: pxeServer }];
      } else {
        // Start Aztec Node
        if (options.node) {
          const { startNode } = await import('./cmds/start_node.js');
          services = await startNode(options, signalHandlers, userLog);
        } else if (options.pxe) {
          const { startPXE } = await import('./cmds/start_pxe.js');
          services = await startPXE(options, signalHandlers, userLog);
        } else if (options.archiver) {
          const { startArchiver } = await import('./cmds/start_archiver.js');
          services = await startArchiver(options, signalHandlers);
        } else if (options.p2pBootstrap) {
          const { startP2PBootstrap } = await import('./cmds/start_p2p_bootstrap.js');
          await startP2PBootstrap(options, userLog, debugLogger);
        } else if (options.prover) {
          const { startProver } = await import('./cmds/start_prover.js');
          services = await startProver(options, signalHandlers, userLog);
        } else if (options.txe) {
          const { startTXE } = await import('./cmds/start_txe.js');
          startTXE(options, debugLogger);
        }
      }
      installSignalHandlers(debugLogger.info, signalHandlers);

      if (services.length) {
        const rpcServer = createNamespacedJsonRpcServer(services, debugLogger);

        const app = rpcServer.getApp(API_PREFIX);
        // add status route
        const statusRouter = createStatusRouter(API_PREFIX);
        app.use(statusRouter.routes()).use(statusRouter.allowedMethods());

        const httpServer = http.createServer(app.callback());
        httpServer.listen(options.port);
        userLog(`Aztec Server listening on port ${options.port}`);
      }
    });

  program.configureHelp({ sortSubcommands: true });

  program.addHelpText(
    'after',
    `
  
  Additional commands:

    test [options]: starts a dockerized TXE node via     
      $ aztec start --txe
    then runs 
      $ aztec-nargo test --silence-warnings --use-legacy --oracle-resolver=<TXE_ADDRESS> [options]
    `,
  );

  return program;
}
