#!/usr/bin/env node
import { createConsoleLogger } from '@aztec/foundation/log';

import { Command } from 'commander';

import { compileNoir } from './cli/compileNoir.js';
import { generateNoirInterface } from './cli/noir-interface.js';
import { generateTypescriptInterface } from './cli/typescript.js';

const program = new Command();
const log = createConsoleLogger('aztec:compiler-cli');

const main = async () => {
  program.name('aztec-compile');
  compileNoir(program, 'compile', log);
  generateTypescriptInterface(program, 'typescript', log);
  generateNoirInterface(program, 'interface', log);
  await program.parseAsync(process.argv);
};

main().catch(err => {
  log(`Error running command`);
  log(err);
  process.exit(1);
});
