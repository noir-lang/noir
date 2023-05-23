import { PublicExecution, PublicExecutionResult, PublicExecutor, isPublicExecutionResult } from '@aztec/acir-simulator';
import {
  ARGS_LENGTH,
  AztecAddress,
  CircuitsWasm,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  EMITTED_EVENTS_LENGTH,
  Fr,
  KERNEL_PUBLIC_DATA_READS_LENGTH,
  KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
  KernelCircuitPublicInputs,
  MembershipWitness,
  NEW_L2_TO_L1_MSGS_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  PreviousKernelData,
  Proof,
  PublicCallData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicKernelInputs,
  PublicKernelInputsNoPreviousKernel,
  PublicKernelPublicInputs,
  RETURN_VALUES_LENGTH,
  SignedTxRequest,
  VK_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { computeCallStackItemHash } from '@aztec/circuits.js/abis';
import { isArrayEmpty, padArrayEnd, padArrayStart } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { ContractDataSource, MerkleTreeId, PrivateTx, PublicTx, Tx } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';
import { getVerificationKeys } from '../index.js';
import { EmptyPublicProver } from '../prover/empty.js';
import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { getPublicExecutor } from '../simulator/public_executor.js';
import { WasmPublicKernelCircuitSimulator } from '../simulator/public_kernel.js';
import { ProcessedTx, makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';
import { getCombinedHistoricTreeRoots } from './utils.js';
import { Tuple, mapTuple } from '@aztec/foundation/serialize';

/**
 * Creates new instances of PublicProcessor given the provided merkle tree db and contract data source.
 */
export class PublicProcessorFactory {
  constructor(private merkleTree: MerkleTreeOperations, private contractDataSource: ContractDataSource) {}

  /**
   * Creates a new instance of a PublicProcessor.
   * @returns A new instance of a PublicProcessor.
   */
  public create() {
    return new PublicProcessor(
      this.merkleTree,
      getPublicExecutor(this.merkleTree, this.contractDataSource),
      new WasmPublicKernelCircuitSimulator(),
      new EmptyPublicProver(),
      this.contractDataSource,
    );
  }
}

/**
 * Converts Txs lifted from the P2P module into ProcessedTx objects by executing
 * any public function calls in them. Txs with private calls only are unaffected.
 */
export class PublicProcessor {
  constructor(
    protected db: MerkleTreeOperations,
    protected publicExecutor: PublicExecutor,
    protected publicKernel: PublicKernelCircuitSimulator,
    protected publicProver: PublicProver,
    protected contractDataSource: ContractDataSource,

    private log = createDebugLogger('aztec:sequencer:public-processor'),
  ) {}

  /**
   * Run each tx through the public circuit and the public kernel circuit if needed.
   * @param txs - Txs to process.
   * @returns The list of processed txs with their circuit simulation outputs.
   */
  public async process(txs: Tx[]): Promise<[ProcessedTx[], Tx[]]> {
    const result: ProcessedTx[] = [];
    const failed: Tx[] = [];

    for (const tx of txs) {
      this.log(`Processing tx ${await tx.getTxHash()}`);
      try {
        result.push(await this.processTx(tx));
      } catch (err) {
        this.log(`Error processing tx ${await tx.getTxHash()}: ${err}`);
        failed.push(tx);
      }
    }
    return [result, failed];
  }

  /**
   * Makes an empty processed tx. Useful for padding a block to a power of two number of txs.
   * @returns A processed tx with empty data.
   */
  public async makeEmptyProcessedTx() {
    const historicTreeRoots = await getCombinedHistoricTreeRoots(this.db);
    return makeEmptyProcessedTx(historicTreeRoots);
  }

  protected async processTx(tx: Tx): Promise<ProcessedTx> {
    if (tx.isPublic()) {
      const [publicKernelOutput, publicKernelProof] = await this.processPublicTx(tx);
      return makeProcessedTx(tx, publicKernelOutput, publicKernelProof);
    } else if (tx.isPrivate() && !isArrayEmpty(tx.data.end.publicCallStack, item => item.isZero())) {
      const [publicKernelOutput, publicKernelProof] = await this.processEnqueuedPublicCalls(tx);
      return makeProcessedTx(tx, publicKernelOutput, publicKernelProof);
    } else if (tx.isPrivate()) {
      return makeProcessedTx(tx);
    } else {
      return this.makeEmptyProcessedTx();
    }
  }

  protected async processEnqueuedPublicCalls(tx: PrivateTx): Promise<[PublicKernelPublicInputs, Proof]> {
    this.log(`Executing enqueued public calls for tx ${await tx.getTxHash()}`);
    if (!tx.enqueuedPublicFunctionCalls) throw new Error(`Missing preimages for enqueued public calls`);

    // We execute the requests in order, which means reversing the input as the stack pops from the end of the array
    const executionStack: (PublicExecution | PublicExecutionResult)[] = [...tx.enqueuedPublicFunctionCalls].reverse();
    return await this.processExecutionStack(executionStack, undefined, tx.data, tx.proof);
  }

  protected async processPublicTx(tx: PublicTx): Promise<[PublicKernelPublicInputs, Proof]> {
    this.log(`Executing public tx request ${await tx.getTxHash()}`);
    const firstExecution = await this.publicExecutor.getPublicExecution(tx.txRequest.txRequest);
    const firstResult: PublicExecutionResult = await this.publicExecutor.execute(firstExecution);
    const executionStack = [firstResult];
    return await this.processExecutionStack(executionStack, tx.txRequest, undefined, undefined);
  }

  protected async processExecutionStack(
    executionStack: (PublicExecution | PublicExecutionResult)[],
    txRequest: SignedTxRequest | undefined,
    kernelOutput: KernelCircuitPublicInputs | undefined,
    kernelProof: Proof | undefined,
  ): Promise<[PublicKernelPublicInputs, Proof]> {
    if (!executionStack.length) throw new Error(`Execution stack cannot be empty`);

    while (executionStack.length) {
      const current = executionStack.pop()!;
      const isExecutionRequest = !isPublicExecutionResult(current);
      const result = isExecutionRequest ? await this.publicExecutor.execute(current) : current;
      const functionSelector = result.execution.functionData.functionSelectorBuffer.toString('hex');
      this.log(`Running public kernel circuit for ${functionSelector}@${result.execution.contractAddress.toString()}`);
      executionStack.push(...result.nestedExecutions);
      const preimages = await this.getPublicCallStackPreimages(result);
      const callData = await this.getPublicCallData(result, preimages, isExecutionRequest);
      [kernelOutput, kernelProof] = await this.runKernelCircuit(callData, txRequest, kernelOutput, kernelProof);
    }

    return [kernelOutput!, kernelProof!];
  }

  protected async runKernelCircuit(
    callData: PublicCallData,
    txRequest: SignedTxRequest | undefined,
    previousOutput: KernelCircuitPublicInputs | undefined,
    previousProof: Proof | undefined,
  ): Promise<[KernelCircuitPublicInputs, Proof]> {
    const output = await this.getKernelCircuitOutput(callData, txRequest, previousOutput, previousProof);
    const proof = await this.publicProver.getPublicKernelCircuitProof(output);
    return [output, proof];
  }

  protected async getKernelCircuitOutput(
    callData: PublicCallData,
    txRequest: SignedTxRequest | undefined,
    previousOutput: KernelCircuitPublicInputs | undefined,
    previousProof: Proof | undefined,
  ): Promise<KernelCircuitPublicInputs> {
    if (previousOutput?.isPrivate && previousProof) {
      // Run the public kernel circuit with previous private kernel
      const previousKernel = this.getPreviousKernelData(previousOutput, previousProof);
      const inputs = new PublicKernelInputs(previousKernel, callData);
      return this.publicKernel.publicKernelCircuitPrivateInput(inputs);
    } else if (previousOutput && previousProof) {
      // Run the public kernel circuit with previous public kernel
      const previousKernel = this.getPreviousKernelData(previousOutput, previousProof);
      const inputs = new PublicKernelInputs(previousKernel, callData);
      return this.publicKernel.publicKernelCircuitNonFirstIteration(inputs);
    } else if (txRequest) {
      // Run the public kernel circuit with no previous kernel
      const treeRoots = await getCombinedHistoricTreeRoots(this.db);
      const inputs = new PublicKernelInputsNoPreviousKernel(txRequest, callData, treeRoots);
      return this.publicKernel.publicKernelCircuitNoInput(inputs);
    } else {
      throw new Error(`No public kernel circuit for inputs`);
    }
  }

  protected getPreviousKernelData(previousOutput: KernelCircuitPublicInputs, previousProof: Proof): PreviousKernelData {
    const vk = getVerificationKeys().publicKernelCircuit;
    const vkIndex = 0;
    const vkSiblingPath = MembershipWitness.random(VK_TREE_HEIGHT).siblingPath;
    return new PreviousKernelData(previousOutput, previousProof, vk, vkIndex, vkSiblingPath);
  }

  protected async getPublicCircuitPublicInputs(result: PublicExecutionResult) {
    const publicDataTreeInfo = await this.db.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE);
    const historicPublicDataTreeRoot = Fr.fromBuffer(publicDataTreeInfo.root);
    const callStackPreimages = await this.getPublicCallStackPreimages(result);
    const wasm = await CircuitsWasm.get();
    const publicCallStack = mapTuple(callStackPreimages, item =>
      item.isEmpty() ? Fr.zero() : computeCallStackItemHash(wasm, item),
    );

    return PublicCircuitPublicInputs.from({
      callContext: result.execution.callContext,
      proverAddress: AztecAddress.random(),
      args: padArrayEnd(result.execution.args, Fr.ZERO, ARGS_LENGTH),
      emittedEvents: padArrayEnd([], Fr.ZERO, EMITTED_EVENTS_LENGTH),
      newL2ToL1Msgs: padArrayEnd([], Fr.ZERO, NEW_L2_TO_L1_MSGS_LENGTH),
      returnValues: padArrayEnd(result.returnValues, Fr.ZERO, RETURN_VALUES_LENGTH),
      contractStorageReads: padArrayEnd(
        result.contractStorageReads,
        ContractStorageRead.empty(),
        KERNEL_PUBLIC_DATA_READS_LENGTH,
      ),
      contractStorageUpdateRequests: padArrayEnd(
        result.contractStorageUpdateRequests,
        ContractStorageUpdateRequest.empty(),
        KERNEL_PUBLIC_DATA_UPDATE_REQUESTS_LENGTH,
      ),
      publicCallStack,
      historicPublicDataTreeRoot,
    });
  }

  protected async getPublicCallStackItem(result: PublicExecutionResult, isExecutionRequest = false) {
    return new PublicCallStackItem(
      result.execution.contractAddress,
      result.execution.functionData,
      await this.getPublicCircuitPublicInputs(result),
      isExecutionRequest,
    );
  }

  protected async getPublicCallStackPreimages(result: PublicExecutionResult) {
    const nested = result.nestedExecutions;
    const preimages: PublicCallStackItem[] = await Promise.all(nested.map(n => this.getPublicCallStackItem(n)));
    if (preimages.length > PUBLIC_CALL_STACK_LENGTH) {
      throw new Error(`Public call stack size exceeded (max ${PUBLIC_CALL_STACK_LENGTH}, got ${preimages.length})`);
    }

    // Top of the stack is at the end of the array, so we padStart
    return padArrayStart(preimages, PublicCallStackItem.empty(), PUBLIC_CALL_STACK_LENGTH);
  }

  protected getBytecodeHash(_result: PublicExecutionResult) {
    // TODO: Determine how to calculate bytecode hash. Circuits just check it isn't zero for now.
    // See https://github.com/AztecProtocol/aztec3-packages/issues/378
    const bytecodeHash = new Fr(1n);
    return Promise.resolve(bytecodeHash);
  }

  /**
   * Calculates the PublicCircuitOutput for this execution result along with its proof,
   * and assembles a PublicCallData object from it.
   * @param result - The execution result.
   * @param preimages - The preimages of the callstack items.
   * @param isExecutionRequest - Whether the current callstack item should be considered a public fn execution request.
   * @returns A corresponding PublicCallData object.
   */
  protected async getPublicCallData(
    result: PublicExecutionResult,
    preimages: Tuple<PublicCallStackItem, typeof PUBLIC_CALL_STACK_LENGTH>,
    isExecutionRequest = false,
  ) {
    const bytecodeHash = await this.getBytecodeHash(result);
    const callStackItem = await this.getPublicCallStackItem(result, isExecutionRequest);
    const portalContractAddress = result.execution.callContext.portalContractAddress.toField();
    const proof = await this.publicProver.getPublicCircuitProof(callStackItem.publicInputs);
    return new PublicCallData(callStackItem, preimages, proof, portalContractAddress, bytecodeHash);
  }
}
