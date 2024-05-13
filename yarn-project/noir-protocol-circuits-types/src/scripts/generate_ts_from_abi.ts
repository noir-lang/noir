import { createConsoleLogger } from '@aztec/foundation/log';

import { codegen } from '@noir-lang/noir_codegen';
import { type CompiledCircuit } from '@noir-lang/types';
import { pascalCase } from 'change-case';
import fs from 'fs/promises';

const log = createConsoleLogger('aztec:noir-contracts');

const circuits = [
  'parity_base',
  'parity_root',
  'private_kernel_init',
  'private_kernel_inner',
  'private_kernel_tail',
  'private_kernel_tail_to_public',
  'public_kernel_setup',
  'public_kernel_app_logic',
  'public_kernel_teardown',
  'public_kernel_tail',
  'rollup_base',
  'rollup_merge',
  'rollup_root',
];

const main = async () => {
  try {
    await fs.access('./src/types/');
  } catch (error) {
    await fs.mkdir('./src/types', { recursive: true });
  }
  const programs: [string, CompiledCircuit][] = [];
  // Collect all circuits
  for (const circuit of circuits) {
    const rawData = await fs.readFile(`./src/target/${circuit}.json`, 'utf-8');
    const abiObj: CompiledCircuit = JSON.parse(rawData);
    programs.push([pascalCase(circuit), abiObj]);
  }
  const code = codegen(
    programs,
    false, // Don't embed artifacts
    true, // Use fixed length arrays
  );
  await fs.writeFile('./src/types/index.ts', code);
};

try {
  await main();
} catch (err: unknown) {
  log(`Error generating types ${err}`);
  process.exit(1);
}
