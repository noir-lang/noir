import * as fs from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const NOIR_CONSTANTS_FILE = '../../../../noir-projects/noir-protocol-circuits/crates/types/src/constants.nr';
const TS_CONSTANTS_FILE = '../constants.gen.ts';
const SOLIDITY_CONSTANTS_FILE = '../../../../l1-contracts/src/core/libraries/ConstantsGen.sol';

/**
 * Parsed content.
 */
interface ParsedContent {
  /**
   * Constants.
   */
  constants: { [key: string]: string };
  /**
   * GeneratorIndexEnum.
   */
  generatorIndexEnum: { [key: string]: number };
}

/**
 * Processes a collection of constants and generates code to export them as TypeScript constants.
 *
 * @param constants - An object containing key-value pairs representing constants.
 * @returns A string containing code that exports the constants as TypeScript constants.
 */
function processConstantsTS(constants: { [key: string]: string }): string {
  const code: string[] = [];
  Object.entries(constants).forEach(([key, value]) => {
    code.push(`export const ${key} = ${+value > Number.MAX_SAFE_INTEGER ? value + 'n' : value};`);
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
function processEnumTS(enumName: string, enumValues: { [key: string]: number }): string {
  const code: string[] = [];

  code.push(`export enum ${enumName} {`);

  Object.entries(enumValues).forEach(([key, value]) => {
    code.push(`  ${key} = ${value},`);
  });

  code.push('}');

  return code.join('\n');
}

/**
 * Processes a collection of constants and generates code to export them as Solidity constants.
 *
 * @param constants - An object containing key-value pairs representing constants.
 * @param prefix - A prefix to add to the constant names.
 * @returns A string containing code that exports the constants as Noir constants.
 */
function processConstantsSolidity(constants: { [key: string]: string }, prefix = ''): string {
  const code: string[] = [];
  Object.entries(constants).forEach(([key, value]) => {
    code.push(`  uint256 internal constant ${prefix}${key} = ${value};`);
  });
  return code.join('\n');
}

/**
 * Generate the constants file in Typescript.
 */
function generateTypescriptConstants({ constants, generatorIndexEnum }: ParsedContent, targetPath: string) {
  const result = [
    '/* eslint-disable */\n// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants',
    processConstantsTS(constants),
    processEnumTS('GeneratorIndex', generatorIndexEnum),
  ].join('\n');

  fs.writeFileSync(targetPath, result);
}

/**
 * Generate the constants file in Solidity.
 */
function generateSolidityConstants({ constants }: ParsedContent, targetPath: string) {
  const resultSolidity: string = `// GENERATED FILE - DO NOT EDIT, RUN yarn remake-constants in circuits.js
// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 Aztec Labs.
pragma solidity >=0.8.18;

/**
 * @title Constants Library
 * @author Aztec Labs
 * @notice Library that contains constants used throughout the Aztec protocol
 */
library Constants {
  // Prime field modulus
  uint256 internal constant P =
    21888242871839275222246405745257275088548364400416034343698204186575808495617;
  uint256 internal constant MAX_FIELD_VALUE = P - 1;

${processConstantsSolidity(constants)}
}\n`;

  fs.writeFileSync(targetPath, resultSolidity);
}

/**
 * Parse the content of the constants file in Noir.
 */
function parseNoirFile(fileContent: string): ParsedContent {
  const constants: { [key: string]: string } = {};
  const generatorIndexEnum: { [key: string]: number } = {};

  fileContent.split('\n').forEach(l => {
    const line = l.trim();
    if (!line || line.match(/^\/\/|\/?\*/)) {
      return;
    }

    const [, name, _type, value] = line.match(/global\s+(\w+)(\s*:\s*\w+)?\s*=\s*(0x[a-fA-F0-9]+|[\d_]+);/) || [];
    if (!name || !value) {
      // eslint-disable-next-line no-console
      console.warn(`Unknown content: ${line}`);
      return;
    }

    const [, indexName] = name.match(/GENERATOR_INDEX__(\w+)/) || [];
    if (indexName) {
      generatorIndexEnum[indexName] = +value;
    } else {
      constants[name] = value;
    }
  });

  return { constants, generatorIndexEnum };
}

/**
 * Convert the Noir constants to TypeScript and Solidity.
 */
function main(): void {
  const __dirname = dirname(fileURLToPath(import.meta.url));

  const noirConstantsFile = join(__dirname, NOIR_CONSTANTS_FILE);
  const noirConstants = fs.readFileSync(noirConstantsFile, 'utf-8');
  const parsedContent = parseNoirFile(noirConstants);

  // Typescript
  const tsTargetPath = join(__dirname, TS_CONSTANTS_FILE);
  generateTypescriptConstants(parsedContent, tsTargetPath);

  // Solidity
  const solidityTargetPath = join(__dirname, SOLIDITY_CONSTANTS_FILE);
  fs.mkdirSync(dirname(solidityTargetPath), { recursive: true });
  generateSolidityConstants(parsedContent, solidityTargetPath);
}

main();
