import { execSync } from 'child_process';
import { readFileSync, readdirSync, statSync } from 'fs';
import { emptyDirSync } from 'fs-extra';
import path from 'path';

import { NoirCompiledContract } from '../noir_artifact.js';

/** Compilation options */
export type CompileOpts = {
  /** Silence output from nargo compile. */
  quiet?: boolean;
  /** Path to the nargo binary. */
  nargoBin?: string;
};

/**
 * A class that compiles noir contracts using nargo via the shell.
 */
export class NargoContractCompiler {
  constructor(private projectPath: string, private opts: CompileOpts = {}) {}

  /**
   * Compiles the contracts in projectPath and returns the Noir artifact.
   * @returns Noir artifact of the compiled contracts.
   */
  public compile(): Promise<NoirCompiledContract[]> {
    const stdio = this.opts.quiet ? 'ignore' : 'inherit';
    const nargoBin = this.opts.nargoBin ?? 'nargo';
    execSync(`${nargoBin} --version`, { cwd: this.projectPath, stdio });
    emptyDirSync(this.getTargetFolder());
    execSync(`${nargoBin} compile --contracts `, { cwd: this.projectPath, stdio });
    return Promise.resolve(this.collectArtifacts());
  }

  private collectArtifacts(): NoirCompiledContract[] {
    const artifacts = [];
    for (const filename of readdirSync(this.getTargetFolder())) {
      const file = path.join(this.getTargetFolder(), filename);
      if (statSync(file).isFile() && file.endsWith('.json')) {
        artifacts.push(JSON.parse(readFileSync(file).toString()) as NoirCompiledContract);
      }
    }
    return artifacts;
  }

  private getTargetFolder() {
    return path.join(this.projectPath, 'target');
  }
}
