import { fileURLToPath } from '@aztec/aztec.js';
import { createConsoleLogger, createDebugLogger } from '@aztec/foundation/log';
import { AztecLmdbStore } from '@aztec/kv-store/lmdb';

import { Command } from 'commander';
import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';

import { injectCommands } from '../cmds/index.js';
import { WalletDB } from '../storage/wallet_db.js';

const userLog = createConsoleLogger();
const debugLogger = createDebugLogger('aztec:wallet');

const { WALLET_DATA_DIRECTORY } = process.env;

/** CLI wallet main entrypoint */
async function main() {
  const packageJsonPath = resolve(dirname(fileURLToPath(import.meta.url)), '../../package.json');
  const walletVersion: string = JSON.parse(readFileSync(packageJsonPath).toString()).version;

  const db = WalletDB.getInstance();

  const program = new Command('wallet');
  program
    .description('Aztec wallet')
    .version(walletVersion)
    .option('-d, --data-dir <string>', 'Storage directory for wallet data', WALLET_DATA_DIRECTORY)
    .hook('preAction', command => {
      const dataDir = command.optsWithGlobals().dataDir;
      db.init(AztecLmdbStore.open(dataDir));
    });

  injectCommands(program, userLog, debugLogger, db);
  await program.parseAsync(process.argv);
}

main().catch(err => {
  debugLogger.error(`Error in command execution`);
  debugLogger.error(err + '\n' + err.stack);
  process.exit(1);
});
