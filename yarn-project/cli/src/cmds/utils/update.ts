/* eslint-disable jsdoc/require-jsdoc */
import { type LogFn } from '@aztec/foundation/log';

import { relative, resolve } from 'path';
import { parse } from 'semver';

import { type DependencyChanges } from './update/common.js';
import { GITHUB_TAG_PREFIX } from './update/github.js';
import { updateAztecNr } from './update/noir.js';
import { getNewestVersion, updateAztecDeps, updateLockfile } from './update/npm.js';

const AZTECJS_PACKAGE = '@aztec/aztec.js';
const UPDATE_DOCS_URL = 'https://docs.aztec.network/developers/updating';

export async function updateProject(
  projectPath: string,
  contracts: string[],
  aztecVersion: string,
  log: LogFn,
): Promise<void> {
  const targetAztecVersion =
    aztecVersion === 'latest' ? await getNewestVersion(AZTECJS_PACKAGE, 'latest') : parse(aztecVersion);

  if (!targetAztecVersion) {
    throw new Error(`Invalid aztec version ${aztecVersion}`);
  }

  const projectDependencyChanges: DependencyChanges[] = [];
  try {
    const npmChanges = await updateAztecDeps(resolve(process.cwd(), projectPath), targetAztecVersion, log);
    if (npmChanges.dependencies.length > 0) {
      updateLockfile(projectPath, log);
    }

    projectDependencyChanges.push(npmChanges);
  } catch (err) {
    if (err instanceof Error && 'code' in err && err.code === 'ENOENT') {
      log(`No package.json found in ${projectPath}. Skipping npm update...`);
    } else {
      throw err;
    }
  }

  for (const contract of contracts) {
    try {
      projectDependencyChanges.push(
        await updateAztecNr(
          resolve(process.cwd(), projectPath, contract),
          `${GITHUB_TAG_PREFIX}-v${targetAztecVersion.version}`,
          log,
        ),
      );
    } catch (err) {
      if (err instanceof Error && 'code' in err && err.code === 'ENOENT') {
        log(`No Nargo.toml found in ${relative(process.cwd(), contract)}. Skipping...`);
      } else {
        throw err;
      }
    }
  }

  log(`To update Docker containers follow instructions at ${UPDATE_DOCS_URL}`);

  projectDependencyChanges.forEach(changes => {
    printChanges(changes, log);
  });
}

function printChanges(changes: DependencyChanges, log: LogFn): void {
  log(`\nIn ${relative(process.cwd(), changes.file)}:`);
  if (changes.dependencies.length === 0) {
    log('  No changes');
  } else {
    changes.dependencies.forEach(({ name, from, to }) => {
      log(`  Updated ${name} from ${from} to ${to}`);
    });
  }
}
