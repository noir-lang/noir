import {
  AuthWitness,
  L1NotePayload,
  MerkleTreeId,
  Note,
  type NoteStatus,
  NullifierMembershipWitness,
  PublicDataWitness,
  PublicDataWrite,
  TaggedLog,
  type UnencryptedL2Log,
} from '@aztec/circuit-types';
import { type CircuitWitnessGenerationStats } from '@aztec/circuit-types/stats';
import {
  CallContext,
  FunctionData,
  Gas,
  GlobalVariables,
  Header,
  type KeyValidationRequest,
  NULLIFIER_SUBTREE_HEIGHT,
  type NULLIFIER_TREE_HEIGHT,
  type NullifierLeafPreimage,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  type PUBLIC_DATA_TREE_HEIGHT,
  PrivateCallStackItem,
  PrivateCircuitPublicInputs,
  PrivateContextInputs,
  PublicCallRequest,
  PublicDataTreeLeaf,
  type PublicDataTreeLeafPreimage,
  TxContext,
  computeContractClassId,
  deriveKeys,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import { Aes128, Schnorr } from '@aztec/circuits.js/barretenberg';
import { computePublicDataTreeLeafSlot, siloNoteHash, siloNullifier } from '@aztec/circuits.js/hash';
import {
  type ContractArtifact,
  type FunctionAbi,
  FunctionSelector,
  type NoteSelector,
  countArgumentsSize,
} from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, GrumpkinScalar, type Point } from '@aztec/foundation/fields';
import { type Logger, applyStringFormatting } from '@aztec/foundation/log';
import { Timer } from '@aztec/foundation/timer';
import { type KeyStore } from '@aztec/key-store';
import { ContractDataOracle } from '@aztec/pxe';
import {
  ContractsDataSourcePublicDB,
  ExecutionError,
  type ExecutionNoteCache,
  type MessageLoadOracleInputs,
  type NoteData,
  Oracle,
  type PackedValuesCache,
  PublicExecutor,
  type TypedOracle,
  WorldStateDB,
  acvm,
  createSimulationError,
  extractCallStack,
  pickNotes,
  toACVMWitness,
  witnessMapToFields,
} from '@aztec/simulator';
import { type ContractInstance, type ContractInstanceWithAddress } from '@aztec/types/contracts';
import { MerkleTreeSnapshotOperationsFacade, type MerkleTrees } from '@aztec/world-state';

import { type TXEDatabase } from '../util/txe_database.js';
import { TXEPublicContractDataSource } from '../util/txe_public_contract_data_source.js';
import { TXEPublicStateDB } from '../util/txe_public_state_db.js';

export class TXE implements TypedOracle {
  private blockNumber = 0;
  private sideEffectsCounter = 0;
  private contractAddress: AztecAddress;
  private msgSender: AztecAddress;
  private functionSelector = FunctionSelector.fromField(new Fr(0));

  private contractDataOracle: ContractDataOracle;

  private version: Fr = Fr.ONE;
  private chainId: Fr = Fr.ONE;

  constructor(
    private logger: Logger,
    private trees: MerkleTrees,
    private packedValuesCache: PackedValuesCache,
    private noteCache: ExecutionNoteCache,
    private keyStore: KeyStore,
    private txeDatabase: TXEDatabase,
  ) {
    this.contractDataOracle = new ContractDataOracle(txeDatabase);
    this.contractAddress = AztecAddress.random();
    this.msgSender = AztecAddress.fromField(new Fr(0));
  }

  // Utils

  getChainId() {
    return Promise.resolve(this.chainId);
  }

  getVersion() {
    return Promise.resolve(this.version);
  }

  getMsgSender() {
    return this.msgSender;
  }

  getFunctionSelector() {
    return this.functionSelector;
  }

  setMsgSender(msgSender: Fr) {
    this.msgSender = msgSender;
  }

