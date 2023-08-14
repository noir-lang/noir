import { PublicExecution, PublicExecutionResult, PublicExecutor, isPublicExecutionResult } from '@aztec/acir-simulator';
import {
  AztecAddress,
  CircuitsWasm,
  ContractStorageRead,
  ContractStorageUpdateRequest,
  Fr,
  GlobalVariables,
  HistoricBlockData,
  KernelCircuitPublicInputs,
  MAX_NEW_COMMITMENTS_PER_CALL,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_CALL,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL,
  MAX_PUBLIC_DATA_READS_PER_CALL,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
  MembershipWitness,
  PreviousKernelData,
  Proof,
  PublicCallData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicKernelInputs,
  PublicKernelPublicInputs,
  RETURN_VALUES_LENGTH,
  VK_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { computeCallStackItemHash, computeVarArgsHash } from '@aztec/circuits.js/abis';
import { isArrayEmpty, padArrayEnd, padArrayStart } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { Tuple, mapTuple, to2Fields } from '@aztec/foundation/serialize';
import { ContractDataSource, FunctionL2Logs, L1ToL2MessageSource, MerkleTreeId, Tx } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';

import { getVerificationKeys } from '../index.js';
import { EmptyPublicProver } from '../prover/empty.js';
import { PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { getPublicExecutor } from '../simulator/public_executor.js';
import { WasmPublicKernelCircuitSimulator } from '../simulator/public_kernel.js';
import { ProcessedTx, makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';
import { getHistoricBlockData } from './utils.js';

/**
 * Creates new instances of PublicProcessor given the provided merkle tree db and contract data source.
 */
export class PublicProcessorFactory {
  constructor(
    private merkleTree: MerkleTreeOperations,
    private contractDataSource: ContractDataSource,
    private l1Tol2MessagesDataSource: L1ToL2MessageSource,
  ) {}

  /**
   * Creates a new instance of a PublicProcessor.
   * @param prevGlobalVariables - The global variables for the previous block, used to calculate the prev global variables hash.
   * @param globalVariables - The global variables for the block being processed.
   * @returns A new instance of a PublicProcessor.
   */
  public async create(
    prevGlobalVariables: GlobalVariables,
    globalVariables: GlobalVariables,
  ): Promise<PublicProcessor> {
    const blockData = await getHistoricBlockData(this.merkleTree, prevGlobalVariables);
    return new PublicProcessor(
      this.merkleTree,
      getPublicExecutor(this.merkleTree, this.contractDataSource, this.l1Tol2MessagesDataSource, blockData),
      new WasmPublicKernelCircuitSimulator(),
      new EmptyPublicProver(),
      this.contractDataSource,
      globalVariables,
      blockData,
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
    protected globalVariables: GlobalVariables,
    protected blockData: HistoricBlockData,

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
  public makeEmptyProcessedTx(): Promise<ProcessedTx> {
    const { chainId, version } = this.globalVariables;
    return makeEmptyProcessedTx(this.blockData, chainId, version);
  }

  protected async processTx(tx: Tx): Promise<ProcessedTx> {
    if (!isArrayEmpty(tx.data.end.publicCallStack, item => item.isZero())) {
      const [publicKernelOutput, publicKernelProof, newUnencryptedFunctionLogs] = await this.processEnqueuedPublicCalls(
        tx,
      );
      tx.unencryptedLogs.addFunctionLogs(newUnencryptedFunctionLogs);

      return makeProcessedTx(tx, publicKernelOutput, publicKernelProof);
    } else {
      return makeProcessedTx(tx);
    }
  }

  protected async processEnqueuedPublicCalls(tx: Tx): Promise<[PublicKernelPublicInputs, Proof, FunctionL2Logs[]]> {
    this.log(`Executing enqueued public calls for tx ${await tx.getTxHash()}`);
    if (!tx.enqueuedPublicFunctionCalls) throw new Error(`Missing preimages for enqueued public calls`);

    // We execute the requests in order, which means reversing the input as the stack pops from the end of the array
    const executionStack: (PublicExecution | PublicExecutionResult)[] = [...tx.enqueuedPublicFunctionCalls].reverse();

    let kernelOutput = tx.data;
    let kernelProof = tx.proof;
    const newUnencryptedFunctionLogs: FunctionL2Logs[] = [];

    while (executionStack.length) {
      const current = executionStack.pop()!;
      const isExecutionRequest = !isPublicExecutionResult(current);
      const result = isExecutionRequest ? await this.publicExecutor.execute(current, this.globalVariables) : current;
      newUnencryptedFunctionLogs.push(result.unencryptedLogs);
      const functionSelector = result.execution.functionData.functionSelectorBuffer.toString('hex');
      this.log(`Running public kernel circuit for ${functionSelector}@${result.execution.contractAddress.toString()}`);
      executionStack.push(...result.nestedExecutions);
      const preimages = await this.getPublicCallStackPreimages(result);
      const callData = await this.getPublicCallData(result, preimages, isExecutionRequest);
      [kernelOutput, kernelProof] = await this.runKernelCircuit(callData, kernelOutput, kernelProof);
    }

    return [kernelOutput, kernelProof, newUnencryptedFunctionLogs];
  }

  protected async runKernelCircuit(
    callData: PublicCallData,
    previousOutput: KernelCircuitPublicInputs,
    previousProof: Proof,
  ): Promise<[KernelCircuitPublicInputs, Proof]> {
    const output = await this.getKernelCircuitOutput(callData, previousOutput, previousProof);
    const proof = await this.publicProver.getPublicKernelCircuitProof(output);
    return [output, proof];
  }

  protected getKernelCircuitOutput(
    callData: PublicCallData,
    previousOutput: KernelCircuitPublicInputs,
    previousProof: Proof,
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

    // TODO(https://github.com/AztecProtocol/aztec-packages/issues/1165) --> set this in Noir
    const unencryptedLogsHash = to2Fields(result.unencryptedLogs.hash());
    const unencryptedLogPreimagesLength = new Fr(result.unencryptedLogs.getSerializedLength());

    return PublicCircuitPublicInputs.from({
      callContext: result.execution.callContext,
      proverAddress: AztecAddress.ZERO,
      argsHash: await computeVarArgsHash(wasm, result.execution.args),
      newCommitments: padArrayEnd(result.newCommitments, Fr.ZERO, MAX_NEW_COMMITMENTS_PER_CALL),
      newNullifiers: padArrayEnd(result.newNullifiers, Fr.ZERO, MAX_NEW_NULLIFIERS_PER_CALL),
      newL2ToL1Msgs: padArrayEnd(result.newL2ToL1Messages, Fr.ZERO, MAX_NEW_L2_TO_L1_MSGS_PER_CALL),
      returnValues: padArrayEnd(result.returnValues, Fr.ZERO, RETURN_VALUES_LENGTH),
      contractStorageReads: padArrayEnd(
        result.contractStorageReads,
        ContractStorageRead.empty(),
        MAX_PUBLIC_DATA_READS_PER_CALL,
      ),
      contractStorageUpdateRequests: padArrayEnd(
        result.contractStorageUpdateRequests,
        ContractStorageUpdateRequest.empty(),
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_CALL,
      ),
      publicCallStack,
      unencryptedLogsHash,
      unencryptedLogPreimagesLength,
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
    if (preimages.length > MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL) {
      throw new Error(
        `Public call stack size exceeded (max ${MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL}, got ${preimages.length})`,
      );
    }

    // Top of the stack is at the end of the array, so we padStart
    return padArrayStart(preimages, PublicCallStackItem.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL);
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
    preimages: Tuple<PublicCallStackItem, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_CALL>,
    isExecutionRequest = false,
  ) {
    const bytecodeHash = await this.getBytecodeHash(result);
    const callStackItem = await this.getPublicCallStackItem(result, isExecutionRequest);
    const portalContractAddress = result.execution.callContext.portalContractAddress.toField();
    const proof = await this.publicProver.getPublicCircuitProof(callStackItem.publicInputs);
    return new PublicCallData(callStackItem, preimages, proof, portalContractAddress, bytecodeHash);
  }
}
