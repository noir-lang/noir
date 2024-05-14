import { BBNativeRollupProver, type BBProverConfig, TestCircuitProver } from '@aztec/bb-prover';
import { type ProvingJobSource } from '@aztec/circuit-types';
import { sleep } from '@aztec/foundation/sleep';
import { type SimulationProvider } from '@aztec/simulator';

import { mkdtemp } from 'fs/promises';
import { join } from 'path';

import { ProverAgent } from './prover-agent.js';

/**
 * Utility class that spawns N prover agents all connected to the same queue
 */
export class ProverPool {
  private agents: ProverAgent[] = [];
  private running = false;

  constructor(private size: number, private agentFactory: (i: number) => ProverAgent | Promise<ProverAgent>) {}

  async start(source: ProvingJobSource): Promise<void> {
    if (this.running) {
      throw new Error('Prover pool is already running');
    }

    // lock the pool state here since creating agents is async
    this.running = true;

    // handle start, stop, start cycles by only creating as many agents as were requested
    for (let i = this.agents.length; i < this.size; i++) {
      this.agents.push(await this.agentFactory(i));
    }

    for (const agent of this.agents) {
      agent.start(source);
      // stagger that start of each agent to avoid contention
      await sleep(10);
    }
  }

  async stop(): Promise<void> {
    if (!this.running) {
      return;
    }

    for (const agent of this.agents) {
      await agent.stop();
    }

    this.running = false;
  }

  async rescale(newSize: number): Promise<void> {
    if (newSize > this.size) {
      this.size = newSize;
      for (let i = this.agents.length; i < newSize; i++) {
        this.agents.push(await this.agentFactory(i));
      }
    } else if (newSize < this.size) {
      this.size = newSize;
      while (this.agents.length > newSize) {
        await this.agents.pop()?.stop();
      }
    }
  }

  static testPool(simulationProvider?: SimulationProvider, size = 1, agentPollIntervalMS = 10): ProverPool {
    return new ProverPool(
      size,
      i => new ProverAgent(new TestCircuitProver(simulationProvider), agentPollIntervalMS, `test-prover-${i}`),
    );
  }

  static nativePool(config: Omit<BBProverConfig, 'circuitFilter'>, size: number, agentPollIntervalMS = 10): ProverPool {
    // TODO generate keys ahead of time so that each agent doesn't have to do it
    return new ProverPool(size, async i => {
      const [acvmWorkingDirectory, bbWorkingDirectory] = await Promise.all([
        mkdtemp(join(config.acvmWorkingDirectory, 'agent-')),
        mkdtemp(join(config.bbWorkingDirectory, 'agent-')),
      ]);
      return new ProverAgent(
        await BBNativeRollupProver.new({
          acvmBinaryPath: config.acvmBinaryPath,
          acvmWorkingDirectory,
          bbBinaryPath: config.bbBinaryPath,
          bbWorkingDirectory,
        }),
        agentPollIntervalMS,
        `bb-prover-${i}`,
      );
    });
  }
}