  setFunctionSelector(functionSelector: FunctionSelector) {
    this.functionSelector = functionSelector;
  }

  getSideEffectsCounter() {
    return this.sideEffectsCounter;
  }

  setSideEffectsCounter(sideEffectsCounter: number) {
    this.sideEffectsCounter = sideEffectsCounter;
  }

  setContractAddress(contractAddress: AztecAddress) {
    this.contractAddress = contractAddress;
  }

  setBlockNumber(blockNumber: number) {
    this.blockNumber = blockNumber;
  }

  getTrees() {
    return this.trees;
  }

  getContractDataOracle() {
    return this.contractDataOracle;
  }

  getTXEDatabase() {
    return this.txeDatabase;
  }

  getKeyStore() {
    return this.keyStore;
  }

  async addContractInstance(contractInstance: ContractInstanceWithAddress) {
    await this.txeDatabase.addContractInstance(contractInstance);
  }

  async addContractArtifact(artifact: ContractArtifact) {
    const contractClass = getContractClassFromArtifact(artifact);
    await this.txeDatabase.addContractArtifact(computeContractClassId(contractClass), artifact);
  }

  async getPrivateContextInputs(
    blockNumber: number,
    sideEffectsCounter = this.sideEffectsCounter,
    isStaticCall = false,
    isDelegateCall = false,
  ) {
    const trees = this.getTrees();
    const stateReference = await trees.getStateReference(false);
    const inputs = PrivateContextInputs.empty();
    inputs.historicalHeader.globalVariables.blockNumber = new Fr(blockNumber);
    inputs.historicalHeader.state = stateReference;
    inputs.callContext.msgSender = this.msgSender;
    inputs.callContext.storageContractAddress = this.contractAddress;
    inputs.callContext.isStaticCall = isStaticCall;
    inputs.callContext.isDelegateCall = isDelegateCall;
    inputs.startSideEffectCounter = sideEffectsCounter;
    inputs.callContext.functionSelector = this.functionSelector;
    return inputs;
  }

  getPublicContextInputs() {
    const inputs = {
      argsHash: new Fr(0),
      isStaticCall: false,
      toFields: function () {
        return [this.argsHash, new Fr(this.isStaticCall)];
      },
    };
    return inputs;
  }

  async avmOpcodeNullifierExists(innerNullifier: Fr, targetAddress: AztecAddress): Promise<boolean> {
    const nullifier = siloNullifier(targetAddress, innerNullifier!);
    const db = this.trees.asLatest();
    const index = await db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    return index !== undefined;
  }

  async avmOpcodeEmitNullifier(nullifier: Fr) {
    const db = this.trees.asLatest();
    const siloedNullifier = siloNullifier(this.contractAddress, nullifier);
    await db.batchInsert(MerkleTreeId.NULLIFIER_TREE, [siloedNullifier.toBuffer()], NULLIFIER_SUBTREE_HEIGHT);
    return Promise.resolve();
  }

  async avmOpcodeEmitNoteHash(innerNoteHash: Fr) {
    const db = this.trees.asLatest();
    const noteHash = siloNoteHash(this.contractAddress, innerNoteHash);
    await db.appendLeaves(MerkleTreeId.NOTE_HASH_TREE, [noteHash]);
    return Promise.resolve();
  }

  deriveKeys(secret: Fr) {
    return deriveKeys(secret);
  }

  async addAuthWitness(address: AztecAddress, messageHash: Fr) {
    const account = this.txeDatabase.getAccount(address);
    const privateKey = await this.keyStore.getMasterSecretKey(account.publicKeys.masterIncomingViewingPublicKey);
    const schnorr = new Schnorr();
    const signature = schnorr.constructSignature(messageHash.toBuffer(), privateKey).toBuffer();
    const authWitness = new AuthWitness(messageHash, [...signature]);
    return this.txeDatabase.addAuthWitness(authWitness.requestHash, authWitness.witness);
  }

