import { type PublicKey } from '@aztec/circuit-types';
import { FunctionData } from '@aztec/circuits.js';
import {
  type ContractArtifact,
  type FunctionArtifact,
  encodeArguments,
  getFunctionArtifact,
} from '@aztec/foundation/abi';

import { type AuthWitnessProvider } from '../account/interface.js';
import { type Wallet } from '../account/wallet.js';
import { type ExecutionRequestInit } from '../api/entrypoint.js';
import { Contract } from '../contract/contract.js';
import { DeployMethod, type DeployOptions } from '../contract/deploy_method.js';
import { EntrypointPayload } from '../entrypoint/payload.js';

/**
 * Contract interaction for deploying an account contract. Handles fee preparation and contract initialization.
 */
export class DeployAccountMethod extends DeployMethod {
  #authWitnessProvider: AuthWitnessProvider;
  #feePaymentArtifact: FunctionArtifact | undefined;

  constructor(
    authWitnessProvider: AuthWitnessProvider,
    publicKey: PublicKey,
    wallet: Wallet,
    artifact: ContractArtifact,
    args: any[] = [],
    constructorNameOrArtifact?: string | FunctionArtifact,
    feePaymentNameOrArtifact?: string | FunctionArtifact,
  ) {
    super(
      publicKey,
      wallet,
      artifact,
      (address, wallet) => Contract.at(address, artifact, wallet),
      args,
      constructorNameOrArtifact,
    );

    this.#authWitnessProvider = authWitnessProvider;
    this.#feePaymentArtifact =
      typeof feePaymentNameOrArtifact === 'string'
        ? getFunctionArtifact(artifact, feePaymentNameOrArtifact)
        : feePaymentNameOrArtifact;
  }

  protected async getInitializeFunctionCalls(options: DeployOptions): Promise<ExecutionRequestInit> {
    const exec = await super.getInitializeFunctionCalls(options);

    if (options.fee && this.#feePaymentArtifact) {
      const { address } = this.getInstance();
      const feePayload = await EntrypointPayload.fromFeeOptions(options?.fee);

      exec.calls.push({
        to: address,
        args: encodeArguments(this.#feePaymentArtifact, [feePayload]),
        functionData: FunctionData.fromAbi(this.#feePaymentArtifact),
      });

      exec.authWitnesses ??= [];
      exec.packedArguments ??= [];

      exec.authWitnesses.push(await this.#authWitnessProvider.createAuthWit(feePayload.hash()));
      exec.packedArguments.push(...feePayload.packedArguments);
    }

    return exec;
  }
}
