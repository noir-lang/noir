#!/usr/bin/env node
import { createConsoleLogger } from '@aztec/foundation/log';

import { Command } from 'commander';

import { compileContract } from './cli/contract.js';

const program = new Command();
const log = createConsoleLogger('aztec:compiler-cli');

const main = async () => {
  compileContract(program.name('aztec-compile'), 'contract', log);

  await program.parseAsync(process.argv);
};

main().catch(err => {
  log(`Error running command`);
  log(err);
  process.exit(1);
});