  async addNullifiers(contractAddress: AztecAddress, nullifiers: Fr[]) {
    const db = this.trees.asLatest();
    const siloedNullifiers = nullifiers.map(nullifier => siloNullifier(contractAddress, nullifier).toBuffer());

    await db.batchInsert(MerkleTreeId.NULLIFIER_TREE, siloedNullifiers, NULLIFIER_SUBTREE_HEIGHT);
  }

  async addNoteHashes(contractAddress: AztecAddress, innerNoteHashes: Fr[]) {
    const db = this.trees.asLatest();
    const siloedNoteHashes = innerNoteHashes.map(innerNoteHash => siloNoteHash(contractAddress, innerNoteHash));
    await db.appendLeaves(MerkleTreeId.NOTE_HASH_TREE, siloedNoteHashes);
  }

  // TypedOracle

  getBlockNumber() {
    return Promise.resolve(this.blockNumber);
  }

  getContractAddress() {
    return Promise.resolve(this.contractAddress);
  }

  getRandomField() {
    return Fr.random();
  }

  packArgumentsArray(args: Fr[]) {
    return Promise.resolve(this.packedValuesCache.pack(args));
  }

  packReturns(returns: Fr[]) {
    return Promise.resolve(this.packedValuesCache.pack(returns));
  }

  unpackReturns(returnsHash: Fr) {
    return Promise.resolve(this.packedValuesCache.unpack(returnsHash));
  }

  getKeyValidationRequest(pkMHash: Fr): Promise<KeyValidationRequest> {
    return this.keyStore.getKeyValidationRequest(pkMHash, this.contractAddress);
  }

  async getContractInstance(address: AztecAddress): Promise<ContractInstance> {
    const contractInstance = await this.contractDataOracle.getContractInstance(address);
    if (!contractInstance) {
      throw new Error(`Contract instance not found for address ${address}`);
    }
    return contractInstance;
  }

  getMembershipWitness(_blockNumber: number, _treeId: MerkleTreeId, _leafValue: Fr): Promise<Fr[] | undefined> {
    throw new Error('Method not implemented.');
  }

  async getSiblingPath(blockNumber: number, treeId: MerkleTreeId, leafIndex: Fr) {
    const committedDb = new MerkleTreeSnapshotOperationsFacade(this.trees, blockNumber);
    const result = await committedDb.getSiblingPath(treeId, leafIndex.toBigInt());
    return result.toFields();
  }

  async getNullifierMembershipWitness(
    blockNumber: number,
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    const committedDb = new MerkleTreeSnapshotOperationsFacade(this.trees, blockNumber);
    const index = await committedDb.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    if (!index) {
      return undefined;
    }

    const leafPreimagePromise = committedDb.getLeafPreimage(MerkleTreeId.NULLIFIER_TREE, index);
    const siblingPathPromise = committedDb.getSiblingPath<typeof NULLIFIER_TREE_HEIGHT>(
      MerkleTreeId.NULLIFIER_TREE,
      BigInt(index),
    );

    const [leafPreimage, siblingPath] = await Promise.all([leafPreimagePromise, siblingPathPromise]);

    if (!leafPreimage) {
      return undefined;
    }

    return new NullifierMembershipWitness(BigInt(index), leafPreimage as NullifierLeafPreimage, siblingPath);
  }

