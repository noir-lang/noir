import {
  AztecAddress,
  CircuitsWasm,
  EMITTED_EVENTS_LENGTH,
  Fr,
  KernelCircuitPublicInputs,
  MembershipWitness,
  NEW_L2_TO_L1_MSGS_LENGTH,
  PUBLIC_CALL_STACK_LENGTH,
  PreviousKernelData,
  PublicCallData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  PublicKernelInputs,
  PublicKernelInputsNoPreviousKernel,
  PublicKernelPublicInputs,
  RETURN_VALUES_LENGTH,
  STATE_READS_LENGTH,
  STATE_TRANSITIONS_LENGTH,
  SignedTxRequest,
  StateRead,
  StateTransition,
  VK_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { ContractDataSource, MerkleTreeId, PublicTx, Tx } from '@aztec/types';
import { MerkleTreeOperations } from '@aztec/world-state';

import { PublicExecutionResult, PublicExecutor } from '@aztec/acir-simulator';
import { computeCallStackItemHash } from '@aztec/circuits.js/abis';
import { padArrayEnd, padArrayStart } from '@aztec/foundation/collection';
import { createDebugLogger } from '@aztec/foundation/log';
import { getVerificationKeys } from '../index.js';
import { Proof, PublicProver } from '../prover/index.js';
import { PublicKernelCircuitSimulator } from '../simulator/index.js';
import { ProcessedTx, makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';
import { getCombinedHistoricTreeRoots } from './utils.js';

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
    } else if (tx.isPrivate()) {
      return makeProcessedTx(tx);
    } else {
      return this.makeEmptyProcessedTx();
    }
  }

  protected async processPublicTx(tx: PublicTx): Promise<[PublicKernelPublicInputs, Proof]> {
    const firstExecution = await this.publicExecutor.getPublicExecution(tx.txRequest.txRequest);
    const firstResult: PublicExecutionResult = await this.publicExecutor.execute(firstExecution);
    const executionStack = [firstResult];

    let kernelOutput: KernelCircuitPublicInputs | undefined = undefined;
    let kernelProof: Proof | undefined = undefined;

    while (executionStack.length) {
      const result = executionStack.pop()!;
      executionStack.push(...result.nestedExecutions);
      const callData = await this.getPublicCallData(result);
      [kernelOutput, kernelProof] = await this.runKernelCircuit(tx.txRequest, callData, kernelOutput, kernelProof);
    }

    return [kernelOutput!, kernelProof!];
  }

  protected async runKernelCircuit(
    txRequest: SignedTxRequest,
    callData: PublicCallData,
    previousOutput: KernelCircuitPublicInputs | undefined,
    previousProof: Proof | undefined,
  ): Promise<[KernelCircuitPublicInputs, Proof]> {
    const output = await this.getKernelCircuitOutput(txRequest, callData, previousOutput, previousProof);

    // TODO: This should be set by the public kernel circuit.
    // See https://github.com/AztecProtocol/aztec-packages/issues/482
    const contractTreeInfo = await this.db.getTreeInfo(MerkleTreeId.CONTRACT_TREE);
    const privateDataTreeInfo = await this.db.getTreeInfo(MerkleTreeId.PRIVATE_DATA_TREE);
    const nullifierTreeInfo = await this.db.getTreeInfo(MerkleTreeId.NULLIFIER_TREE);
    const outputRoots = output.constants.historicTreeRoots.privateHistoricTreeRoots;
    outputRoots.nullifierTreeRoot = Fr.fromBuffer(nullifierTreeInfo.root);
    outputRoots.contractTreeRoot = Fr.fromBuffer(contractTreeInfo.root);
    outputRoots.privateDataTreeRoot = Fr.fromBuffer(privateDataTreeInfo.root);

    const proof = await this.publicProver.getPublicKernelCircuitProof(output);
    return [output, proof];
  }

  protected getKernelCircuitOutput(
    txRequest: SignedTxRequest,
    callData: PublicCallData,
    previousOutput: KernelCircuitPublicInputs | undefined,
    previousProof: Proof | undefined,
  ): Promise<KernelCircuitPublicInputs> {
    if (previousOutput && previousProof) {
      if (previousOutput.isPrivateKernel) {
        throw new Error(`Calling public functions from private ones is not implemented yet`);
      }

      const vk = getVerificationKeys().publicKernelCircuit;
      const vkIndex = 0;
      const vkSiblingPath = MembershipWitness.random(VK_TREE_HEIGHT).siblingPath;
      const previousKernel = new PreviousKernelData(previousOutput, previousProof, vk, vkIndex, vkSiblingPath);
      const inputs = new PublicKernelInputs(previousKernel, callData);
      // TODO: This should be set by the public kernel circuit
      inputs.previousKernel.publicInputs.end.publicCallCount = new Fr(1n);
      return this.publicKernel.publicKernelCircuitNonFirstIteration(inputs);
    } else {
      const inputs = new PublicKernelInputsNoPreviousKernel(txRequest, callData);
      return this.publicKernel.publicKernelCircuitNoInput(inputs);
    }
  }

  protected async getPublicCircuitPublicInputs(result: PublicExecutionResult) {
    const publicDataTreeInfo = await this.db.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE);
    const historicPublicDataTreeRoot = Fr.fromBuffer(publicDataTreeInfo.root);
    const callStackPreimages = await this.getPublicCallStackPreimages(result);
    const wasm = await CircuitsWasm.get();
    const publicCallStack = callStackPreimages.map(item => computeCallStackItemHash(wasm, item));

    return PublicCircuitPublicInputs.from({
      args: result.execution.args,
      callContext: result.execution.callContext,
      proverAddress: AztecAddress.random(),
      emittedEvents: padArrayEnd([], Fr.ZERO, EMITTED_EVENTS_LENGTH),
      newL2ToL1Msgs: padArrayEnd([], Fr.ZERO, NEW_L2_TO_L1_MSGS_LENGTH),
      returnValues: padArrayEnd(result.returnValues, Fr.ZERO, RETURN_VALUES_LENGTH),
      stateReads: padArrayEnd(result.stateReads, StateRead.empty(), STATE_READS_LENGTH),
      stateTransitions: padArrayEnd(result.stateTransitions, StateTransition.empty(), STATE_TRANSITIONS_LENGTH),
      publicCallStack,
      historicPublicDataTreeRoot,
    });
  }

  protected async getPublicCallStackItem(result: PublicExecutionResult) {
    return new PublicCallStackItem(
      result.execution.contractAddress,
      result.execution.functionData,
      await this.getPublicCircuitPublicInputs(result),
    );
  }

  protected async getPublicCallStackPreimages(result: PublicExecutionResult) {
    const nested = result.nestedExecutions;
    const preimages: PublicCallStackItem[] = await Promise.all(nested.map(n => this.getPublicCallStackItem(n)));
    if (preimages.length > PUBLIC_CALL_STACK_LENGTH) {
      throw new Error(`Public call stack size exceeded (max ${PUBLIC_CALL_STACK_LENGTH}, got ${preimages.length})`);
    }

    const emptyPreimage = PublicCallStackItem.empty();
    // TODO: Remove the msgSender set once circuits dont validate empty call stack items
    emptyPreimage.publicInputs.callContext.msgSender = result.execution.contractAddress;
    // Top of the stack is at the end of the array, so we padStart
    return padArrayStart(preimages, emptyPreimage, PUBLIC_CALL_STACK_LENGTH);
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
   * @returns A corresponding PublicCallData object.
   */
  protected async getPublicCallData(result: PublicExecutionResult) {
    const bytecodeHash = await this.getBytecodeHash(result);
    const callStackItem = await this.getPublicCallStackItem(result);
    const preimages = await this.getPublicCallStackPreimages(result);
    const portalContractAddress = result.execution.callContext.portalContractAddress.toField();
    const proof = await this.publicProver.getPublicCircuitProof(callStackItem.publicInputs);
    return new PublicCallData(callStackItem, preimages, proof, portalContractAddress, bytecodeHash);
  }
}
