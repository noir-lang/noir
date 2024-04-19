import { type ProvingQueueConsumer } from './proving-queue.js';

/** An agent that reads proving jobs from the queue, creates the proof and submits back the result */
export interface ProvingAgent {
  /**
   * Starts the agent to read proving jobs from the queue.
   * @param queue - The queue to read proving jobs from.
   */
  start(queue: ProvingQueueConsumer): void;

  /**
   * Stops the agent. Does nothing if the agent is not running.
   */
  stop(): Promise<void>;
}
