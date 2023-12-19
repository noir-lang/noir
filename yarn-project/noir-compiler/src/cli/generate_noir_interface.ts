import { LogFn } from '@aztec/foundation/log';

import { mkdir, readFile, readdir, stat, writeFile } from 'fs/promises';
import path, { resolve } from 'path';

import { generateNoirContractInterface } from '../index.js';
import { isContractArtifact } from '../utils.js';

/**
 *
 */
export async function generateNoirInterface(
  projectPath: string,
  options: {
    // eslint-disable-next-line jsdoc/require-jsdoc
    outdir: string;
    // eslint-disable-next-line jsdoc/require-jsdoc
    artifacts: string;
  },
  log: LogFn,
) {
  const { outdir, artifacts } = options;
  if (typeof projectPath !== 'string') {
    throw new Error(`Missing project path argument`);
  }
  const currentDir = process.cwd();

  const artifactsDir = resolve(projectPath, artifacts);
  for (const artifactsDirItem of await readdir(artifactsDir)) {
    const artifactPath = resolve(artifactsDir, artifactsDirItem);
    if ((await stat(artifactPath)).isFile() && artifactPath.endsWith('.json')) {
      const contract = JSON.parse((await readFile(artifactPath)).toString());
      if (!isContractArtifact(contract)) {
        continue;
      }
      const interfacePath = resolve(projectPath, outdir, `${contract.name}_interface.nr`);
      log(`Writing ${contract.name} Noir external interface to ${path.relative(currentDir, interfacePath)}`);
      try {
        const noirInterface = generateNoirContractInterface(contract);
        await mkdir(path.dirname(interfacePath), { recursive: true });
        await writeFile(interfacePath, noirInterface);
      } catch (err) {
        log(`Error generating interface for ${artifactPath}: ${err}`);
      }
    }
  }
}
