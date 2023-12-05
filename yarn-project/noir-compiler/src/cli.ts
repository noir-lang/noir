#!/usr/bin/env node
import { createConsoleLogger } from '@aztec/foundation/log';

import { Command } from 'commander';

import { addNoirCompilerCommanderActions } from './cli/add_noir_compiler_commander_actions.js';

const program = new Command();
const log = createConsoleLogger('aztec:compiler-cli');

const main = async () => {
  program.name('aztec-compile');
  addNoirCompilerCommanderActions(program, log);
  await program.parseAsync(process.argv);
};

main().catch(err => {
  log(`Error running command`);
  log(err);
  process.exit(1);
});
