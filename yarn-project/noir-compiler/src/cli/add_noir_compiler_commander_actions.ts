import { LogFn } from '@aztec/foundation/log';

import { Command } from 'commander';
import { dirname } from 'path';

/**
 * CLI options for configuring behavior
 */
interface Options {
  // eslint-disable-next-line jsdoc/require-jsdoc
  outdir: string;
  // eslint-disable-next-line jsdoc/require-jsdoc
  typescript: string | undefined;
  // eslint-disable-next-line jsdoc/require-jsdoc
  interface: string | undefined;
  // eslint-disable-next-line jsdoc/require-jsdoc
  compiler: string | undefined;
}

/**
 *
 */
export function addNoirCompilerCommanderActions(program: Command, log: LogFn = () => {}) {
  addCodegenCommanderAction(program, log);
}

/**
 *
 */
export function addCompileCommanderAction(program: Command, log: LogFn = () => {}) {
  program
    .command('compile')
    .argument('<project-path>', 'Path to the bin or Aztec.nr project to compile')
    .option('-o, --outdir <path>', 'Output folder for the binary artifacts, relative to the project path', 'target')
    .option('-ts, --typescript <path>', 'Optional output folder for generating typescript wrappers', undefined)
    .option('-i, --interface <path>', 'Optional output folder for generating an Aztec.nr contract interface', undefined)
    .option('-c --compiler <string>', 'Which compiler to use. Either nargo or wasm.', 'wasm')
    .description('Compiles the Noir Source in the target project')

    .action(async (projectPath: string, options: Options) => {
      const { compileNoir } = await import('./compile_noir.js');
      await compileNoir(projectPath, options, log);
    });
}

/**
 *
 */
export function addCodegenCommanderAction(program: Command, _: LogFn = () => {}) {
  program
    .command('codegen')
    .argument('<noir-abi-path>', 'Path to the Noir ABI or project dir.')
    .option('-o, --outdir <path>', 'Output folder for the generated code.')
    .option('-d, --debug', 'Include debug info.')
    .option('--ts', 'Generate TypeScript wrapper.')
    .option('--nr', 'Generate Noir interface.')
    .description('Validates and generates an Aztec Contract ABI from Noir ABI.')
    .action(async (noirAbiPath: string, { debug, outdir, ts, nr }) => {
      if (ts && nr) {
        throw new Error('--ts and --nr are mutually exclusive.');
      }
      const { generateCode } = await import('./codegen.js');
      generateCode(outdir || dirname(noirAbiPath), noirAbiPath, debug, ts, nr);
    });
}
