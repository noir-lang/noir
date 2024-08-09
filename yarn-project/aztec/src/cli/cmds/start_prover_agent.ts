import { BBNativeRollupProver, TestCircuitProver } from '@aztec/bb-prover';
import { type ServerCircuitProver } from '@aztec/circuit-types';
import { type ProverClientConfig, proverClientConfigMappings } from '@aztec/prover-client';
import { ProverAgent, createProvingJobSourceClient } from '@aztec/prover-client/prover-agent';
import {
  type TelemetryClientConfig,
  createAndStartTelemetryClient,
  telemetryClientConfigMappings,
} from '@aztec/telemetry-client/start';

import { type ServiceStarter, extractRelevantOptions } from '../util.js';

export const startProverAgent: ServiceStarter = async (options, signalHandlers, logger) => {
  const proverConfig = extractRelevantOptions<ProverClientConfig>(options, proverClientConfigMappings);

  if (!proverConfig.nodeUrl) {
    throw new Error('Starting prover without an orchestrator is not supported');
  }

  logger(`Connecting to prover at ${proverConfig.nodeUrl}`);
  const source = createProvingJobSourceClient(proverConfig.nodeUrl, 'provingJobSource');

  const telemetryConfig = extractRelevantOptions<TelemetryClientConfig>(options, telemetryClientConfigMappings);
  const telemetry = createAndStartTelemetryClient(telemetryConfig);

  let circuitProver: ServerCircuitProver;
  if (proverConfig.realProofs) {
    if (!proverConfig.acvmBinaryPath || !proverConfig.bbBinaryPath) {
      throw new Error('Cannot start prover without simulation or native prover options');
    }
    circuitProver = await BBNativeRollupProver.new(proverConfig, telemetry);
  } else {
    circuitProver = new TestCircuitProver(telemetry, undefined, proverConfig);
  }

  const agent = new ProverAgent(
    circuitProver,
    proverConfig.proverAgentConcurrency,
    proverConfig.proverAgentPollInterval,
  );
  agent.start(source);
  logger(`Started prover agent with concurrency limit of ${proverConfig.proverAgentConcurrency}`);

  signalHandlers.push(() => agent.stop());

  return Promise.resolve([]);
};
