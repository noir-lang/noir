import { Fr, computeSecretHash, fileURLToPath } from '@aztec/aztec.js';
import { type LogFn, createConsoleLogger, createDebugLogger } from '@aztec/foundation/log';
import { AztecLmdbStore } from '@aztec/kv-store/lmdb';

import { Argument, Command } from 'commander';
import { readFileSync } from 'fs';
import { dirname, resolve } from 'path';

import { injectCommands } from '../cmds/index.js';
import { Aliases, WalletDB } from '../storage/wallet_db.js';
import { createAliasOption } from '../utils/options/index.js';

const userLog = createConsoleLogger();
const debugLogger = createDebugLogger('aztec:wallet');

const { WALLET_DATA_DIRECTORY } = process.env;

function injectInternalCommands(program: Command, log: LogFn, db: WalletDB) {
  program
    .command('alias')
    .description('Aliases information for easy reference.')
    .addArgument(new Argument('<type>', 'Type of alias to create').choices(Aliases))
    .argument('<key>', 'Key to alias.')
    .argument('<value>', 'Value to assign to the alias.')
    .action(async (type, key, value) => {
      value = db.tryRetrieveAlias(value) || value;
      await db.storeAlias(type, key, value, log);
    });

  program
    .command('add-secret')
    .description('Creates an aliased secret to use in other commands')
    .addOption(createAliasOption('Key to alias the secret with', false).makeOptionMandatory(true))
    .action(async (_options, command) => {
      const options = command.optsWithGlobals();
      const { alias } = options;
      const value = Fr.random();
      const hash = computeSecretHash(value);

      await db.storeAlias('secrets', alias, Buffer.from(value.toString()), log);
      await db.storeAlias('secrets', `${alias}:hash`, Buffer.from(hash.toString()), log);
    });

  return program;
}

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
    .hook('preSubcommand', command => {
      const dataDir = command.optsWithGlobals().dataDir;
      db.init(AztecLmdbStore.open(dataDir));
    });

  injectCommands(program, userLog, debugLogger, db);
  injectInternalCommands(program, userLog, db);
  await program.parseAsync(process.argv);
}

main().catch(err => {
  debugLogger.error(`Error in command execution`);
  debugLogger.error(err + '\n' + err.stack);
  process.exit(1);
});
