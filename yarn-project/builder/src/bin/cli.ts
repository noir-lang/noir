#!/usr/bin/env node
import { createConsoleLogger } from '@aztec/foundation/log';

import { Command } from 'commander';

import { injectCommands as injectBuilderCommands } from '../index.js';

const log = createConsoleLogger('aztec:builder');

const main = async () => {
  const program = new Command('aztec-builder');

  injectBuilderCommands(program);
  await program.parseAsync(process.argv);
  // I force exit here because spawnSync in npm.ts just blocks the process from exiting. Spent a bit of time debugging
  // it without success and I think it doesn't make sense to invest more time in this.
  process.exit(0);
};

main().catch(err => {
  log(`Error running command`);
  log(err);
  process.exit(1);
});
