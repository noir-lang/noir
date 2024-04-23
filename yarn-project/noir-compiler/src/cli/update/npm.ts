import { type LogFn } from '@aztec/foundation/log';

import { spawnSync } from 'child_process';
import { existsSync } from 'fs';
import { readFile } from 'fs/promises';
import { join, relative, resolve } from 'path';
import { type SemVer, parse } from 'semver';

import { type DependencyChanges } from './common.js';
import { atomicUpdateFile } from './utils.js';

const deprecatedNpmPackages = new Set<string>(['@aztec/cli', '@aztec/aztec-sandbox']);
const npmDeprecationMessage = `
The following packages have been deprecated and will no longer be updated on the npm registry:
${Array.from(deprecatedNpmPackages)
  .map(pkg => `  - ${pkg}`)
  .join('\n')}
Remove them from package.json
`;

/**
 * Looks up a package.json file and returns its contents
 * @param projectPath - Path to Nodejs project
 * @returns The parsed package.json
 */
export async function readPackageJson(projectPath: string): Promise<{
  /** dependencies */
  dependencies?: Record<string, string>;
  /** devDependencies */
  devDependencies?: Record<string, string>;
}> {
  const configFilepath = resolve(join(projectPath, 'package.json'));
  const pkg = JSON.parse(await readFile(configFilepath, 'utf-8'));

  return pkg;
}

/**
 * Queries the npm registry for the latest version of a package
 * @param packageName - The package to query
 * @param distTag - The distribution tag
 * @returns The latest version of the package on that distribution tag
 */
export async function getNewestVersion(packageName: string, distTag = 'latest'): Promise<SemVer> {
  const url = new URL(packageName, 'https://registry.npmjs.org/');
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url}`);
  }

  const body = await response.json();
  const latestVersion = parse(body['dist-tags'][distTag]);
  if (!latestVersion) {
    throw new Error(`Failed to get latest version from registry`);
  }

  return latestVersion;
}

/**
 * Updates a project's \@aztec/* dependencies to the specific version
 * @param projectPath - Path to Nodejs project
 * @param aztecVersion - The version to update to
 * @returns True if the project was updated
 */
export async function updateAztecDeps(
  projectPath: string,
  aztecVersion: SemVer,
  log: LogFn,
): Promise<DependencyChanges> {
  const pkg = await readPackageJson(projectPath);
  const changes: DependencyChanges = {
    file: resolve(join(projectPath, 'package.json')),
    dependencies: [],
  };

  log(`Updating @aztec packages to ${aztecVersion} in ${relative(process.cwd(), changes.file)}`);
  const version = aztecVersion.version;

  let detectedDeprecatedPackages = false;

  for (const depType of ['dependencies', 'devDependencies'] as const) {
    const dependencies = pkg[depType];
    if (!dependencies) {
      continue;
    }

    for (const name of Object.keys(dependencies)) {
      if (!name.startsWith('@aztec/')) {
        continue;
      }

      // different release schedule
      if (name === '@aztec/aztec-ui') {
        continue;
      }

      if (deprecatedNpmPackages.has(name)) {
        detectedDeprecatedPackages = true;
        continue;
      }

      if (dependencies[name] !== version) {
        changes.dependencies.push({
          name,
          from: dependencies[name],
          to: version,
        });

        dependencies[name] = version;
      }
    }
  }

  if (detectedDeprecatedPackages) {
    log(npmDeprecationMessage);
  }

  if (changes.dependencies.length > 0) {
    const contents = JSON.stringify(pkg, null, 2) + '\n';
    await atomicUpdateFile(resolve(join(projectPath, 'package.json')), contents);
  }

  return changes;
}

/**
 * Updates a project's yarn.lock or package-lock.json
 * @param projectPath - Path to Nodejs project
 */
export function updateLockfile(projectPath: string, log: LogFn): void {
  const isNpm = existsSync(resolve(join(projectPath, 'package-lock.json')));
  const isYarn = existsSync(resolve(join(projectPath, 'yarn.lock')));
  const isPnpm = existsSync(resolve(join(projectPath, 'pnpm-lock.yaml')));

  if (isPnpm) {
    spawnSync('pnpm', ['install'], {
      cwd: projectPath,
      stdio: 'inherit',
    });
  } else if (isYarn) {
    spawnSync('yarn', ['install'], {
      cwd: projectPath,
      stdio: 'inherit',
    });
  } else if (isNpm) {
    spawnSync('npm', ['install'], {
      cwd: projectPath,
      stdio: 'inherit',
    });
  } else {
    log(`No lockfile found in ${projectPath}. Skipping lockfile update...`);
  }
}