  async getPublicDataTreeWitness(blockNumber: number, leafSlot: Fr): Promise<PublicDataWitness | undefined> {
    const committedDb = new MerkleTreeSnapshotOperationsFacade(this.trees, blockNumber);
    const lowLeafResult = await committedDb.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot.toBigInt());
    if (!lowLeafResult) {
      return undefined;
    } else {
      const preimage = (await committedDb.getLeafPreimage(
        MerkleTreeId.PUBLIC_DATA_TREE,
        lowLeafResult.index,
      )) as PublicDataTreeLeafPreimage;
      const path = await committedDb.getSiblingPath<typeof PUBLIC_DATA_TREE_HEIGHT>(
        MerkleTreeId.PUBLIC_DATA_TREE,
        lowLeafResult.index,
      );
      return new PublicDataWitness(lowLeafResult.index, preimage, path);
    }
  }

  getLowNullifierMembershipWitness(
    _blockNumber: number,
    _nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    throw new Error('Method not implemented.');
  }

  getHeader(_blockNumber: number): Promise<Header | undefined> {
    throw new Error('Method not implemented.');
  }

  getCompleteAddress(account: AztecAddress) {
    return Promise.resolve(this.txeDatabase.getAccount(account));
  }

  getAuthWitness(messageHash: Fr) {
    return this.txeDatabase.getAuthWitness(messageHash);
  }

  popCapsule(): Promise<Fr[]> {
    throw new Error('Method not implemented.');
  }

  getNotes(
    storageSlot: Fr,
    numSelects: number,
    selectByIndexes: number[],
    selectByOffsets: number[],
    selectByLengths: number[],
    selectValues: Fr[],
    selectComparators: number[],
    sortByIndexes: number[],
    sortByOffsets: number[],
    sortByLengths: number[],
    sortOrder: number[],
    limit: number,
    offset: number,
    _status: NoteStatus,
  ) {
    // Nullified pending notes are already removed from the list.
    const pendingNotes = this.noteCache.getNotes(this.contractAddress, storageSlot);

    const notes = pickNotes<NoteData>(pendingNotes, {
      selects: selectByIndexes.slice(0, numSelects).map((index, i) => ({
        selector: { index, offset: selectByOffsets[i], length: selectByLengths[i] },
        value: selectValues[i],
        comparator: selectComparators[i],
      })),
      sorts: sortByIndexes.map((index, i) => ({
        selector: { index, offset: sortByOffsets[i], length: sortByLengths[i] },
        order: sortOrder[i],
      })),
      limit,
      offset,
    });

    this.logger.debug(
      `Returning ${notes.length} notes for ${this.contractAddress} at ${storageSlot}: ${notes
        .map(n => `${n.nonce.toString()}:[${n.note.items.map(i => i.toString()).join(',')}]`)
        .join(', ')}`,
    );

    return Promise.resolve(notes);
  }

  notifyCreatedNote(storageSlot: Fr, noteTypeId: NoteSelector, noteItems: Fr[], innerNoteHash: Fr, counter: number) {
    const note = new Note(noteItems);
    this.noteCache.addNewNote(
      {
        contractAddress: this.contractAddress,
        storageSlot,
        nonce: Fr.ZERO, // Nonce cannot be known during private execution.
        note,
        siloedNullifier: undefined, // Siloed nullifier cannot be known for newly created note.
        innerNoteHash,
      },
      counter,
    );
    return Promise.resolve();
  }

  notifyNullifiedNote(innerNullifier: Fr, innerNoteHash: Fr, _counter: number) {
    this.noteCache.nullifyNote(this.contractAddress, innerNullifier, innerNoteHash);
    return Promise.resolve();
  }

  async checkNullifierExists(innerNullifier: Fr): Promise<boolean> {
    const nullifier = siloNullifier(this.contractAddress, innerNullifier!);
    const db = this.trees.asLatest();
    const index = await db.findLeafIndex(MerkleTreeId.NULLIFIER_TREE, nullifier.toBuffer());
    return index !== undefined;
  }

  getL1ToL2MembershipWitness(
    _contractAddress: AztecAddress,
    _messageHash: Fr,
    _secret: Fr,
  ): Promise<MessageLoadOracleInputs<16>> {
    throw new Error('Method not implemented.');
  }

  async avmOpcodeStorageRead(slot: Fr, length: Fr) {
    const db = this.trees.asLatest();

    const result = [];

    for (let i = 0; i < length.toNumber(); i++) {
      const leafSlot = computePublicDataTreeLeafSlot(this.contractAddress, slot.add(new Fr(i))).toBigInt();

      const lowLeafResult = await db.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot);
      if (!lowLeafResult || !lowLeafResult.alreadyPresent) {
        result.push(Fr.ZERO);
        continue;
      }

      const preimage = (await db.getLeafPreimage(
        MerkleTreeId.PUBLIC_DATA_TREE,
        lowLeafResult.index,
      )) as PublicDataTreeLeafPreimage;

      result.push(preimage.value);
    }
    return result;
  }

  async storageRead(
    contractAddress: Fr,
    startStorageSlot: Fr,
    blockNumber: number,
    numberOfElements: number,
  ): Promise<Fr[]> {
    const db =
      blockNumber === (await this.getBlockNumber())
        ? this.trees.asLatest()
        : new MerkleTreeSnapshotOperationsFacade(this.trees, blockNumber);

    const values = [];
    for (let i = 0n; i < numberOfElements; i++) {
      const storageSlot = startStorageSlot.add(new Fr(i));
      const leafSlot = computePublicDataTreeLeafSlot(contractAddress, storageSlot).toBigInt();

      const lowLeafResult = await db.getPreviousValueIndex(MerkleTreeId.PUBLIC_DATA_TREE, leafSlot);

      let value = Fr.ZERO;
      if (lowLeafResult && lowLeafResult.alreadyPresent) {
        const preimage = (await db.getLeafPreimage(
          MerkleTreeId.PUBLIC_DATA_TREE,
          lowLeafResult.index,
        )) as PublicDataTreeLeafPreimage;
        value = preimage.value;
      }
      this.logger.debug(`Oracle storage read: slot=${storageSlot.toString()} value=${value}`);
      values.push(value);
    }
    return values;
  }

  async storageWrite(startStorageSlot: Fr, values: Fr[]): Promise<Fr[]> {
    const db = this.trees.asLatest();

    const publicDataWrites = values.map((value, i) => {
      const storageSlot = startStorageSlot.add(new Fr(i));
      this.logger.debug(`Oracle storage write: slot=${storageSlot.toString()} value=${value}`);
      return new PublicDataWrite(computePublicDataTreeLeafSlot(this.contractAddress, storageSlot), value);
    });
    await db.batchInsert(
      MerkleTreeId.PUBLIC_DATA_TREE,
      publicDataWrites.map(write => new PublicDataTreeLeaf(write.leafIndex, write.newValue).toBuffer()),
      PUBLIC_DATA_SUBTREE_HEIGHT,
    );
    return publicDataWrites.map(write => write.newValue);
  }

  emitEncryptedLog(_contractAddress: AztecAddress, _randomness: Fr, _encryptedNote: Buffer, _counter: number): void {
    return;
  }

  emitEncryptedNoteLog(_noteHashCounter: number, _encryptedNote: Buffer, _counter: number): void {
    return;
  }

  computeEncryptedNoteLog(
    contractAddress: AztecAddress,
    storageSlot: Fr,
    noteTypeId: NoteSelector,
    ovKeys: KeyValidationRequest,
    ivpkM: Point,
    preimage: Fr[],
  ): Buffer {
    const note = new Note(preimage);
    const l1NotePayload = new L1NotePayload(note, contractAddress, storageSlot, noteTypeId);
    const taggedNote = new TaggedLog(l1NotePayload);

    const ephSk = GrumpkinScalar.random();

    const recipient = AztecAddress.random();

    return taggedNote.encrypt(ephSk, recipient, ivpkM, ovKeys);
  }

  emitUnencryptedLog(_log: UnencryptedL2Log, _counter: number): void {
    return;
  }

  emitContractClassUnencryptedLog(_log: UnencryptedL2Log, _counter: number): Fr {
    throw new Error('Method not implemented.');
  }

  async callPrivateFunction(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ): Promise<PrivateCallStackItem> {
    this.logger.verbose(
      `Executing external function ${targetContractAddress}:${functionSelector}(${await this.getDebugFunctionName(
        targetContractAddress,
        functionSelector,
      )}) isStaticCall=${isStaticCall} isDelegateCall=${isDelegateCall}`,
    );

    // Store and modify env
    const currentContractAddress = AztecAddress.fromField(this.contractAddress);
    const currentMessageSender = AztecAddress.fromField(this.msgSender);
    const currentFunctionSelector = FunctionSelector.fromField(this.functionSelector.toField());
    this.setMsgSender(this.contractAddress);
    this.setContractAddress(targetContractAddress);
    this.setFunctionSelector(functionSelector);

    const artifact = await this.contractDataOracle.getFunctionArtifact(targetContractAddress, functionSelector);

    const acir = artifact.bytecode;
    const initialWitness = await this.getInitialWitness(
      artifact,
      argsHash,
      sideEffectCounter,
      isStaticCall,
      isDelegateCall,
    );
    const acvmCallback = new Oracle(this);
    const timer = new Timer();
    try {
      const acirExecutionResult = await acvm(acir, initialWitness, acvmCallback).catch((err: Error) => {
        const execError = new ExecutionError(
          err.message,
          {
            contractAddress: targetContractAddress,
            functionSelector,
          },
          extractCallStack(err, artifact.debug),
          { cause: err },
        );
        this.logger.debug(`Error executing private function ${targetContractAddress}:${functionSelector}`);
        throw createSimulationError(execError);
      });
      const duration = timer.ms();
      const returnWitness = witnessMapToFields(acirExecutionResult.returnWitness);
      const publicInputs = PrivateCircuitPublicInputs.fromFields(returnWitness);

      const initialWitnessSize = witnessMapToFields(initialWitness).length * Fr.SIZE_IN_BYTES;
      this.logger.debug(`Ran external function ${targetContractAddress.toString()}:${functionSelector}`, {
        circuitName: 'app-circuit',
        duration,
        eventName: 'circuit-witness-generation',
        inputSize: initialWitnessSize,
        outputSize: publicInputs.toBuffer().length,
        appCircuitName: 'noname',
      } satisfies CircuitWitnessGenerationStats);

      const callStackItem = new PrivateCallStackItem(
        targetContractAddress,
        new FunctionData(functionSelector, true),
        publicInputs,
      );
      // Apply side effects
      this.sideEffectsCounter = publicInputs.endSideEffectCounter.toNumber();

      await this.addNullifiers(
        targetContractAddress,
        publicInputs.nullifiers.filter(nullifier => !nullifier.isEmpty()).map(nullifier => nullifier.value),
      );

      await this.addNoteHashes(
        targetContractAddress,
        publicInputs.noteHashes.filter(noteHash => !noteHash.isEmpty()).map(noteHash => noteHash.value),
      );

      return callStackItem;
    } finally {
      this.setContractAddress(currentContractAddress);
      this.setMsgSender(currentMessageSender);
      this.setFunctionSelector(currentFunctionSelector);
    }
  }

  async getInitialWitness(
    abi: FunctionAbi,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    const argumentsSize = countArgumentsSize(abi);

    const args = this.packedValuesCache.unpack(argsHash);

    if (args.length !== argumentsSize) {
      throw new Error('Invalid arguments size');
    }

    const privateContextInputs = await this.getPrivateContextInputs(
      this.blockNumber - 1,
      sideEffectCounter,
      isStaticCall,
      isDelegateCall,
    );

    const fields = [...privateContextInputs.toFields(), ...args];

    return toACVMWitness(0, fields);
  }

  public async getDebugFunctionName(address: AztecAddress, selector: FunctionSelector): Promise<string | undefined> {
    const instance = await this.contractDataOracle.getContractInstance(address);
    if (!instance) {
      return undefined;
    }
    const artifact = await this.contractDataOracle.getContractArtifact(instance!.contractClassId);
    if (!artifact) {
      return undefined;
    }

    const f = artifact.functions.find(f =>
      FunctionSelector.fromNameAndParameters(f.name, f.parameters).equals(selector),
    );
    if (!f) {
      return undefined;
    }

    return `${artifact.name}:${f.name}`;
  }

  async executePublicFunction(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    args: Fr[],
    callContext: CallContext,
  ) {
    const header = Header.empty();
    header.state = await this.trees.getStateReference(true);
    header.globalVariables.blockNumber = new Fr(await this.getBlockNumber());
    header.state.partial.nullifierTree.root = Fr.fromBuffer(
      (await this.trees.getTreeInfo(MerkleTreeId.NULLIFIER_TREE, true)).root,
    );
    header.state.partial.noteHashTree.root = Fr.fromBuffer(
      (await this.trees.getTreeInfo(MerkleTreeId.NOTE_HASH_TREE, true)).root,
    );
    header.state.partial.publicDataTree.root = Fr.fromBuffer(
      (await this.trees.getTreeInfo(MerkleTreeId.PUBLIC_DATA_TREE, true)).root,
    );
    header.state.l1ToL2MessageTree.root = Fr.fromBuffer(
      (await this.trees.getTreeInfo(MerkleTreeId.L1_TO_L2_MESSAGE_TREE, true)).root,
    );
    const executor = new PublicExecutor(
      new TXEPublicStateDB(this),
      new ContractsDataSourcePublicDB(new TXEPublicContractDataSource(this)),
      new WorldStateDB(this.trees.asLatest()),
      header,
    );
    const execution = {
      contractAddress: targetContractAddress,
      functionSelector,
      args,
      callContext,
    };

    return executor.simulate(
      execution,
      GlobalVariables.empty(),
      Gas.test(),
      TxContext.empty(),
      /* pendingNullifiers */ [],
      /* transactionFee */ Fr.ONE,
    );
  }

  async avmOpcodeCall(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    args: Fr[],
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ) {
    // Store and modify env
    const currentContractAddress = AztecAddress.fromField(this.contractAddress);
    const currentMessageSender = AztecAddress.fromField(this.msgSender);
    const currentFunctionSelector = FunctionSelector.fromField(this.functionSelector.toField());
    this.setMsgSender(this.contractAddress);
    this.setContractAddress(targetContractAddress);
    this.setFunctionSelector(functionSelector);

    const callContext = CallContext.empty();
    callContext.msgSender = this.msgSender;
    callContext.functionSelector = this.functionSelector;
    callContext.storageContractAddress = targetContractAddress;
    callContext.isStaticCall = isStaticCall;
    callContext.isDelegateCall = isDelegateCall;

    const executionResult = await this.executePublicFunction(
      targetContractAddress,
      functionSelector,
      args,
      callContext,
    );

    // Apply side effects
    if (!executionResult.reverted) {
      this.sideEffectsCounter += executionResult.endSideEffectCounter.toNumber();
    }
    this.setContractAddress(currentContractAddress);
    this.setMsgSender(currentMessageSender);
    this.setFunctionSelector(currentFunctionSelector);

    return executionResult;
  }

  async callPublicFunction(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ): Promise<Fr[]> {
    // Store and modify env
    const currentContractAddress = AztecAddress.fromField(this.contractAddress);
    const currentMessageSender = AztecAddress.fromField(this.msgSender);
    const currentFunctionSelector = FunctionSelector.fromField(this.functionSelector.toField());
    this.setMsgSender(this.contractAddress);
    this.setContractAddress(targetContractAddress);
    this.setFunctionSelector(functionSelector);

    const callContext = CallContext.empty();
    callContext.msgSender = this.msgSender;
    callContext.functionSelector = this.functionSelector;
    callContext.storageContractAddress = targetContractAddress;
    callContext.isStaticCall = isStaticCall;
    callContext.isDelegateCall = isDelegateCall;

    const args = this.packedValuesCache.unpack(argsHash);

    const executionResult = await this.executePublicFunction(
      targetContractAddress,
      functionSelector,
      args,
      callContext,
    );

    // Apply side effects
    this.sideEffectsCounter = executionResult.endSideEffectCounter.toNumber();
    this.setContractAddress(currentContractAddress);
    this.setMsgSender(currentMessageSender);
    this.setFunctionSelector(currentFunctionSelector);

    return executionResult.returnValues;
  }

  async enqueuePublicFunctionCall(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ): Promise<PublicCallRequest> {
    // Store and modify env
    const currentContractAddress = AztecAddress.fromField(this.contractAddress);
    const currentMessageSender = AztecAddress.fromField(this.msgSender);
    const currentFunctionSelector = FunctionSelector.fromField(this.functionSelector.toField());
    this.setMsgSender(this.contractAddress);
    this.setContractAddress(targetContractAddress);
    this.setFunctionSelector(functionSelector);

    const callContext = CallContext.empty();
    callContext.msgSender = this.msgSender;
    callContext.functionSelector = this.functionSelector;
    callContext.storageContractAddress = targetContractAddress;
    callContext.isStaticCall = isStaticCall;
    callContext.isDelegateCall = isDelegateCall;

    const args = this.packedValuesCache.unpack(argsHash);

    const executionResult = await this.executePublicFunction(
      targetContractAddress,
      functionSelector,
      args,
      callContext,
    );

    // Apply side effects
    this.sideEffectsCounter += executionResult.endSideEffectCounter.toNumber();
    this.setContractAddress(currentContractAddress);
    this.setMsgSender(currentMessageSender);
    this.setFunctionSelector(currentFunctionSelector);

    const parentCallContext = CallContext.empty();
    parentCallContext.msgSender = currentMessageSender;
    parentCallContext.functionSelector = currentFunctionSelector;
    parentCallContext.storageContractAddress = currentContractAddress;
    parentCallContext.isStaticCall = isStaticCall;
    parentCallContext.isDelegateCall = isDelegateCall;

    return PublicCallRequest.from({
      parentCallContext,
      contractAddress: targetContractAddress,
      functionSelector,
      callContext,
      sideEffectCounter,
      args,
    });
  }

  setPublicTeardownFunctionCall(
    targetContractAddress: AztecAddress,
    functionSelector: FunctionSelector,
    argsHash: Fr,
    sideEffectCounter: number,
    isStaticCall: boolean,
    isDelegateCall: boolean,
  ): Promise<PublicCallRequest> {
    // Definitely not right, in that the teardown should always be last.
    // But useful for executing flows.
    return this.enqueuePublicFunctionCall(
      targetContractAddress,
      functionSelector,
      argsHash,
      sideEffectCounter,
      isStaticCall,
      isDelegateCall,
    );
  }

  aes128Encrypt(input: Buffer, initializationVector: Buffer, key: Buffer): Buffer {
    const aes128 = new Aes128();
    return aes128.encryptBufferCBC(input, initializationVector, key);
  }

  debugLog(message: string, fields: Fr[]): void {
    this.logger.verbose(`debug_log ${applyStringFormatting(message, fields)}`);
  }

  emitEncryptedEventLog(
    _contractAddress: AztecAddress,
    _randomness: Fr,
    _encryptedEvent: Buffer,
    _counter: number,
  ): void {
    return;
  }

  computeEncryptedEventLog(
    _contractAddress: AztecAddress,
    _randomness: Fr,
    _eventTypeId: Fr,
    _ovKeys: KeyValidationRequest,
    _ivpkM: Point,
    _preimage: Fr[],
  ): Buffer {
    throw new Error('Method not implemented.');
  }
}
