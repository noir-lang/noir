#!/usr/bin/env node
import { injectCommands as injectBuilderCommands } from '@aztec/cli/builder';
import { createConsoleLogger } from '@aztec/foundation/log';

import { Command } from 'commander';

const log = createConsoleLogger('aztec:builder');

const main = async () => {
  const program = new Command('aztec-builder');

  injectBuilderCommands(program, log);
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
