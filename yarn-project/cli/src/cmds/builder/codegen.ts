import { type GenerateCodeOptions, generateCode } from './contract-interface-gen/codegen.js';

/**
 * Generates Noir interface or Typescript interface for a folder or single file from a Noir compilation artifact.
 */
export function codegen(outputPath: string, fileOrDirPath: string, opts: GenerateCodeOptions = {}) {
  generateCode(outputPath, fileOrDirPath, opts);
}
