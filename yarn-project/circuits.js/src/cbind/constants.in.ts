import * as fs from 'fs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';
import { callCbind } from './cbind.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

/**
 * Convert the C++ constants to TypeScript.
 */
async function main(): Promise<void> {
  const wasm = await CircuitsWasm.get();
  const constants = callCbind(wasm, 'get_circuit_constants', []);
  const generatorIndexEnum = callCbind(wasm, 'get_circuit_generator_index', []);
  const privateStateNoteGeneratorIndexEnum = callCbind(wasm, 'get_circuit_private_state_note_generator_index', []);
  const privateStateTypeEnum = callCbind(wasm, 'get_circuit_private_state_type', []);

  const result: string =
    '/* eslint-disable */\n// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants\n' +
    processConstants(constants) +
    processEnum('GeneratorIndex', generatorIndexEnum) +
    processEnum('PrivateStateNoteGeneratorIndex', privateStateNoteGeneratorIndexEnum) +
    processEnum('PrivateStateType', privateStateTypeEnum);

  const __dirname = dirname(fileURLToPath(import.meta.url));
  fs.writeFileSync(__dirname + '/constants.gen.ts', result);
}

/**
 * Processes a collection of constants and generates code to export them as TypeScript constants.
 *
 * @param constants - An object containing key-value pairs representing constants.
 * @returns A string containing code that exports the constants as TypeScript constants.
 */
function processConstants(constants: { [key: string]: number }): string {
  const code: string[] = [];
  Object.entries(constants).forEach(([key, value]) => {
    code.push(`export const ${key} = ${value};`);
  });
  return code.join('\n');
}

/**
 * Processes an enum and generates code to export it as a TypeScript enum.
 *
 * @param enumName - The name of the enum.
 * @param enumValues - An object containing key-value pairs representing enum values.
 * @returns A string containing code that exports the enum as a TypeScript enum.
 */
function processEnum(enumName: string, enumValues: { [key: string]: number }): string {
  const code: string[] = [];

  code.push(`export enum ${enumName} {`);

  Object.entries(enumValues).forEach(([key, value]) => {
    code.push(`  ${key} = ${value},`);
  });

  code.push('}');

  return code.join('\n');
}

// eslint-disable-next-line no-console
main().catch(console.error);
