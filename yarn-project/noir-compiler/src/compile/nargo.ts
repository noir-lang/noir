import { execSync } from 'child_process';
import { readFileSync, readdirSync, statSync } from 'fs';
import { emptyDirSync } from 'fs-extra';
import path from 'path';

import { NoirCompilationArtifacts, NoirCompiledContract, NoirDebugMetadata } from '../noir_artifact.js';

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
  public compile(): Promise<NoirCompilationArtifacts[]> {
    const stdio = this.opts.quiet ? 'ignore' : 'inherit';
    const nargoBin = this.opts.nargoBin ?? 'nargo';
    execSync(`${nargoBin} --version`, { cwd: this.projectPath, stdio });
    emptyDirSync(this.getTargetFolder());
    execSync(`${nargoBin} compile --output-debug `, { cwd: this.projectPath, stdio });
    return Promise.resolve(this.collectArtifacts());
  }

  private collectArtifacts(): NoirCompilationArtifacts[] {
    const contractArtifacts = new Map<string, NoirCompiledContract>();
    const debugArtifacts = new Map<string, NoirDebugMetadata>();

    for (const filename of readdirSync(this.getTargetFolder())) {
      const file = path.join(this.getTargetFolder(), filename);
      if (statSync(file).isFile() && file.endsWith('.json')) {
        if (filename.startsWith('debug_')) {
          debugArtifacts.set(
            filename.replace('debug_', ''),
            JSON.parse(readFileSync(file).toString()) as NoirDebugMetadata,
          );
        } else {
          contractArtifacts.set(filename, JSON.parse(readFileSync(file).toString()) as NoirCompiledContract);
        }
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
    return path.join(this.projectPath, 'target');
  }
}
