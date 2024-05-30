#!/usr/bin/env -S node --no-warnings
import { createConsoleLogger, createDebugLogger } from '@aztec/foundation/log';

import 'source-map-support/register.js';

import { getProgram } from '../index.js';

const debugLogger = createDebugLogger('aztec:cli-client');
const log = createConsoleLogger();

/** CLI main entrypoint */
async function main() {
  process.once('SIGINT', () => process.exit(0));
  process.once('SIGTERM', () => process.exit(0));

  const program = getProgram(log, debugLogger);
  await program.parseAsync(process.argv);
}

main().catch(err => {
  log(`Error in command execution`);
  log(err);
  process.exit(1);
});
