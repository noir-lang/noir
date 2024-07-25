import { type PXE, createDebugLogger } from '@aztec/aztec.js';

import { Bot } from './bot.js';
import { type BotConfig } from './config.js';

export class BotRunner {
  private log = createDebugLogger('aztec:bot');
  private interval?: NodeJS.Timeout;
  private bot?: Promise<Bot>;
  private pxe?: PXE;
  private running: Set<Promise<void>> = new Set();

  public constructor(private config: BotConfig, dependencies: { pxe?: PXE } = {}) {
    this.pxe = dependencies.pxe;
  }

  /** Initializes the bot if needed. Blocks until the bot setup is finished. */
  public async setup() {
    if (!this.bot) {
      this.log.verbose(`Setting up bot`);
      await this.#createBot();
      this.log.info(`Bot set up completed`);
    }
  }

  /**
   * Initializes the bot if needed and starts sending txs at regular intervals.
   * Blocks until the bot setup is finished.
   */
  public async start() {
    await this.setup();
    if (!this.interval) {
      this.log.info(`Starting bot with interval of ${this.config.txIntervalSeconds}s`);
      this.interval = setInterval(() => this.run(), this.config.txIntervalSeconds * 1000);
    }
  }

  /**
   * Stops sending txs. Returns once all ongoing txs are finished.
   */
  public async stop() {
    if (this.interval) {
      this.log.verbose(`Stopping bot`);
      clearInterval(this.interval);
      this.interval = undefined;
    }
    if (this.running.size > 0) {
      this.log.verbose(`Waiting for ${this.running.size} running txs to finish`);
      await Promise.all(this.running);
    }
    this.log.info(`Stopped bot`);
  }

  /** Returns whether the bot is running. */
  public isRunning() {
    return !!this.interval;
  }

  /**
   * Updates the bot config and recreates the bot. Will stop and restart the bot automatically if it was
   * running when this method was called. Blocks until the new bot is set up.
   */
  public async update(config: BotConfig) {
    this.log.verbose(`Updating bot config`);
    const wasRunning = this.isRunning();
    if (wasRunning) {
      await this.stop();
    }
    this.config = { ...this.config, ...config };
    await this.#createBot();
    this.log.info(`Bot config updated`);
    if (wasRunning) {
      await this.start();
    }
  }

  /**
   * Triggers a single iteration of the bot. Requires the bot to be initialized.
   * Blocks until the run is finished.
   */
  public async run() {
    if (!this.bot) {
      throw new Error(`Bot is not initialized`);
    }
    this.log.verbose(`Manually triggered bot run`);
    const bot = await this.bot;
    const promise = bot.run();
    this.running.add(promise);
    await promise;
    this.running.delete(promise);
  }

  /** Returns the current configuration for the bot. */
  public getConfig() {
    return this.config;
  }

  async #createBot() {
    this.bot = Bot.create(this.config, { pxe: this.pxe });
    await this.bot;
  }
}
