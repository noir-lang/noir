import { delimiter, join, sep } from 'path';
import { unzip } from 'unzipit';

import { FileManager } from '../file-manager/file-manager';
import { Package } from '../package';
import { Dependency, DependencyResolver } from './dependency-resolver';
import { DependencyConfig, GitDependencyConfig } from '../../types/noir_package_config';
import { LogData } from '../../utils';

/**
 * Downloads dependencies from github
 */
export class GithubDependencyResolver implements DependencyResolver {
  #fm: FileManager;
  #fetch: typeof fetch;
  #log;

  constructor(fm: FileManager, fetcher: typeof fetch) {
    this.#fm = fm;
    this.#fetch = fetcher;
    this.#log = (msg: string, _data?: LogData) => {
      console.log(msg);
    };
  }

  /**
   * Resolves a dependency from github. Returns null if URL is for a different website.
   * @param _pkg - The package to resolve the dependency for
   * @param dependency - The dependency configuration
   * @returns asd
   */
  async resolveDependency(_pkg: Package, dependency: DependencyConfig): Promise<Dependency | null> {
    // TODO accept ssh urls?
    // TODO github authentication?
    if (!('git' in dependency) || new URL(dependency.git).hostname != 'github.com') {
      return null;
    }

    const archivePath = await this.#fetchZipFromGithub(dependency);
    const libPath = await this.#extractZip(dependency, archivePath);
    return {
      version: dependency.tag,
      package: await Package.open(libPath, this.#fm),
    };
  }

  async #fetchZipFromGithub(dependency: Pick<GitDependencyConfig, 'git' | 'tag'>): Promise<string> {
    const git_host = new URL(dependency.git);
    if (git_host !== null && git_host.host != 'github.com') {
      throw new Error('Only github dependencies are supported');
    }

    const url = resolveGithubCodeArchive(dependency, 'zip');
    const localArchivePath = join('archives', safeFilename(url.pathname));

    // TODO should check signature before accepting any file
    if (this.#fm.hasFileSync(localArchivePath)) {
      this.#log('using cached archive', { url: url.href, path: localArchivePath });
      return localArchivePath;
    }

    const response = await this.#fetch(url, {
      method: 'GET',
    });

    if (!response.ok || !response.body) {
      throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
    }

    const tmpFile = localArchivePath + '.tmp';
    await this.#fm.writeFile(tmpFile, response.body);
    await this.#fm.moveFile(tmpFile, localArchivePath);

    return localArchivePath;
  }

  async #extractZip(dependency: GitDependencyConfig, archivePath: string): Promise<string> {
    const gitUrl = new URL(dependency.git);
    // extract the archive to this location
    const extractLocation = join('libs', safeFilename(gitUrl.pathname + '@' + (dependency.tag ?? 'HEAD')));

    // where we expect to find this package after extraction
    // it might already exist if the archive got unzipped previously
    const packagePath = join(extractLocation, dependency.directory ?? '');

    if (this.#fm.hasFileSync(packagePath)) {
      return packagePath;
    }

    const { entries } = await unzip(await this.#fm.readFile(archivePath));

    // extract to a temporary directory, then move it to the final location
    // TODO empty the temp directory first
    const tmpExtractLocation = extractLocation + '.tmp';
    for (const entry of Object.values(entries)) {
      if (entry.isDirectory) {
        continue;
      }

      // remove the first path segment, because it'll be the archive name
      const name = stripSegments(entry.name, 1);
      const path = join(tmpExtractLocation, name);
      await this.#fm.writeFile(path, (await entry.blob()).stream());
    }

    await this.#fm.moveFile(tmpExtractLocation, extractLocation);

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
export function resolveGithubCodeArchive(dependency: GitDependencyConfig, format: 'zip' | 'tar'): URL {
  const gitUrl = new URL(dependency.git);
  const [owner, repo] = gitUrl.pathname.slice(1).split('/');
  const ref = dependency.tag ?? 'HEAD';
  const extension = format === 'zip' ? 'zip' : 'tar.gz';

  if (!owner || !repo || gitUrl.hostname !== 'github.com') {
    throw new Error('Invalid Github repository URL');
  }

  // Validate ref to prevent path traversal attacks
  // First decode any URL encoding to catch encoded path traversal attempts
  const decodedRef = decodeURIComponent(ref);
  if (decodedRef.includes('..') || decodedRef.includes('/') || decodedRef.includes('\\')) {
    throw new Error('Invalid git reference. Git references cannot contain path traversal characters');
  }

  return new URL(`https://github.com/${owner}/${repo}/archive/${ref}.${extension}`);
}
