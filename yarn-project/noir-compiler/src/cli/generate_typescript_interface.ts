import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import path from 'path';

import { generateContractArtifact } from '../contract-interface-gen/abi.js';
import { generateTypescriptContractInterface } from '../contract-interface-gen/contractTypescript.js';

/**
 *
 */
export function generateTypescriptInterface(outputPath: string, fileOrDirPath: string, includeDebug = false) {
  const stats = statSync(fileOrDirPath);

  if (stats.isDirectory()) {
    const files = readdirSync(fileOrDirPath).filter(file => file.endsWith('.json') && !file.startsWith('debug_'));
    for (const file of files) {
      const fullPath = path.join(fileOrDirPath, file);
      generateTypescriptInterfaceFromNoirAbi(outputPath, fullPath, includeDebug);
    }
  } else if (stats.isFile()) {
    generateTypescriptInterfaceFromNoirAbi(outputPath, fileOrDirPath, includeDebug);
  }
}

/**
 *
 */
function generateTypescriptInterfaceFromNoirAbi(outputPath: string, noirAbiPath: string, includeDebug: boolean) {
  const contract = JSON.parse(readFileSync(noirAbiPath, 'utf8'));
  const noirDebugPath = includeDebug ? getDebugFilePath(noirAbiPath) : undefined;
  const debug = noirDebugPath ? JSON.parse(readFileSync(noirDebugPath, 'utf8')) : undefined;
  const aztecAbi = generateContractArtifact({ contract, debug });
  const tsWrapper = generateTypescriptContractInterface(aztecAbi, `./${aztecAbi.name}.json`);
  mkdirSync(outputPath, { recursive: true });
  writeFileSync(`${outputPath}/${aztecAbi.name}.ts`, tsWrapper);
  writeFileSync(`${outputPath}/${aztecAbi.name}.json`, JSON.stringify(aztecAbi, undefined, 2));
}

/**
 *
 */
function getDebugFilePath(filePath: string) {
  const dirname = path.dirname(filePath);
  const basename = path.basename(filePath);
  const result = path.join(dirname, 'debug_' + basename);
  return existsSync(result) ? result : undefined;
}
