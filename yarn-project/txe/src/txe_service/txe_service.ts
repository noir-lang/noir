import { SchnorrAccountContractArtifact } from '@aztec/accounts/schnorr';
import { L2Block, MerkleTreeId, PublicDataWrite } from '@aztec/circuit-types';
import {
  Fr,
  FunctionSelector,
  Header,
  KeyValidationRequest,
  PUBLIC_DATA_SUBTREE_HEIGHT,
  Point,
  PublicDataTreeLeaf,
  computePartialAddress,
  getContractInstanceFromDeployParams,
} from '@aztec/circuits.js';
import { computePublicDataTreeLeafSlot } from '@aztec/circuits.js/hash';
import { type ContractArtifact, NoteSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { type Logger } from '@aztec/foundation/log';
import { KeyStore } from '@aztec/key-store';
import { openTmpStore } from '@aztec/kv-store/utils';
import { ExecutionNoteCache, PackedValuesCache, type TypedOracle } from '@aztec/simulator';
import { MerkleTrees } from '@aztec/world-state';

import { TXE } from '../oracle/txe_oracle.js';
import {
  type ForeignCallArray,
  type ForeignCallSingle,
  fromArray,
  fromSingle,
  toArray,
  toForeignCallResult,
  toSingle,
} from '../util/encoding.js';
import { ExpectedFailureError } from '../util/expected_failure_error.js';
import { TXEDatabase } from '../util/txe_database.js';

export class TXEService {
  constructor(private logger: Logger, private typedOracle: TypedOracle) {}

  static async init(logger: Logger) {
    const store = openTmpStore(true);
    const trees = await MerkleTrees.new(store, logger);
    const packedValuesCache = new PackedValuesCache();
    const noteCache = new ExecutionNoteCache();
    const keyStore = new KeyStore(store);
    const txeDatabase = new TXEDatabase(store);
    logger.info(`TXE service initialized`);
    const txe = new TXE(logger, trees, packedValuesCache, noteCache, keyStore, txeDatabase);
    const service = new TXEService(logger, txe);
    await service.advanceBlocksBy(toSingle(new Fr(1n)));
    return service;
  }

  // Cheatcodes

  async getPrivateContextInputs(blockNumber: ForeignCallSingle) {
    const inputs = await (this.typedOracle as TXE).getPrivateContextInputs(fromSingle(blockNumber).toNumber());
    return toForeignCallResult(inputs.toFields().map(toSingle));
  }

  getPublicContextInputs() {
    const inputs = (this.typedOracle as TXE).getPublicContextInputs();
    return toForeignCallResult(inputs.toFields().map(toSingle));
  }

  async advanceBlocksBy(blocks: ForeignCallSingle) {
    const nBlocks = fromSingle(blocks).toNumber();
    this.logger.debug(`time traveling ${nBlocks} blocks`);
    const trees = (this.typedOracle as TXE).getTrees();
    const header = Header.empty();
    const l2Block = L2Block.empty();
    header.state = await trees.getStateReference(true);
    const blockNumber = await this.typedOracle.getBlockNumber();
    header.globalVariables.blockNumber = new Fr(blockNumber);
    l2Block.archive.root = Fr.fromBuffer((await trees.getTreeInfo(MerkleTreeId.ARCHIVE, true)).root);
    l2Block.header = header;
    for (let i = 0; i < nBlocks; i++) {
      const blockNumber = await this.typedOracle.getBlockNumber();
      header.globalVariables.blockNumber = new Fr(blockNumber);
      await trees.handleL2BlockAndMessages(l2Block, []);
      (this.typedOracle as TXE).setBlockNumber(blockNumber + 1);
    }
    return toForeignCallResult([]);
  }

  setContractAddress(address: ForeignCallSingle) {
    const typedAddress = AztecAddress.fromField(fromSingle(address));
    (this.typedOracle as TXE).setContractAddress(typedAddress);
    return toForeignCallResult([]);
  }

  deriveKeys(secret: ForeignCallSingle) {
    const keys = (this.typedOracle as TXE).deriveKeys(fromSingle(secret));
    return toForeignCallResult(keys.publicKeys.toFields().map(toSingle));
  }

  async deploy(
    artifact: ContractArtifact,
    initializer: ForeignCallArray,
    _length: ForeignCallSingle,
    args: ForeignCallArray,
    publicKeysHash: ForeignCallSingle,
  ) {
    const initializerStr = fromArray(initializer)
      .map(char => String.fromCharCode(char.toNumber()))
      .join('');
    const decodedArgs = fromArray(args);
    const publicKeysHashFr = fromSingle(publicKeysHash);
    this.logger.debug(
      `Deploy ${artifact.name} with initializer ${initializerStr}(${decodedArgs}) and public keys hash ${publicKeysHashFr}`,
    );

    const instance = getContractInstanceFromDeployParams(artifact, {
      constructorArgs: decodedArgs,
      skipArgsDecoding: true,
      salt: Fr.ONE,
      publicKeysHash: publicKeysHashFr,
      constructorArtifact: initializerStr ? initializerStr : undefined,
      deployer: AztecAddress.ZERO,
    });

    this.logger.debug(`Deployed ${artifact.name} at ${instance.address}`);
    await (this.typedOracle as TXE).addContractInstance(instance);
    await (this.typedOracle as TXE).addContractArtifact(artifact);
    return toForeignCallResult([
      toArray([
        instance.salt,
        instance.deployer,
        instance.contractClassId,
        instance.initializationHash,
        instance.publicKeysHash,
      ]),
    ]);
  }

  async directStorageWrite(
    contractAddress: ForeignCallSingle,
    startStorageSlot: ForeignCallSingle,
    values: ForeignCallArray,
  ) {
    const trees = (this.typedOracle as TXE).getTrees();
    const startStorageSlotFr = fromSingle(startStorageSlot);
    const valuesFr = fromArray(values);
    const contractAddressFr = fromSingle(contractAddress);
    const db = trees.asLatest();

    const publicDataWrites = valuesFr.map((value, i) => {
      const storageSlot = startStorageSlotFr.add(new Fr(i));
      this.logger.debug(`Oracle storage write: slot=${storageSlot.toString()} value=${value}`);
      return new PublicDataWrite(computePublicDataTreeLeafSlot(contractAddressFr, storageSlot), value);
    });
    await db.batchInsert(
      MerkleTreeId.PUBLIC_DATA_TREE,
      publicDataWrites.map(write => new PublicDataTreeLeaf(write.leafIndex, write.newValue).toBuffer()),
      PUBLIC_DATA_SUBTREE_HEIGHT,
    );
    return toForeignCallResult([toArray(publicDataWrites.map(write => write.newValue))]);
  }

  async createAccount() {
    const keyStore = (this.typedOracle as TXE).getKeyStore();
    const completeAddress = await keyStore.createAccount();
    const accountStore = (this.typedOracle as TXE).getTXEDatabase();
    await accountStore.setAccount(completeAddress.address, completeAddress);
    this.logger.debug(`Created account ${completeAddress.address}`);
    return toForeignCallResult([
      toSingle(completeAddress.address),
      ...completeAddress.publicKeys.toFields().map(toSingle),
    ]);
  }

  async addAccount(secret: ForeignCallSingle) {
    const keys = (this.typedOracle as TXE).deriveKeys(fromSingle(secret));
    const args = [keys.publicKeys.masterIncomingViewingPublicKey.x, keys.publicKeys.masterIncomingViewingPublicKey.y];
    const hash = keys.publicKeys.hash();
    const artifact = SchnorrAccountContractArtifact;
    const instance = getContractInstanceFromDeployParams(artifact, {
      constructorArgs: args,
      skipArgsDecoding: true,
      salt: Fr.ONE,
      publicKeysHash: hash,
      constructorArtifact: 'constructor',
      deployer: AztecAddress.ZERO,
    });

    this.logger.debug(`Deployed ${artifact.name} at ${instance.address}`);
    await (this.typedOracle as TXE).addContractInstance(instance);
    await (this.typedOracle as TXE).addContractArtifact(artifact);

    const keyStore = (this.typedOracle as TXE).getKeyStore();
    const completeAddress = await keyStore.addAccount(fromSingle(secret), computePartialAddress(instance));
    const accountStore = (this.typedOracle as TXE).getTXEDatabase();
    await accountStore.setAccount(completeAddress.address, completeAddress);
    this.logger.debug(`Created account ${completeAddress.address}`);
    return toForeignCallResult([
      toSingle(completeAddress.address),
      ...completeAddress.publicKeys.toFields().map(toSingle),
    ]);
  }

  setMsgSender(msgSender: ForeignCallSingle) {
    (this.typedOracle as TXE).setMsgSender(fromSingle(msgSender));
    return toForeignCallResult([]);
  }

  getMsgSender() {
    const msgSender = (this.typedOracle as TXE).getMsgSender();
    return toForeignCallResult([toSingle(msgSender)]);
  }

  getSideEffectsCounter() {
    const counter = (this.typedOracle as TXE).getSideEffectsCounter();
    return toForeignCallResult([toSingle(new Fr(counter))]);
  }

  async addAuthWitness(address: ForeignCallSingle, messageHash: ForeignCallSingle) {
    await (this.typedOracle as TXE).addAuthWitness(fromSingle(address), fromSingle(messageHash));
    return toForeignCallResult([]);
  }

  async assertPublicCallFails(
    address: ForeignCallSingle,
    functionSelector: ForeignCallSingle,
    _length: ForeignCallSingle,
    args: ForeignCallArray,
  ) {
    const parsedAddress = fromSingle(address);
    const parsedSelector = FunctionSelector.fromField(fromSingle(functionSelector));
    const result = await (this.typedOracle as TXE).avmOpcodeCall(
      parsedAddress,
      parsedSelector,
      fromArray(args),
      false,
      false,
    );
    if (!result.reverted) {
      throw new ExpectedFailureError('Public call did not revert');
    }

    return toForeignCallResult([]);
  }

  async assertPrivateCallFails(
    targetContractAddress: ForeignCallSingle,
    functionSelector: ForeignCallSingle,
    argsHash: ForeignCallSingle,
    sideEffectCounter: ForeignCallSingle,
    isStaticCall: ForeignCallSingle,
    isDelegateCall: ForeignCallSingle,
  ) {
    try {
      await this.typedOracle.callPrivateFunction(
        fromSingle(targetContractAddress),
        FunctionSelector.fromField(fromSingle(functionSelector)),
        fromSingle(argsHash),
        fromSingle(sideEffectCounter).toNumber(),
        fromSingle(isStaticCall).toBool(),
        fromSingle(isDelegateCall).toBool(),
      );
      throw new ExpectedFailureError('Private call did not fail');
    } catch (e) {
      if (e instanceof ExpectedFailureError) {
        throw e;
      }
    }
    return toForeignCallResult([]);
  }

  setFunctionSelector(functionSelector: ForeignCallSingle) {
    (this.typedOracle as TXE).setFunctionSelector(FunctionSelector.fromField(fromSingle(functionSelector)));
    return toForeignCallResult([]);
  }

  getFunctionSelector() {
    const functionSelector = (this.typedOracle as TXE).getFunctionSelector();
    return toForeignCallResult([toSingle(functionSelector.toField())]);
  }

  // PXE oracles

  getRandomField() {
    return toForeignCallResult([toSingle(this.typedOracle.getRandomField())]);
  }

  async getContractAddress() {
    const contractAddress = await this.typedOracle.getContractAddress();
    return toForeignCallResult([toSingle(contractAddress.toField())]);
  }

  async getBlockNumber() {
    const blockNumber = await this.typedOracle.getBlockNumber();
    return toForeignCallResult([toSingle(new Fr(blockNumber))]);
  }

  async avmOpcodeAddress() {
    const contractAddress = await this.typedOracle.getContractAddress();
    return toForeignCallResult([toSingle(contractAddress.toField())]);
  }

  async avmOpcodeBlockNumber() {
    const blockNumber = await this.typedOracle.getBlockNumber();
    return toForeignCallResult([toSingle(new Fr(blockNumber))]);
  }

  avmOpcodeFunctionSelector() {
    const functionSelector = (this.typedOracle as TXE).getFunctionSelector();
    return toForeignCallResult([toSingle(functionSelector.toField())]);
  }

  async packArgumentsArray(args: ForeignCallArray) {
    const packed = await this.typedOracle.packArgumentsArray(fromArray(args));
    return toForeignCallResult([toSingle(packed)]);
  }

  async packArguments(_length: ForeignCallSingle, values: ForeignCallArray) {
    const packed = await this.typedOracle.packArgumentsArray(fromArray(values));
    return toForeignCallResult([toSingle(packed)]);
  }

  // Since the argument is a slice, noir automatically adds a length field to oracle call.
  async packReturns(_length: ForeignCallSingle, values: ForeignCallArray) {
    const packed = await this.typedOracle.packReturns(fromArray(values));
    return toForeignCallResult([toSingle(packed)]);
  }

  async unpackReturns(returnsHash: ForeignCallSingle) {
    const unpacked = await this.typedOracle.unpackReturns(fromSingle(returnsHash));
    return toForeignCallResult([toArray(unpacked)]);
  }

  // Since the argument is a slice, noir automatically adds a length field to oracle call.
  debugLog(message: ForeignCallArray, _length: ForeignCallSingle, fields: ForeignCallArray) {
    const messageStr = fromArray(message)
      .map(field => String.fromCharCode(field.toNumber()))
      .join('');
    const fieldsFr = fromArray(fields);
    this.typedOracle.debugLog(messageStr, fieldsFr);
    return toForeignCallResult([]);
  }

  async storageRead(
    contractAddress: ForeignCallSingle,
    startStorageSlot: ForeignCallSingle,
    blockNumber: ForeignCallSingle,
    numberOfElements: ForeignCallSingle,
  ) {
    const values = await this.typedOracle.storageRead(
      fromSingle(contractAddress),
      fromSingle(startStorageSlot),
      fromSingle(blockNumber).toNumber(),
      fromSingle(numberOfElements).toNumber(),
    );
    return toForeignCallResult([toArray(values)]);
  }

  async storageWrite(startStorageSlot: ForeignCallSingle, values: ForeignCallArray) {
    const newValues = await this.typedOracle.storageWrite(fromSingle(startStorageSlot), fromArray(values));
    return toForeignCallResult([toArray(newValues)]);
  }

  async getPublicDataTreeWitness(blockNumber: ForeignCallSingle, leafSlot: ForeignCallSingle) {
    const parsedBlockNumber = fromSingle(blockNumber).toNumber();
    const parsedLeafSlot = fromSingle(leafSlot);

    const witness = await this.typedOracle.getPublicDataTreeWitness(parsedBlockNumber, parsedLeafSlot);
    if (!witness) {
      throw new Error(`Public data witness not found for slot ${parsedLeafSlot} at block ${parsedBlockNumber}.`);
    }
    return toForeignCallResult([toArray(witness.toFields())]);
  }

  async getSiblingPath(blockNumber: ForeignCallSingle, treeId: ForeignCallSingle, leafIndex: ForeignCallSingle) {
    const result = await this.typedOracle.getSiblingPath(
      fromSingle(blockNumber).toNumber(),
      fromSingle(treeId).toNumber(),
      fromSingle(leafIndex),
    );
    return toForeignCallResult([toArray(result)]);
  }

  async getNotes(
    storageSlot: ForeignCallSingle,
    numSelects: ForeignCallSingle,
    selectByIndexes: ForeignCallArray,
    selectByOffsets: ForeignCallArray,
    selectByLengths: ForeignCallArray,
    selectValues: ForeignCallArray,
    selectComparators: ForeignCallArray,
    sortByIndexes: ForeignCallArray,
    sortByOffsets: ForeignCallArray,
    sortByLengths: ForeignCallArray,
    sortOrder: ForeignCallArray,
    limit: ForeignCallSingle,
    offset: ForeignCallSingle,
    status: ForeignCallSingle,
    returnSize: ForeignCallSingle,
  ) {
    const noteDatas = await this.typedOracle.getNotes(
      fromSingle(storageSlot),
      fromSingle(numSelects).toNumber(),
      fromArray(selectByIndexes).map(fr => fr.toNumber()),
      fromArray(selectByOffsets).map(fr => fr.toNumber()),
      fromArray(selectByLengths).map(fr => fr.toNumber()),
      fromArray(selectValues),
      fromArray(selectComparators).map(fr => fr.toNumber()),
      fromArray(sortByIndexes).map(fr => fr.toNumber()),
      fromArray(sortByOffsets).map(fr => fr.toNumber()),
      fromArray(sortByLengths).map(fr => fr.toNumber()),
      fromArray(sortOrder).map(fr => fr.toNumber()),
      fromSingle(limit).toNumber(),
      fromSingle(offset).toNumber(),
      fromSingle(status).toNumber(),
    );
    const noteLength = noteDatas?.[0]?.note.items.length ?? 0;
    if (!noteDatas.every(({ note }) => noteLength === note.items.length)) {
      throw new Error('Notes should all be the same length.');
    }

    const contractAddress = noteDatas[0]?.contractAddress ?? Fr.ZERO;

    // Values indicates whether the note is settled or transient.
    const noteTypes = {
      isSettled: new Fr(0),
      isTransient: new Fr(1),
    };
    const flattenData = noteDatas.flatMap(({ nonce, note, index }) => [
      nonce,
      index === undefined ? noteTypes.isTransient : noteTypes.isSettled,
      ...note.items,
    ]);

    const returnFieldSize = fromSingle(returnSize).toNumber();
    const returnData = [noteDatas.length, contractAddress, ...flattenData].map(v => new Fr(v));
    if (returnData.length > returnFieldSize) {
      throw new Error(`Return data size too big. Maximum ${returnFieldSize} fields. Got ${flattenData.length}.`);
    }

    const paddedZeros = Array(returnFieldSize - returnData.length).fill(new Fr(0));
    return toForeignCallResult([toArray([...returnData, ...paddedZeros])]);
  }

  notifyCreatedNote(
    storageSlot: ForeignCallSingle,
    noteTypeId: ForeignCallSingle,
    note: ForeignCallArray,
    innerNoteHash: ForeignCallSingle,
    counter: ForeignCallSingle,
  ) {
    this.typedOracle.notifyCreatedNote(
      fromSingle(storageSlot),
      NoteSelector.fromField(fromSingle(noteTypeId)),
      fromArray(note),
      fromSingle(innerNoteHash),
      fromSingle(counter).toNumber(),
    );
    return toForeignCallResult([toSingle(new Fr(0))]);
  }

  async notifyNullifiedNote(
    innerNullifier: ForeignCallSingle,
    innerNoteHash: ForeignCallSingle,
    counter: ForeignCallSingle,
  ) {
    await this.typedOracle.notifyNullifiedNote(
      fromSingle(innerNullifier),
      fromSingle(innerNoteHash),
      fromSingle(counter).toNumber(),
    );
    return toForeignCallResult([toSingle(new Fr(0))]);
  }

  async checkNullifierExists(innerNullifier: ForeignCallSingle) {
    const exists = await this.typedOracle.checkNullifierExists(fromSingle(innerNullifier));
    return toForeignCallResult([toSingle(new Fr(exists))]);
  }

  async getContractInstance(address: ForeignCallSingle) {
    const instance = await this.typedOracle.getContractInstance(fromSingle(address));
    return toForeignCallResult([
      toArray([
        instance.salt,
        instance.deployer,
        instance.contractClassId,
        instance.initializationHash,
        instance.publicKeysHash,
      ]),
    ]);
  }

  async avmOpcodeGetContractInstance(address: ForeignCallSingle) {
    const instance = await this.typedOracle.getContractInstance(fromSingle(address));
    return toForeignCallResult([
      toArray([
        // AVM requires an extra boolean indicating the instance was found
        new Fr(1),
        instance.salt,
        instance.deployer,
        instance.contractClassId,
        instance.initializationHash,
        instance.publicKeysHash,
      ]),
    ]);
  }

  avmOpcodeSender() {
    const sender = (this.typedOracle as TXE).getMsgSender();
    return toForeignCallResult([toSingle(sender)]);
  }

  async avmOpcodeEmitNullifier(nullifier: ForeignCallSingle) {
    await (this.typedOracle as TXE).avmOpcodeEmitNullifier(fromSingle(nullifier));
    return toForeignCallResult([]);
  }

  async avmOpcodeEmitNoteHash(innerNoteHash: ForeignCallSingle) {
    await (this.typedOracle as TXE).avmOpcodeEmitNoteHash(fromSingle(innerNoteHash));
    return toForeignCallResult([]);
  }

  async avmOpcodeNullifierExists(innerNullifier: ForeignCallSingle, targetAddress: ForeignCallSingle) {
    const exists = await (this.typedOracle as TXE).avmOpcodeNullifierExists(
      fromSingle(innerNullifier),
      AztecAddress.fromField(fromSingle(targetAddress)),
    );
    return toForeignCallResult([toSingle(new Fr(exists))]);
  }

  async avmOpcodeCall(
    _gas: ForeignCallArray,
    address: ForeignCallSingle,
    _length: ForeignCallSingle,
    args: ForeignCallArray,
    functionSelector: ForeignCallSingle,
  ) {
    const result = await (this.typedOracle as TXE).avmOpcodeCall(
      fromSingle(address),
      FunctionSelector.fromField(fromSingle(functionSelector)),
      fromArray(args),
      /* isStaticCall */ false,
      /* isDelegateCall */ false,
    );

    return toForeignCallResult([toArray(result.returnValues), toSingle(new Fr(1))]);
  }

  async avmOpcodeStaticCall(
    _gas: ForeignCallArray,
    address: ForeignCallSingle,
    _length: ForeignCallSingle,
    args: ForeignCallArray,
    functionSelector: ForeignCallSingle,
  ) {
    const result = await (this.typedOracle as TXE).avmOpcodeCall(
      fromSingle(address),
      FunctionSelector.fromField(fromSingle(functionSelector)),
      fromArray(args),
      /* isStaticCall */ true,
      /* isDelegateCall */ false,
    );

    return toForeignCallResult([toArray(result.returnValues), toSingle(new Fr(1))]);
  }

  async avmOpcodeStorageRead(slot: ForeignCallSingle, length: ForeignCallSingle) {
    const values = await (this.typedOracle as TXE).avmOpcodeStorageRead(fromSingle(slot), fromSingle(length));
    return toForeignCallResult([toArray(values)]);
  }

  async avmOpcodeStorageWrite(startStorageSlot: ForeignCallSingle, values: ForeignCallArray) {
    await this.typedOracle.storageWrite(fromSingle(startStorageSlot), fromArray(values));
    return toForeignCallResult([]);
  }

  async getPublicKeysAndPartialAddress(address: ForeignCallSingle) {
    const parsedAddress = AztecAddress.fromField(fromSingle(address));
    const { publicKeys, partialAddress } = await this.typedOracle.getCompleteAddress(parsedAddress);
    return toForeignCallResult([toArray([...publicKeys.toFields(), partialAddress])]);
  }

  async getKeyValidationRequest(pkMHash: ForeignCallSingle) {
    const keyValidationRequest = await this.typedOracle.getKeyValidationRequest(fromSingle(pkMHash));
    return toForeignCallResult([toArray(keyValidationRequest.toFields())]);
  }

  computeEncryptedNoteLog(
    contractAddress: ForeignCallSingle,
    storageSlot: ForeignCallSingle,
    noteTypeId: ForeignCallSingle,
    ovskApp: ForeignCallSingle,
    ovpkMX: ForeignCallSingle,
    ovpkMY: ForeignCallSingle,
    ovpkMIsInfinite: ForeignCallSingle,
    ivpkMX: ForeignCallSingle,
    ivpkMY: ForeignCallSingle,
    ivpkMIsInfinite: ForeignCallSingle,
    recipient: ForeignCallSingle,
    preimage: ForeignCallArray,
  ) {
    const ovpkM = new Point(fromSingle(ovpkMX), fromSingle(ovpkMY), !fromSingle(ovpkMIsInfinite).isZero());
    const ovKeys = new KeyValidationRequest(ovpkM, Fr.fromString(fromSingle(ovskApp).toString()));
    const ivpkM = new Point(fromSingle(ivpkMX), fromSingle(ivpkMY), !fromSingle(ivpkMIsInfinite).isZero());
    const encLog = this.typedOracle.computeEncryptedNoteLog(
      AztecAddress.fromString(fromSingle(contractAddress).toString()),
      Fr.fromString(fromSingle(storageSlot).toString()),
      NoteSelector.fromField(Fr.fromString(fromSingle(noteTypeId).toString())),
      ovKeys,
      ivpkM,
      AztecAddress.fromString(fromSingle(recipient).toString()),
      fromArray(preimage),
    );
    const bytes: Fr[] = [];

    encLog.forEach(v => {
      bytes.push(new Fr(v));
    });
    return toForeignCallResult([toArray(bytes)]);
  }

  emitEncryptedLog(
    _contractAddress: ForeignCallSingle,
    _randomness: ForeignCallSingle,
    _encryptedLog: ForeignCallSingle,
    _counter: ForeignCallSingle,
  ) {
    return toForeignCallResult([]);
  }

  emitEncryptedNoteLog(
    _noteHashCounter: ForeignCallSingle,
    _encryptedNote: ForeignCallArray,
    _counter: ForeignCallSingle,
  ) {
    return toForeignCallResult([]);
  }

  emitEncryptedEventLog(_contractAddress: AztecAddress, _randomness: Fr, _encryptedEvent: Buffer, _counter: number) {
    return toForeignCallResult([]);
  }

  async callPrivateFunction(
    targetContractAddress: ForeignCallSingle,
    functionSelector: ForeignCallSingle,
    argsHash: ForeignCallSingle,
    sideEffectCounter: ForeignCallSingle,
    isStaticCall: ForeignCallSingle,
    isDelegateCall: ForeignCallSingle,
  ) {
    const result = await this.typedOracle.callPrivateFunction(
      fromSingle(targetContractAddress),
      FunctionSelector.fromField(fromSingle(functionSelector)),
      fromSingle(argsHash),
      fromSingle(sideEffectCounter).toNumber(),
      fromSingle(isStaticCall).toBool(),
      fromSingle(isDelegateCall).toBool(),
    );
    return toForeignCallResult([toArray([result.endSideEffectCounter, result.returnsHash])]);
  }

  async getNullifierMembershipWitness(blockNumber: ForeignCallSingle, nullifier: ForeignCallSingle) {
    const parsedBlockNumber = fromSingle(blockNumber).toNumber();
    const witness = await this.typedOracle.getNullifierMembershipWitness(parsedBlockNumber, fromSingle(nullifier));
    if (!witness) {
      throw new Error(`Nullifier membership witness not found at block ${parsedBlockNumber}.`);
    }
    return toForeignCallResult([toArray(witness.toFields())]);
  }

  async getAuthWitness(messageHash: ForeignCallSingle) {
    const parsedMessageHash = fromSingle(messageHash);
    const authWitness = await this.typedOracle.getAuthWitness(parsedMessageHash);
    if (!authWitness) {
      throw new Error(`Auth witness not found for message hash ${parsedMessageHash}.`);
    }
    return toForeignCallResult([toArray(authWitness)]);
  }

  async enqueuePublicFunctionCall(
    targetContractAddress: ForeignCallSingle,
    functionSelector: ForeignCallSingle,
    argsHash: ForeignCallSingle,
    sideEffectCounter: ForeignCallSingle,
    isStaticCall: ForeignCallSingle,
    isDelegateCall: ForeignCallSingle,
  ) {
    await this.typedOracle.enqueuePublicFunctionCall(
      fromSingle(targetContractAddress),
      FunctionSelector.fromField(fromSingle(functionSelector)),
      fromSingle(argsHash),
      fromSingle(sideEffectCounter).toNumber(),
      fromSingle(isStaticCall).toBool(),
      fromSingle(isDelegateCall).toBool(),
    );
    return toForeignCallResult([]);
  }

  public async setPublicTeardownFunctionCall(
    targetContractAddress: ForeignCallSingle,
    functionSelector: ForeignCallSingle,
    argsHash: ForeignCallSingle,
    sideEffectCounter: ForeignCallSingle,
    isStaticCall: ForeignCallSingle,
    isDelegateCall: ForeignCallSingle,
  ) {
    await this.typedOracle.setPublicTeardownFunctionCall(
      fromSingle(targetContractAddress),
      FunctionSelector.fromField(fromSingle(functionSelector)),
      fromSingle(argsHash),
      fromSingle(sideEffectCounter).toNumber(),
      fromSingle(isStaticCall).toBool(),
      fromSingle(isDelegateCall).toBool(),
    );
    return toForeignCallResult([]);
  }

  async getChainId() {
    return toForeignCallResult([toSingle(await this.typedOracle.getChainId())]);
  }

  async getVersion() {
    return toForeignCallResult([toSingle(await this.typedOracle.getVersion())]);
  }

  async addNullifiers(contractAddress: ForeignCallSingle, _length: ForeignCallSingle, nullifiers: ForeignCallArray) {
    await (this.typedOracle as TXE).addNullifiers(fromSingle(contractAddress), fromArray(nullifiers));
    return toForeignCallResult([]);
  }

  async addNoteHashes(contractAddress: ForeignCallSingle, _length: ForeignCallSingle, noteHashes: ForeignCallArray) {
    await (this.typedOracle as TXE).addNoteHashes(fromSingle(contractAddress), fromArray(noteHashes));
    return toForeignCallResult([]);
  }
}
