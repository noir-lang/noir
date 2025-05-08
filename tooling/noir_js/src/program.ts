import { CompiledCircuit } from '@noir-lang/types';
import { generateWitness } from './witness_generation.js';
import initAbi, { abiDecode, InputMap, InputValue } from '@noir-lang/noirc_abi';
import initACVM, { compressWitnessStack, ForeignCallHandler } from '@noir-lang/acvm_js';
import pino, { Logger } from 'pino';
import { Timer } from './utils.js';

export class Noir {
  constructor(private circuit: CompiledCircuit) {}

  /** @ignore */
  async init(): Promise<void> {
    this.#init();
  }

  /** @ignore */
  async #init(logger?: Logger): Promise<void> {
    // If these are available, then we are in the
    // web environment. For the node environment, this
    // is a no-op.
    const init_timer = new Timer();
    if (typeof initAbi === 'function') {
      await Promise.all([initAbi(), initACVM()]);
    }
    logger?.info({ duration: init_timer.ms() }, 'Initializing WASM');
  }

  /**
   * @description
   * Allows to execute a circuit to get its witness and return value.
   *
   * @example
   * ```typescript
   * async execute(inputs)
   * ```
   */
  async execute(
    inputs: InputMap,
    foreignCallHandler?: ForeignCallHandler,
  ): Promise<{ witness: Uint8Array; returnValue: InputValue }> {
    const logger = pino({ name: 'noir_js::execute' });
    const total_timer = new Timer();

    await this.#init(logger);

    const witness_stack = await generateWitness(this.circuit, inputs, foreignCallHandler, logger);

    const abi_decoding_timer = new Timer();
    const main_witness = witness_stack[0].witness;
    const { return_value: returnValue } = abiDecode(this.circuit.abi, main_witness);
    logger.info({ duration: abi_decoding_timer.ms() }, 'ABI decoding');

    const witness_compression_timer = new Timer();
    const witness = compressWitnessStack(witness_stack);
    logger.info({ duration: witness_compression_timer.ms() }, 'Witness compression');

    logger.info({ duration: total_timer.ms() }, 'Total execution');

    return { witness, returnValue };
  }
}
