import { Command } from 'commander';
import { compile, createFileManager } from '@noir-lang/noir_wasm';
import path from 'path';

async function generateProof({ programDir }: { programDir: string }) {
  const fm = createFileManager(path.resolve(programDir));

  await compile(fm);
}

// Prepare a minimal command line interface
const program = new Command();

program
  .command('compile')
  .option('-b, --program-dir <path>', 'program directory')
  .action((args) => generateProof(args));

program.parse(process.argv);
