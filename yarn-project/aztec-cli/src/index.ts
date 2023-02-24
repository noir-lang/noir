#!/usr/bin/env node
import { Command } from 'commander';

const program = new Command();

async function main() {
  program
    .command('run')
    .argument('<cmd>', 'command')
    .action((cmd: string) => {
      console.log(`Running '${cmd}'...`);
    });

  await program.parseAsync(process.argv);
}

main().catch(err => {
  console.log(`Error thrown: ${err}`);
  process.exit(1);
});
