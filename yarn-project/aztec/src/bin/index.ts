import { deployInitialTestAccounts } from '@aztec/accounts/testing';
import { createAztecNodeRpcServer } from '@aztec/aztec-node';
import { fileURLToPath } from '@aztec/aztec.js';
import { createNamespacedJsonRpcServer } from '@aztec/foundation/json-rpc/server';
import { createConsoleLogger, createDebugLogger } from '@aztec/foundation/log';
import { createPXERpcServer } from '@aztec/pxe';

import { readFileSync } from 'fs';
import http from 'http';
import { dirname, resolve } from 'path';

import { getProgram } from '../cli/index.js';
import { createAccountLogs, installSignalHandlers } from '../cli/util.js';
import { createSandbox } from '../sandbox.js';
import { github, splash } from '../splash.js';

const userLog = createConsoleLogger();
const debugLogger = createDebugLogger('aztec:cli');

const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../../package.json');
const cliVersion: string = JSON.parse(readFileSync(packageJsonPath).toString()).version;

const { TEST_ACCOUNTS = 'true', PORT = '8080' } = process.env;

/** CLI & full node main entrypoint */
async function main() {
  if (process.argv.length > 2) {
    // If CLI arguments were provided, run the CLI program.
    const cliProgram = getProgram(userLog, debugLogger);
    await cliProgram.parseAsync(process.argv);
  } else {
    // If no CLI arguments were provided, run aztec full node for sandbox usage.
    userLog(`${splash}\n${github}\n\n`);
    userLog(`Setting up Aztec Sandbox v${cliVersion}, please stand by...`);
    const { aztecNodeConfig, node, pxe, stop } = await createSandbox();
    installSignalHandlers(userLog, [stop]);

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
    const rpcServer = createNamespacedJsonRpcServer([{ node: nodeServer }, { pxe: pxeServer }], debugLogger);

    const app = rpcServer.getApp();
    const httpServer = http.createServer(app.callback());
    httpServer.listen(PORT);
    userLog(`Aztec Server listening on port ${PORT}`);
  }
}

main().catch(err => {
  debugLogger(`Error in command execution`);
  debugLogger(err);
  process.exit(1);
});
