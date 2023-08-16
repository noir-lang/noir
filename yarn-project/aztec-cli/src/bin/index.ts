#!/usr/bin/env -S node --no-warnings
import { createDebugLogger } from '@aztec/aztec.js';
import { createConsoleLogger } from '@aztec/foundation/log';

import { getProgram } from '../index.js';

const debugLogger = createDebugLogger('aztec:cli');
const log = createConsoleLogger();

/** CLI main entrypoint */
async function main() {
  const program = getProgram(log, debugLogger);
  await program.parseAsync(process.argv);
}

main().catch(err => {
  log(`Error in command execution`);
  log(err);
  process.exit(1);
});
