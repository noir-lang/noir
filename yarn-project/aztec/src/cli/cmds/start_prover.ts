import { type ProvingJobSource } from '@aztec/circuit-types';
import { type ProverClientConfig, getProverEnvVars } from '@aztec/prover-client';
import { ProverPool, createProvingJobSourceClient } from '@aztec/prover-client/prover-pool';

import { tmpdir } from 'node:os';

import { type ServiceStarter, parseModuleOptions } from '../util.js';

type ProverOptions = ProverClientConfig &
  Partial<{
    proverUrl: string;
    agents: string;
    acvmBinaryPath?: string;
    bbBinaryPath?: string;
    simulate?: string;
  }>;

export const startProver: ServiceStarter = async (options, signalHandlers, logger) => {
  const proverOptions: ProverOptions = {
    proverUrl: process.env.PROVER_URL,
    ...getProverEnvVars(),
    ...parseModuleOptions(options.prover),
  };
  let source: ProvingJobSource;

  if (typeof proverOptions.proverUrl === 'string') {
    logger(`Connecting to prover at ${proverOptions.proverUrl}`);
    source = createProvingJobSourceClient(proverOptions.proverUrl, 'provingJobSource');
  } else {
    throw new Error('Starting prover without an orchestrator is not supported');
  }

  const agentCount = proverOptions.agents ? parseInt(proverOptions.agents, 10) : proverOptions.proverAgents;
  if (agentCount === 0 || !Number.isSafeInteger(agentCount)) {
    throw new Error('Cannot start prover without agents');
  }

  let pool: ProverPool;
  if (proverOptions.simulate) {
    pool = ProverPool.testPool(undefined, agentCount);
  } else if (proverOptions.acvmBinaryPath && proverOptions.bbBinaryPath) {
    pool = ProverPool.nativePool(
      {
        acvmBinaryPath: proverOptions.acvmBinaryPath,
        bbBinaryPath: proverOptions.bbBinaryPath,
        acvmWorkingDirectory: tmpdir(),
        bbWorkingDirectory: tmpdir(),
      },
      agentCount,
    );
  } else {
    throw new Error('Cannot start prover without simulation or native prover options');
  }

  logger(`Starting ${agentCount} prover agents`);
  await pool.start(source);
  signalHandlers.push(() => pool.stop());

  return Promise.resolve([]);
};
