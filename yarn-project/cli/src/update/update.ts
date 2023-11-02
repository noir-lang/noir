/* eslint-disable jsdoc/require-jsdoc */
import { DebugLogger, LogFn } from '@aztec/foundation/log';

import { relative, resolve } from 'path';
import { SemVer, coerce, gt, lt, parse } from 'semver';

import { createCompatibleClient } from '../client.js';
import { GITHUB_TAG_PREFIX } from '../github.js';
import { DependencyChanges } from './common.js';
import { updateAztecNr } from './noir.js';
import { getNewestVersion as getLatestVersion, readPackageJson, updateAztecDeps, updateLockfile } from './npm.js';

const SANDBOX_PACKAGE = '@aztec/aztec-sandbox';

export async function update(
  projectPath: string,
  contracts: string[],
  pxeUrl: string,
  sandboxVersion: string,
  log: LogFn,
  debugLog: DebugLogger,
): Promise<void> {
  const targetSandboxVersion =
    sandboxVersion === 'latest' ? await getLatestVersion(SANDBOX_PACKAGE, 'latest') : parse(sandboxVersion);

  if (!targetSandboxVersion) {
    throw new Error(`Invalid aztec version ${sandboxVersion}`);
  }

  let currentSandboxVersion = await getNpmSandboxVersion(projectPath, log);

  if (!currentSandboxVersion) {
    currentSandboxVersion = await getRemoteSandboxVersion(pxeUrl, log, debugLog);

    if (currentSandboxVersion && lt(currentSandboxVersion, targetSandboxVersion)) {
      log(`
Sandbox is older than version ${targetSandboxVersion}. If running via docker-compose, follow update instructions:
https://docs.aztec.network/dev_docs/cli/updating

Once the sandbox is updated, run the \`aztec-cli update\` command again`);
      return;
    }
  }

  if (!currentSandboxVersion) {
    throw new Error('Sandbox version could not be detected');
  }

  // sanity check
  if (gt(currentSandboxVersion, targetSandboxVersion)) {
    throw new Error('Local sandbox version is newer than latest version.');
  }

  const npmChanges = await updateAztecDeps(projectPath, targetSandboxVersion, log);
  if (npmChanges.dependencies.length > 0) {
    updateLockfile(projectPath, log);
  }

  const contractChanges: DependencyChanges[] = [];
  for (const contract of contracts) {
    try {
      contractChanges.push(
        await updateAztecNr(
          resolve(projectPath, contract),
          `${GITHUB_TAG_PREFIX}-v${targetSandboxVersion.version}`,
          log,
        ),
      );
    } catch (err) {
      if (err instanceof Error && 'code' in err && err.code === 'ENOENT') {
        log(`No Nargo.toml found in ${relative(process.cwd(), contract)}. Skipping...`);
        process.exit(1);
      }

      throw err;
    }
  }

  printChanges(npmChanges, log);

  contractChanges.forEach(changes => {
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

async function getNpmSandboxVersion(projectPath: string, log: LogFn): Promise<SemVer | null> {
  try {
    const pkg = await readPackageJson(projectPath);
    // use coerce instead of parse because it eliminates semver operators like ~ and ^
    return coerce(pkg.dependencies?.[SANDBOX_PACKAGE]);
  } catch (err) {
    if (err instanceof Error && 'code' in err && err.code === 'ENOENT') {
      log(`No package.json found in ${projectPath}`);
      process.exit(1);
    }

    throw err;
  }
}

async function getRemoteSandboxVersion(pxeUrl: string, log: LogFn, debugLog: DebugLogger): Promise<SemVer | null> {
  try {
    const client = await createCompatibleClient(pxeUrl, debugLog);
    const nodeInfo = await client.getNodeInfo();

    return parse(nodeInfo.sandboxVersion);
  } catch (err) {
    if (err instanceof Error && err.message === 'fetch failed') {
      log(`Could not connect to Sandbox running on ${pxeUrl}`);
      process.exit(1);
    }

    throw err;
  }
}
