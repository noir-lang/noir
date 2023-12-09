import { NoirDependencyConfig, NoirGitDependencyConfig } from '../types/noir_package_config';

import { delimiter, join, sep } from 'path';
import { ZipEntry, unzip } from 'unzipit';

import { IFs } from 'memfs';
import { NoirPackage } from '../package';
import { NoirDependency, NoirDependencyResolver } from './dependency-resolver';

/**
 * Downloads dependencies from github
 */
export class GithubDependencyResolver implements NoirDependencyResolver {
  #fm: IFs;

  constructor(fm: IFs) {
    this.#fm = fm;
  }

  /**
   * Resolves a dependency from github. Returns null if URL is for a different website.
   * @param _pkg - The package to resolve the dependency for
   * @param dependency - The dependency configuration
   * @returns asd
   */
  async resolveDependency(_pkg: NoirPackage, dependency: NoirDependencyConfig): Promise<NoirDependency | null> {
    // TODO accept ssh urls?
    // TODO github authentication?
    if (!('git' in dependency) || !dependency.git.startsWith('https://github.com')) {
      return null;
    }

    const archivePath = await this.#fetchZipFromGithub(dependency);
    const libPath = await this.#extractZip(dependency, archivePath);
    return {
      version: dependency.tag,
      package: NoirPackage.open(libPath, this.#fm),
    };
  }

  async #fetchZipFromGithub(dependency: Pick<NoirGitDependencyConfig, 'git' | 'tag'>): Promise<string> {
    if (!dependency.git.startsWith('https://github.com')) {
      throw new Error('Only github dependencies are supported');
    }

    const url = resolveGithubCodeArchive(dependency, 'zip');
    const localArchivePath = join('archives', safeFilename(url.pathname));

    // TODO should check signature before accepting any file
    if (this.#fm.existsSync(localArchivePath)) {
      console.log('using cached archive', { url: url.href, path: localArchivePath });
      return localArchivePath;
    }

    const response = await fetch(url, {
      method: 'GET',
    });

    if (!response.ok || !response.body) {
      throw new Error(`Failed to fetch ${url}: ${response.statusText}`);
    }

    const chunks: Uint8Array[] = [];
    const reader = response.body.getReader();

    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }

      chunks.push(value);
    }
    const file = new Uint8Array(chunks.reduce((acc, chunk) => acc + chunk.length, 0));
    let offset = 0;
    for (const chunk of chunks) {
      file.set(chunk, offset);
      offset += chunk.length;
    }

    const tmpFile = localArchivePath + '.tmp';

    this.#fm.writeFileSync(tmpFile, file);

    this.#fm.mkdirSync(localArchivePath, { recursive: true });
    this.#fm.renameSync(tmpFile, localArchivePath);

    return localArchivePath;
  }

  async #extractZip(dependency: NoirGitDependencyConfig, archivePath: string): Promise<string> {
    const gitUrl = new URL(dependency.git);
    // extract the archive to this location
    const extractLocation = join('libs', safeFilename(gitUrl.pathname + '@' + (dependency.tag ?? 'HEAD')));

    // where we expect to find this package after extraction
    // it might already exist if the archive got unzipped previously
    const packagePath = join(extractLocation, dependency.directory ?? '');

    if (this.#fm.existsSync(packagePath)) {
      console.log(`Using existing package at ${packagePath}`);
      return packagePath;
    }

    const { entries } = await unzip(this.#fm.readFileSync(archivePath));

    // extract to a temporary directory, then move it to the final location
    // TODO empty the temp directory first
    const tmpExtractLocation = extractLocation + '.tmp';
    for (const entry of Object.values(entries)) {
      if ((entry as ZipEntry).isDirectory) {
        continue;
      }

      // remove the first path segment, because it'll be the archive name
      const name = stripSegments((entry as ZipEntry).name, 1);
      const path = join(tmpExtractLocation, name);

      const stream = (await (entry as ZipEntry).blob()).stream();

      const chunks: Uint8Array[] = [];
      const reader = stream.getReader();

      while (true) {
        const { done, value } = await reader.read();
        if (done) {
          break;
        }

        chunks.push(value);
      }
      const file = new Uint8Array(chunks.reduce((acc, chunk) => acc + chunk.length, 0));
      let offset = 0;
      for (const chunk of chunks) {
        file.set(chunk, offset);
        offset += chunk.length;
      }

      this.#fm.writeFileSync(path, file);
    }

    this.#fm.mkdirSync(extractLocation, { recursive: true });
    this.#fm.renameSync(tmpExtractLocation, extractLocation);

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

  // @ts-ignore
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
