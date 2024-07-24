import { loadContractArtifact } from '@aztec/aztec.js';
import { Fr } from '@aztec/foundation/fields';
import { JsonRpcServer } from '@aztec/foundation/json-rpc/server';
import { type Logger } from '@aztec/foundation/log';

import { readFile, readdir } from 'fs/promises';
import { join } from 'path';

import { TXEService } from './txe_service/txe_service.js';
import { type ForeignCallArray, type ForeignCallResult, fromArray, toForeignCallResult } from './util/encoding.js';

const TXESessions = new Map<number, TXEService>();

type MethodNames<T> = {
  [K in keyof T]: T[K] extends (...args: any[]) => any ? K : never;
}[keyof T];

type TXEForeignCallInput = {
  session_id: number;
  function: MethodNames<TXEService> | 'reset';
  root_path: string;
  package_name: string;
  inputs: any[];
};

class TXEDispatcher {
  constructor(private logger: Logger) {}

  async #processDeployInputs({ inputs, root_path: rootPath, package_name: packageName }: TXEForeignCallInput) {
    const pathStr = fromArray(inputs[0] as ForeignCallArray)
      .map(char => String.fromCharCode(char.toNumber()))
      .join('');
    const contractName = fromArray(inputs[1] as ForeignCallArray)
      .map(char => String.fromCharCode(char.toNumber()))
      .join('');
    let artifactPath = '';
    // We're deploying the contract under test
    // env.deploy_self("contractName")
    if (!pathStr) {
      artifactPath = join(rootPath, './target', `${packageName}-${contractName}.json`);
    } else {
      // We're deploying a contract that belongs in a workspace
      // env.deploy("../path/to/workspace/root@packageName", "contractName")
      if (pathStr.includes('@')) {
        const [workspace, pkg] = pathStr.split('@');
        const targetPath = join(rootPath, workspace, './target');
        this.logger.debug(`Looking for compiled artifact in workspace ${targetPath}`);
        artifactPath = join(targetPath, `${pkg}-${contractName}.json`);
      } else {
        // We're deploying a standalone contract
        // env.deploy("../path/to/contract/root", "contractName")
        const targetPath = join(rootPath, pathStr, './target');
        this.logger.debug(`Looking for compiled artifact in ${targetPath}`);
        [artifactPath] = (await readdir(targetPath)).filter(file => file.endsWith(`-${contractName}.json`));
      }
    }
    this.logger.debug(`Loading compiled artifact ${artifactPath}`);
    const artifact = loadContractArtifact(JSON.parse(await readFile(artifactPath, 'utf-8')));
    inputs.splice(0, 2, artifact);
  }

  // eslint-disable-next-line camelcase
  async resolve_foreign_call(callData: TXEForeignCallInput): Promise<ForeignCallResult> {
    const { session_id: sessionId, function: functionName, inputs } = callData;
    this.logger.debug(`Calling ${functionName} on session ${sessionId}`);

    if (!TXESessions.has(sessionId) && functionName != 'reset') {
      this.logger.info(`Creating new session ${sessionId}`);
      TXESessions.set(sessionId, await TXEService.init(this.logger));
    }

    switch (functionName) {
      case 'reset': {
        TXESessions.delete(sessionId) &&
          this.logger.info(`Called reset on session ${sessionId}, yeeting it out of existence`);
        return toForeignCallResult([]);
      }
      case 'deploy': {
        // Modify inputs and fall through
        await this.#processDeployInputs(callData);
      }
      // eslint-disable-next-line no-fallthrough
      default: {
        const txeService = TXESessions.get(sessionId);
        const response = await (txeService as any)[functionName](...inputs);
        return response;
      }
    }
  }
}

/**
 * Creates an RPC server that forwards calls to the TXE.
 * @param logger - Logger to output to
 * @returns A TXE RPC server.
 */
export function createTXERpcServer(logger: Logger) {
  return new JsonRpcServer(new TXEDispatcher(logger), { Fr }, {}, ['init']);
}
