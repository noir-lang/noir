import { type ClientProtocolCircuitVerifier, Tx } from '@aztec/circuit-types';
import { type CircuitVerificationStats } from '@aztec/circuit-types/stats';
import { type Proof, type VerificationKeyData } from '@aztec/circuits.js';
import { runInDirectory } from '@aztec/foundation/fs';
import { type DebugLogger, type LogFn, createDebugLogger } from '@aztec/foundation/log';
import {
  type ClientProtocolArtifact,
  type ProtocolArtifact,
  ProtocolCircuitArtifacts,
} from '@aztec/noir-protocol-circuits-types';

import * as fs from 'fs/promises';
import * as path from 'path';

import {
  BB_RESULT,
  PROOF_FILENAME,
  VK_FILENAME,
  generateContractForCircuit,
  generateKeyForNoirCircuit,
  verifyClientIvcProof,
  verifyProof,
} from '../bb/execute.js';
import { type BBConfig } from '../config.js';
import { mapProtocolArtifactNameToCircuitName } from '../stats.js';
import { extractVkData } from '../verification_key/verification_key_data.js';

export class BBCircuitVerifier implements ClientProtocolCircuitVerifier {
  private constructor(
    private config: BBConfig,
    private verificationKeys = new Map<ProtocolArtifact, Promise<VerificationKeyData>>(),
    private logger: DebugLogger,
  ) {}

  public static async new(
    config: BBConfig,
    initialCircuits: ProtocolArtifact[] = [],
    logger = createDebugLogger('aztec:bb-verifier'),
  ) {
    const keys = new Map<ProtocolArtifact, Promise<VerificationKeyData>>();
    for (const circuit of initialCircuits) {
      const vkData = await this.generateVerificationKey(
        circuit,
        config.bbBinaryPath,
        config.bbWorkingDirectory,
        logger.debug,
      );
      keys.set(circuit, Promise.resolve(vkData));
    }
    return new BBCircuitVerifier(config, keys, logger);
  }

  private static async generateVerificationKey(
    circuit: ProtocolArtifact,
    bbPath: string,
    workingDirectory: string,
    logFn: LogFn,
  ) {
    return await generateKeyForNoirCircuit(
      bbPath,
      workingDirectory,
      circuit,
      ProtocolCircuitArtifacts[circuit],
      'vk',
      logFn,
    ).then(result => {
      if (result.status === BB_RESULT.FAILURE) {
        throw new Error(`Failed to created verification key for ${circuit}, ${result.reason}`);
      }

      return extractVkData(result.vkPath!);
    });
  }

  public async getVerificationKeyData(circuit: ProtocolArtifact) {
    let promise = this.verificationKeys.get(circuit);
    if (!promise) {
      promise = BBCircuitVerifier.generateVerificationKey(
        circuit,
        this.config.bbBinaryPath,
        this.config.bbWorkingDirectory,
        this.logger.debug,
      );
    }
    this.verificationKeys.set(circuit, promise);
    const vk = await promise;
    return vk.clone();
  }

  public async verifyProofForCircuit(circuit: ProtocolArtifact, proof: Proof) {
    const operation = async (bbWorkingDirectory: string) => {
      const proofFileName = path.join(bbWorkingDirectory, PROOF_FILENAME);
      const verificationKeyPath = path.join(bbWorkingDirectory, VK_FILENAME);
      const verificationKey = await this.getVerificationKeyData(circuit);

      this.logger.debug(`${circuit} Verifying with key: ${verificationKey.keyAsFields.hash.toString()}`);

      await fs.writeFile(proofFileName, proof.buffer);
      await fs.writeFile(verificationKeyPath, verificationKey.keyAsBytes);

      const logFunction = (message: string) => {
        this.logger.debug(`${circuit} BB out - ${message}`);
      };

      const result = await verifyProof(this.config.bbBinaryPath, proofFileName, verificationKeyPath!, logFunction);

      if (result.status === BB_RESULT.FAILURE) {
        const errorMessage = `Failed to verify ${circuit} proof!`;
        throw new Error(errorMessage);
      }

      this.logger.debug(`${circuit} verification successful`, {
        circuitName: mapProtocolArtifactNameToCircuitName(circuit),
        duration: result.durationMs,
        eventName: 'circuit-verification',
        proofType: 'ultra-honk',
      } satisfies CircuitVerificationStats);
    };
    await runInDirectory(this.config.bbWorkingDirectory, operation);
  }

  public async generateSolidityContract(circuit: ProtocolArtifact, contractName: string) {
    const result = await generateContractForCircuit(
      this.config.bbBinaryPath,
      this.config.bbWorkingDirectory,
      circuit,
      ProtocolCircuitArtifacts[circuit],
      contractName,
      this.logger.debug,
    );

    if (result.status === BB_RESULT.FAILURE) {
      throw new Error(`Failed to create verifier contract for ${circuit}, ${result.reason}`);
    }

    return fs.readFile(result.contractPath!, 'utf-8');
  }

  public async verifyProof(tx: Tx): Promise<boolean> {
    try {
      // TODO(#7370) The verification keys should be supplied separately and based on the expectedCircuit
      // rather than read from the tx object itself. We also need the vks for the translator and ecc, which
      // are not being saved along the other vks yet. Reuse the 'verifyProofForCircuit' method above once
      // we have all the verification keys available.
      const expectedCircuit: ClientProtocolArtifact = tx.data.forPublic
        ? 'PrivateKernelTailToPublicArtifact'
        : 'PrivateKernelTailArtifact';
      const circuit = 'ClientIVC';

      // Block below is almost copy-pasted from verifyProofForCircuit
      const operation = async (bbWorkingDirectory: string) => {
        const logFunction = (message: string) => {
          this.logger.debug(`${circuit} BB out - ${message}`);
        };

        await tx.clientIvcProof.writeToOutputDirectory(bbWorkingDirectory);
        const result = await verifyClientIvcProof(this.config.bbBinaryPath, bbWorkingDirectory, logFunction);

        if (result.status === BB_RESULT.FAILURE) {
          const errorMessage = `Failed to verify ${circuit} proof!`;
          throw new Error(errorMessage);
        }

        this.logger.debug(`${circuit} verification successful`, {
          circuitName: mapProtocolArtifactNameToCircuitName(expectedCircuit),
          duration: result.durationMs,
          eventName: 'circuit-verification',
          proofType: 'client-ivc',
        } satisfies CircuitVerificationStats);
      };
      await runInDirectory(this.config.bbWorkingDirectory, operation);
      return true;
    } catch (err) {
      this.logger.warn(`Failed to verify ClientIVC proof for tx ${Tx.getHash(tx)}: ${String(err)}`);
      return false;
    }
  }
}
