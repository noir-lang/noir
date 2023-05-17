#!/usr/bin/env node
import { createLogger } from '@aztec/foundation/log';
import { Command } from 'commander';

const program = new Command();
const log = createLogger('aztec:aztec-cli');

/**
 * A placeholder for the Aztec-cli.
 */
async function main() {
  program
    .command('run')
    .argument('<cmd>', 'command')
    .action((cmd: string) => {
      log(`Running '${cmd}'...`);
    });

  await program.parseAsync(process.argv);
}

main().catch(err => {
  log(`Error thrown: ${err}`);
  process.exit(1);
});
