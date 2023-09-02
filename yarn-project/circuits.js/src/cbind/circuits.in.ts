import { fileURLToPath } from '@aztec/foundation/url';

import { writeFileSync } from 'fs';
import { dirname } from 'path';

import { CircuitsWasm } from '../wasm/circuits_wasm.js';
import { getCbindSchema } from './cbind.js';
import { CbindCompiler } from './compiler.js';

/**
 * Generate TypeScript bindings for functions in CircuitsWasm.
 * This processes the schema for each export and compiles the TypeScript functions
 * to a 'circuits.gen.ts'.
 *
 * @returns -
 */
export async function main() {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  const wasm = await CircuitsWasm.get();
  const compiler = new CbindCompiler();
  for (const [key, value] of Object.entries(wasm.exports())) {
    if (typeof value === 'function' && key.endsWith('__schema')) {
      const cname = key.substring(0, key.length - '__schema'.length);
      compiler.processCbind(cname, getCbindSchema(wasm, cname));
    }
  }
  writeFileSync(__dirname + '/circuits.gen.ts', compiler.compile());
}

// eslint-disable-next-line no-console
main().catch(console.error);
