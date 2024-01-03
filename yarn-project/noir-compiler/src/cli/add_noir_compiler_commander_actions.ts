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
  // addCompileCommanderAction(program, log);
  addGenerateTypescriptCommanderAction(program, log);
  addGenerateNoirInterfaceCommanderAction(program, log);
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
export function addGenerateTypescriptCommanderAction(program: Command, _: LogFn = () => {}) {
  program
    .command('generate-typescript')
    .argument('<noir-abi-path>', 'Path to the Noir ABI or project dir.')
    .option('-o, --outdir <path>', 'Output folder for the generated typescript.')
    .option('-d, --debug', 'Include debug info.')
    .description('Generates TypeScript interface from the given abi.')
    .action(async (noirAbiPath: string, { debug, outdir }) => {
      const { generateTypescriptInterface } = await import('./generate_typescript_interface.js');
      generateTypescriptInterface(outdir || dirname(noirAbiPath), noirAbiPath, debug);
    });
}

/**
 *
 */
export function addGenerateNoirInterfaceCommanderAction(program: Command, _: LogFn = () => {}) {
  return program
    .command('generate-noir-interface')
    .argument('<noir-abi-path>', 'Path to the Noir ABI or project dir.')
    .option('-o, --outdir <path>', 'Output folder for the generated noir.')
    .description('Generates Noir interfaces from the artifacts in the given project')
    .action(async (noirAbiPath: string, { outdir }) => {
      const { generateNoirInterface } = await import('./generate_noir_interface.js');
      generateNoirInterface(outdir || dirname(noirAbiPath), noirAbiPath);
    });
}
