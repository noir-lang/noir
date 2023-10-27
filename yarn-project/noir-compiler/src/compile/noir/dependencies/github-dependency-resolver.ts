import { createDebugOnlyLogger } from '@aztec/foundation/log';

import { delimiter, join, sep } from 'node:path';
import { unzip } from 'unzipit';

import { FileManager } from '../file-manager/file-manager.js';
import { NoirDependencyConfig, NoirGitDependencyConfig } from '../package-config.js';
import { NoirPackage } from '../package.js';
import { DependencyResolver } from './dependency-resolver.js';

/**
 * Downloads dependencies from github
 */
export class GithubDependencyResolver implements DependencyResolver {
  #fm: FileManager;
  #log = createDebugOnlyLogger('aztec:compile:github-dependency-resolver');

  constructor(fm: FileManager) {
    this.#fm = fm;
  }

  /**
   * Resolves a dependency from github. Returns null if URL is for a different website.
   * @param _pkg - The package to resolve the dependency for
   * @param dependency - The dependency configuration
   * @returns asd
   */
  async resolveDependency(_pkg: NoirPackage, dependency: NoirDependencyConfig): Promise<NoirPackage | null> {
    // TODO accept ssh urls?
    // TODO github authentication?
    if (!('git' in dependency) || !dependency.git.startsWith('https://github.com')) {
      return null;
    }

    const archivePath = await this.#fetchZipFromGithub(dependency);
    const libPath = await this.#extractZip(dependency, archivePath);
    return NoirPackage.open(libPath, this.#fm);
  }

  async #fetchZipFromGithub(dependency: Pick<NoirGitDependencyConfig, 'git' | 'tag'>): Promise<string> {
    if (!dependency.git.startsWith('https://github.com')) {
      throw new Error('Only github dependencies are supported');
    }

    const url = resolveGithubCodeArchive(dependency, 'zip');
    const localArchivePath = join('archives', safeFilename(url.pathname));

    // TODO should check signature before accepting any file
    if (this.#fm.hasFileSync(localArchivePath)) {
      this.#log('using cached archive', { url: url.href, path: localArchivePath });
      return localArchivePath;
    }

    const response = await fetch(url, {
      method: 'GET',
    });

    if (!response.ok || !response.body) {
      throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
    }

    const tmpFile = localArchivePath + '.tmp';
    await this.#fm.writeFile(tmpFile, response.body);
    this.#fm.moveFileSync(tmpFile, localArchivePath);

    return localArchivePath;
  }

  async #extractZip(dependency: NoirGitDependencyConfig, archivePath: string): Promise<string> {
    const gitUrl = new URL(dependency.git);
    const extractLocation = join('libs', safeFilename(gitUrl.pathname + '@' + (dependency.tag ?? 'HEAD')));
    const tmpExtractLocation = extractLocation + '.tmp';
    const packagePath = join(extractLocation, dependency.directory ?? '');

    if (this.#fm.hasFileSync(packagePath)) {
      this.#log(`Using existing package at ${packagePath}`);
      return packagePath;
    }

    const { entries } = await unzip(this.#fm.readFileSync(archivePath));

    for (const entry of Object.values(entries)) {
      if (entry.isDirectory) {
        continue;
      }

      const name = stripSegments(entry.name, 1);
      if (dependency.directory && !name.startsWith(dependency.directory)) {
        continue;
      }
      const path = join(tmpExtractLocation, name);
      await this.#fm.writeFile(path, (await entry.blob()).stream());
    }

    if (dependency.directory) {
      this.#fm.moveFileSync(join(tmpExtractLocation, dependency.directory), packagePath);
    } else {
      this.#fm.moveFileSync(tmpExtractLocation, packagePath);
    }

    return packagePath;
  }
}

/**
 * Strips the first n segments from a path
 */
function stripSegments(path: string, count: number): string {
  const segments = path.split(sep).filter(Boolean);
  return segments.slice(count).join(sep);
}

/**
 * Returns a safe filename for a value
 * @param val - The value to convert
 */
export function safeFilename(val: string): string {
  if (!val) {
    throw new Error('invalid value');
  }

  return val.replaceAll(sep, '_').replaceAll(delimiter, '_').replace(/^_+/, '');
}

/**
 * Resolves a dependency's archive URL.
 * @param dependency - The dependency configuration
 * @returns The URL to the library archive
 */
export function resolveGithubCodeArchive(dependency: NoirGitDependencyConfig, format: 'zip' | 'tar'): URL {
  const gitUrl = new URL(dependency.git);
  const [owner, repo] = gitUrl.pathname.slice(1).split('/');
  const ref = dependency.tag ?? 'HEAD';
  const extension = format === 'zip' ? 'zip' : 'tar.gz';

  if (!owner || !repo || gitUrl.hostname !== 'github.com') {
    throw new Error('Invalid Github repository URL');
  }

  return new URL(`https://github.com/${owner}/${repo}/archive/${ref}.${extension}`);
}
