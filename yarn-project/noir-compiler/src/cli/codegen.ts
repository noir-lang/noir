import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import path from 'path';

import { generateContractArtifact } from '../contract-interface-gen/abi.js';
import { generateTypescriptContractInterface } from '../contract-interface-gen/contractTypescript.js';
import { generateNoirContractInterface } from '../contract-interface-gen/noir.js';

/**
 *
 */
export function generateCode(outputPath: string, fileOrDirPath: string, includeDebug = false, ts = false, nr = false) {
  const stats = statSync(fileOrDirPath);

  if (stats.isDirectory()) {
    const files = readdirSync(fileOrDirPath).filter(file => file.endsWith('.json') && !file.startsWith('debug_'));
    for (const file of files) {
      const fullPath = path.join(fileOrDirPath, file);
      generateFromNoirAbi(outputPath, fullPath, includeDebug, ts, nr);
    }
  } else if (stats.isFile()) {
    generateFromNoirAbi(outputPath, fileOrDirPath, includeDebug, ts, nr);
  }
}

/**
 *
 */
function generateFromNoirAbi(outputPath: string, noirAbiPath: string, includeDebug: boolean, ts: boolean, nr: boolean) {
  const contract = JSON.parse(readFileSync(noirAbiPath, 'utf8'));
  const noirDebugPath = includeDebug ? getDebugFilePath(noirAbiPath) : undefined;
  const debug = noirDebugPath ? JSON.parse(readFileSync(noirDebugPath, 'utf8')) : undefined;
  const aztecAbi = generateContractArtifact({ contract, debug });

  mkdirSync(outputPath, { recursive: true });

  if (nr) {
    const noirContract = generateNoirContractInterface(aztecAbi);
    writeFileSync(`${outputPath}/${aztecAbi.name}.nr`, noirContract);
    return;
  }

  writeFileSync(`${outputPath}/${aztecAbi.name}.json`, JSON.stringify(aztecAbi, undefined, 2));

  if (ts) {
    const tsWrapper = generateTypescriptContractInterface(aztecAbi, `./${aztecAbi.name}.json`);
    writeFileSync(`${outputPath}/${aztecAbi.name}.ts`, tsWrapper);
  }
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
