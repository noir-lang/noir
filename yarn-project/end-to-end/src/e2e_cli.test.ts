import { AztecNodeService } from '@aztec/aztec-node';
import { AztecAddress, AztecRPCServer } from '@aztec/aztec-rpc';
import { startHttpRpcServer } from '@aztec/aztec-sandbox/http';
import { createDebugLogger } from '@aztec/aztec.js';
import { getProgram } from '@aztec/cli';
import { DebugLogger } from '@aztec/foundation/log';
import { AztecRPC } from '@aztec/types';

import stringArgv from 'string-argv';
import { format } from 'util';

import { setup } from './fixtures/utils.js';

const HTTP_PORT = 9009;

// Spins up a new http server wrapping the set up rpc server, and tests cli commands against it
describe('cli', () => {
  let cli: ReturnType<typeof getProgram>;
  let http: ReturnType<typeof startHttpRpcServer>;
  let debug: DebugLogger;
  let aztecNode: AztecNodeService | undefined;
  let aztecRpcServer: AztecRPC;

  // All logs emitted by the cli will be collected here, and reset between tests
  const logs: string[] = [];

  beforeAll(async () => {
    debug = createDebugLogger('aztec:e2e_cli');
    const context = await setup(2);
    debug(`Environment set up`);
    const { deployL1ContractsValues } = context;
    ({ aztecNode, aztecRpcServer } = context);
    http = startHttpRpcServer(aztecRpcServer, deployL1ContractsValues, HTTP_PORT);
    debug(`HTTP RPC server started in port ${HTTP_PORT}`);
    const log = (...args: any[]) => {
      logs.push(format(...args));
      debug(...args);
    };
    cli = getProgram(log, debug);
  });

  afterAll(async () => {
    http.close();
    await aztecNode?.stop();
    await (aztecRpcServer as AztecRPCServer).stop();
  });

  beforeEach(() => {
    logs.splice(0);
  });

  // Run a command on the CLI
  const run = (cmd: string) =>
    cli.parseAsync(stringArgv(cmd, 'node', 'dest/bin/index.js').concat(['--rpc-url', `http://localhost:${HTTP_PORT}`]));

  // Returns first match across all logs collected so far
  const findInLogs = (regex: RegExp) => {
    for (const log of logs) {
      const match = regex.exec(log);
      if (match) return match;
    }
  };

  it('creates an account', async () => {
    const accountsBefore = await aztecRpcServer.getAccounts();
    await run(`create-account`);
    const newAddress = findInLogs(/Address:\s+(?<address>0x[a-fA-F0-9]+)/)?.groups?.address;
    expect(newAddress).toBeDefined();

    const accountsAfter = await aztecRpcServer.getAccounts();
    const expectedAccounts = [...accountsBefore.map(a => a.address), AztecAddress.fromString(newAddress!)];
    expect(accountsAfter.map(a => a.address)).toEqual(expectedAccounts);
  });
});
