import { MemoryProvingQueue } from './memory-proving-queue.js';
import { type ProvingAgent } from './prover-agent.js';
import { type ProvingQueue } from './proving-queue.js';

/**
 * Utility class that spawns N prover agents all connected to the same queue
 */
export class ProverPool {
  private agents: ProvingAgent[] = [];
  private running = false;

  constructor(
    private size: number,
    private agentFactory: (i: number) => ProvingAgent | Promise<ProvingAgent>,
    public readonly queue: ProvingQueue = new MemoryProvingQueue(),
  ) {}

  async start(): Promise<void> {
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
      agent.start(this.queue);
    }
  }

  async stop(): Promise<void> {
    if (!this.running) {
      throw new Error('Prover pool is not running');
    }

    for (const agent of this.agents) {
      await agent.stop();
    }

    this.running = false;
  }
}
