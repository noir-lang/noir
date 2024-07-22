import { BBNativeRollupProver, TestCircuitProver } from '@aztec/bb-prover';
import { type ServerCircuitProver } from '@aztec/circuit-types';
import { getProverEnvVars } from '@aztec/prover-client';
import { ProverAgent, createProvingJobSourceClient } from '@aztec/prover-client/prover-agent';
import {
  createAndStartTelemetryClient,
  getConfigEnvVars as getTelemetryClientConfig,
} from '@aztec/telemetry-client/start';

import { type ServiceStarter, parseModuleOptions } from '../util.js';

export const startProverAgent: ServiceStarter = async (options, signalHandlers, logger) => {
  const proverOptions = {
    ...getProverEnvVars(),
    ...parseModuleOptions(options.prover),
  };

  if (!proverOptions.nodeUrl) {
    throw new Error('Starting prover without an orchestrator is not supported');
  }

  logger(`Connecting to prover at ${proverOptions.nodeUrl}`);
  const source = createProvingJobSourceClient(proverOptions.nodeUrl, 'provingJobSource');

  const agentConcurrency =
    // string if it was set as a CLI option, ie start --prover proverAgentConcurrency=10
    typeof proverOptions.proverAgentConcurrency === 'string'
      ? parseInt(proverOptions.proverAgentConcurrency, 10)
      : proverOptions.proverAgentConcurrency;

  const pollInterval =
    // string if it was set as a CLI option, ie start --prover proverAgentPollInterval=10
    typeof proverOptions.proverAgentPollInterval === 'string'
      ? parseInt(proverOptions.proverAgentPollInterval, 10)
      : proverOptions.proverAgentPollInterval;

  const telemetry = createAndStartTelemetryClient(getTelemetryClientConfig());
  let circuitProver: ServerCircuitProver;
  if (proverOptions.realProofs) {
    if (!proverOptions.acvmBinaryPath || !proverOptions.bbBinaryPath) {
      throw new Error('Cannot start prover without simulation or native prover options');
    }

    circuitProver = await BBNativeRollupProver.new(
      {
        acvmBinaryPath: proverOptions.acvmBinaryPath,
        bbBinaryPath: proverOptions.bbBinaryPath,
        acvmWorkingDirectory: proverOptions.acvmWorkingDirectory,
        bbWorkingDirectory: proverOptions.bbWorkingDirectory,
      },
      telemetry,
    );
  } else {
    circuitProver = new TestCircuitProver(telemetry);
  }

  const agent = new ProverAgent(circuitProver, agentConcurrency, pollInterval);
  agent.start(source);
  logger(`Started prover agent with concurrency limit of ${agentConcurrency}`);

  signalHandlers.push(() => agent.stop());

  return Promise.resolve([]);
};
