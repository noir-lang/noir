import { LogFn, createDebugLogger } from '@aztec/foundation/log';

import { execSync } from 'child_process';
import { emptyDir } from 'fs-extra';
import { readFile, readdir, stat, unlink } from 'fs/promises';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

import { NoirCommit, NoirTag } from '../index.js';
import { NoirCompiledContract, NoirContractCompilationArtifacts, NoirDebugMetadata } from '../noir_artifact.js';

/** Compilation options */
export type CompileOpts = {
  /** Silence output from nargo compile. */
  quiet?: boolean;
  /** Path to the nargo binary. */
  nargoBin?: string;
  /** Logging function */
  log?: LogFn;
};

/**
 *
 */
function getCurrentDir() {
  if (typeof __dirname !== 'undefined') {
    return __dirname;
  } else {
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore
    return dirname(fileURLToPath(import.meta.url));
  }
}

/**
 * A class that compiles Aztec.nr contracts using nargo via the shell.
 */
export class NargoContractCompiler {
  private log: LogFn;
  constructor(private projectPath: string, private opts: CompileOpts = {}) {
    this.log = opts.log ?? createDebugLogger('aztec:noir-compiler');
  }

  /**
   * Compiles the contracts in projectPath and returns the Aztec.nr artifact.
   * @returns Aztec.nr artifact of the compiled contracts.
   */
  public async compile(): Promise<NoirContractCompilationArtifacts[]> {
    const stdio = this.opts.quiet ? 'ignore' : 'inherit';
    const nargoBin = this.opts.nargoBin ?? getCurrentDir() + '/../../../../noir/target/release/nargo';
    const version = execSync(`${nargoBin} --version`, { cwd: this.projectPath, stdio: 'pipe' }).toString();
    this.checkNargoBinVersion(version.replace('\n', ''));
    await emptyDir(this.getTargetFolder());
    execSync(`${nargoBin} compile`, { cwd: this.projectPath, stdio });
    return Promise.resolve(this.collectArtifacts());
  }

  private checkNargoBinVersion(version: string) {
    if (!version.includes(NoirCommit)) {
      this.log(
        `Warning: the nargo version installed locally does not match the expected one. This may cause issues when compiling or deploying contracts. Consider updating your nargo or aztec-cli installation. \n- Expected: ${NoirTag} (git version hash: ${NoirCommit})\n- Found: ${version}`,
      );
    } else if (!this.opts.quiet) {
      this.log(`Using ${version}`);
    }
  }

  private async collectArtifacts(): Promise<NoirContractCompilationArtifacts[]> {
    const contractArtifacts = new Map<string, NoirCompiledContract>();
    const debugArtifacts = new Map<string, NoirDebugMetadata>();

    for (const filename of await readdir(this.getTargetFolder())) {
      const file = join(this.getTargetFolder(), filename);
      if ((await stat(file)).isFile() && file.endsWith('.json')) {
        if (filename.startsWith('debug_')) {
          debugArtifacts.set(
            filename.replace('debug_', ''),
            JSON.parse((await readFile(file)).toString()) as NoirDebugMetadata,
          );
        } else {
          contractArtifacts.set(filename, JSON.parse((await readFile(file)).toString()) as NoirCompiledContract);
        }
        // Delete the file as it is not needed anymore and it can cause issues with prettier
        await unlink(file);
      }
    }

    return [...contractArtifacts.entries()].map(([filename, contractArtifact]) => {
      const debugArtifact = debugArtifacts.get(filename);
      return {
        contract: contractArtifact,
        debug: debugArtifact,
      };
    });
  }

  private getTargetFolder() {
    return join(this.projectPath, 'target');
  }
}
