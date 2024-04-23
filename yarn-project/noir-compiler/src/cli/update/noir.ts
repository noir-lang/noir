import { type LogFn } from '@aztec/foundation/log';
import { parseNoirPackageConfig } from '@aztec/foundation/noir';

import TOML from '@iarna/toml';
import { readFile } from 'fs/promises';
import { join, relative, resolve } from 'path';

import { type DependencyChanges } from './common.js';
import { atomicUpdateFile, prettyPrintNargoToml } from './utils.js';

/**
 * Updates Aztec.nr dependencies
 * @param contractPath - Path to the contract to be updated
 * @param tag - The tag to update to
 * @param log - Logging function
 */
export async function updateAztecNr(contractPath: string, tag: string, log: LogFn): Promise<DependencyChanges> {
  const configFilepath = resolve(join(contractPath, 'Nargo.toml'));
  const packageConfig = parseNoirPackageConfig(TOML.parse(await readFile(configFilepath, 'utf-8')));
  const changes: DependencyChanges = {
    dependencies: [],
    file: configFilepath,
  };

  log(`Updating Aztec.nr libraries to ${tag} in ${relative(process.cwd(), changes.file)}`);
  for (const dep of Object.values(packageConfig.dependencies)) {
    if (!('git' in dep)) {
      continue;
    }

    // remove trailing slash
    const gitUrl = dep.git.toLowerCase().replace(/\/$/, '');
    if (gitUrl !== 'https://github.com/aztecprotocol/aztec-packages') {
      continue;
    }

    if (dep.tag !== tag) {
      // show the Aztec.nr package name rather than the lib name
      const dirParts = dep.directory?.split('/') ?? [];
      changes.dependencies.push({
        name: dirParts.slice(-2).join('/'),
        from: dep.tag,
        to: tag,
      });

      dep.tag = tag;
      dep.directory = dep.directory?.replace('yarn-project/', 'noir-projects/');
    }
  }

  if (changes.dependencies.length > 0) {
    const contents = prettyPrintNargoToml(packageConfig);
    await atomicUpdateFile(configFilepath, contents);
  }

  return changes;
}
