import { MerkleTreeId, UnencryptedL2Log } from '@aztec/circuit-types';
import { KeyValidationRequest } from '@aztec/circuits.js';
import { FunctionSelector, NoteSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';

import { type ACVMField } from '../acvm_types.js';
import { frToBoolean, frToNumber, fromACVMField } from '../deserialize.js';
import { toACVMField } from '../serialize.js';
import { type TypedOracle } from './typed_oracle.js';

/**
 * A data source that has all the apis required by Aztec.nr.
 */
export class Oracle {
  constructor(private typedOracle: TypedOracle) {}

  getRandomField(): ACVMField {
    const val = this.typedOracle.getRandomField();
    return toACVMField(val);
  }

  async packArgumentsArray(args: ACVMField[]): Promise<ACVMField> {
    const packed = await this.typedOracle.packArgumentsArray(args.map(fromACVMField));
    return toACVMField(packed);
  }

  async packArguments(_length: ACVMField[], values: ACVMField[]): Promise<ACVMField> {
    const packed = await this.typedOracle.packArgumentsArray(values.map(fromACVMField));
    return toACVMField(packed);
  }

  // Since the argument is a slice, noir automatically adds a length field to oracle call.
  async packReturns(_length: ACVMField[], values: ACVMField[]): Promise<ACVMField> {
    const packed = await this.typedOracle.packReturns(values.map(fromACVMField));
    return toACVMField(packed);
  }

  async unpackReturns([returnsHash]: ACVMField[]): Promise<ACVMField[]> {
    const unpacked = await this.typedOracle.unpackReturns(fromACVMField(returnsHash));
    return unpacked.map(toACVMField);
  }

  async getBlockNumber(): Promise<ACVMField> {
    return toACVMField(await this.typedOracle.getBlockNumber());
  }

  async getContractAddress(): Promise<ACVMField> {
    return toACVMField(await this.typedOracle.getContractAddress());
  }

  async getVersion(): Promise<ACVMField> {
    return toACVMField(await this.typedOracle.getVersion());
  }

  async getChainId(): Promise<ACVMField> {
    return toACVMField(await this.typedOracle.getChainId());
  }

  async getKeyValidationRequest([pkMHash]: ACVMField[]): Promise<ACVMField[]> {
    const { pkM, skApp } = await this.typedOracle.getKeyValidationRequest(fromACVMField(pkMHash));

    return [toACVMField(pkM.x), toACVMField(pkM.y), toACVMField(pkM.isInfinite), toACVMField(skApp)];
  }

  async getContractInstance([address]: ACVMField[]) {
    const instance = await this.typedOracle.getContractInstance(AztecAddress.fromField(fromACVMField(address)));

    return [
      instance.salt,
      instance.deployer,
      instance.contractClassId,
      instance.initializationHash,
      instance.publicKeysHash,
    ].map(toACVMField);
  }

  async getMembershipWitness(
    [blockNumber]: ACVMField[],
    [treeId]: ACVMField[],
    [leafValue]: ACVMField[],
  ): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));
    const parsedTreeId = frToNumber(fromACVMField(treeId));
    const parsedLeafValue = fromACVMField(leafValue);

    const witness = await this.typedOracle.getMembershipWitness(parsedBlockNumber, parsedTreeId, parsedLeafValue);
    if (!witness) {
      throw new Error(
        `Leaf ${leafValue} not found in the tree ${MerkleTreeId[parsedTreeId]} at block ${parsedBlockNumber}.`,
      );
    }
    return witness.map(toACVMField);
  }

  async getSiblingPath(
    [blockNumber]: ACVMField[],
    [treeId]: ACVMField[],
    [leafIndex]: ACVMField[],
  ): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));
    const parsedTreeId = frToNumber(fromACVMField(treeId));
    const parsedLeafIndex = fromACVMField(leafIndex);

    const path = await this.typedOracle.getSiblingPath(parsedBlockNumber, parsedTreeId, parsedLeafIndex);
    return path.map(toACVMField);
  }

  async getNullifierMembershipWitness(
    [blockNumber]: ACVMField[],
    [nullifier]: ACVMField[], // nullifier, we try to find the witness for (to prove inclusion)
  ): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));
    const parsedNullifier = fromACVMField(nullifier);

    const witness = await this.typedOracle.getNullifierMembershipWitness(parsedBlockNumber, parsedNullifier);
    if (!witness) {
      throw new Error(`Nullifier witness not found for nullifier ${parsedNullifier} at block ${parsedBlockNumber}.`);
    }
    return witness.toFields().map(toACVMField);
  }

  async getLowNullifierMembershipWitness(
    [blockNumber]: ACVMField[],
    [nullifier]: ACVMField[], // nullifier, we try to find the low nullifier witness for (to prove non-inclusion)
  ): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));
    const parsedNullifier = fromACVMField(nullifier);

    const witness = await this.typedOracle.getLowNullifierMembershipWitness(parsedBlockNumber, parsedNullifier);
    if (!witness) {
      throw new Error(
        `Low nullifier witness not found for nullifier ${parsedNullifier} at block ${parsedBlockNumber}.`,
      );
    }
    return witness.toFields().map(toACVMField);
  }

  async getPublicDataTreeWitness([blockNumber]: ACVMField[], [leafSlot]: ACVMField[]): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));
    const parsedLeafSlot = fromACVMField(leafSlot);

    const witness = await this.typedOracle.getPublicDataTreeWitness(parsedBlockNumber, parsedLeafSlot);
    if (!witness) {
      throw new Error(`Public data witness not found for slot ${parsedLeafSlot} at block ${parsedBlockNumber}.`);
    }
    return witness.toFields().map(toACVMField);
  }

  async getHeader([blockNumber]: ACVMField[]): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));

    const header = await this.typedOracle.getHeader(parsedBlockNumber);
    if (!header) {
      throw new Error(`Block header not found for block ${parsedBlockNumber}.`);
    }
    return header.toFields().map(toACVMField);
  }

  async getAuthWitness([messageHash]: ACVMField[]): Promise<ACVMField[]> {
    const messageHashField = fromACVMField(messageHash);
    const witness = await this.typedOracle.getAuthWitness(messageHashField);
    if (!witness) {
      throw new Error(`Authorization not found for message hash ${messageHashField}`);
    }
    return witness.map(toACVMField);
  }

  async popCapsule(): Promise<ACVMField[]> {
    const capsule = await this.typedOracle.popCapsule();
    if (!capsule) {
      throw new Error(`No capsules available`);
    }
    return capsule.map(toACVMField);
  }

  async getPublicKeysAndPartialAddress([address]: ACVMField[]): Promise<ACVMField[]> {
    const parsedAddress = AztecAddress.fromField(fromACVMField(address));
    const { publicKeys, partialAddress } = await this.typedOracle.getCompleteAddress(parsedAddress);

    return [...publicKeys.toFields(), partialAddress].map(toACVMField);
  }

  async getNotes(
    [storageSlot]: ACVMField[],
    [numSelects]: ACVMField[],
    selectByIndexes: ACVMField[],
    selectByOffsets: ACVMField[],
    selectByLengths: ACVMField[],
    selectValues: ACVMField[],
    selectComparators: ACVMField[],
    sortByIndexes: ACVMField[],
    sortByOffsets: ACVMField[],
    sortByLengths: ACVMField[],
    sortOrder: ACVMField[],
    [limit]: ACVMField[],
    [offset]: ACVMField[],
    [status]: ACVMField[],
    [returnSize]: ACVMField[],
  ): Promise<ACVMField[]> {
    const noteDatas = await this.typedOracle.getNotes(
      fromACVMField(storageSlot),
      +numSelects,
      selectByIndexes.map(s => +s),
      selectByOffsets.map(s => +s),
      selectByLengths.map(s => +s),
      selectValues.map(fromACVMField),
      selectComparators.map(s => +s),
      sortByIndexes.map(s => +s),
      sortByOffsets.map(s => +s),
      sortByLengths.map(s => +s),
      sortOrder.map(s => +s),
      +limit,
      +offset,
      +status,
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

    const returnFieldSize = +returnSize;
    const returnData = [noteDatas.length, contractAddress, ...flattenData].map(v => toACVMField(v));
    if (returnData.length > returnFieldSize) {
      throw new Error(`Return data size too big. Maximum ${returnFieldSize} fields. Got ${flattenData.length}.`);
    }

    const paddedZeros = Array(returnFieldSize - returnData.length).fill(toACVMField(0));
    return returnData.concat(paddedZeros);
  }

  notifyCreatedNote(
    [storageSlot]: ACVMField[],
    [noteTypeId]: ACVMField[],
    note: ACVMField[],
    [innerNoteHash]: ACVMField[],
    [counter]: ACVMField[],
  ): ACVMField {
    this.typedOracle.notifyCreatedNote(
      fromACVMField(storageSlot),
      NoteSelector.fromField(fromACVMField(noteTypeId)),
      note.map(fromACVMField),
      fromACVMField(innerNoteHash),
      +counter,
    );
    return toACVMField(0);
  }

  async notifyNullifiedNote(
    [innerNullifier]: ACVMField[],
    [innerNoteHash]: ACVMField[],
    [counter]: ACVMField[],
  ): Promise<ACVMField> {
    await this.typedOracle.notifyNullifiedNote(fromACVMField(innerNullifier), fromACVMField(innerNoteHash), +counter);
    return toACVMField(0);
  }

  async checkNullifierExists([innerNullifier]: ACVMField[]): Promise<ACVMField> {
    const exists = await this.typedOracle.checkNullifierExists(fromACVMField(innerNullifier));
    return toACVMField(exists);
  }

  async getL1ToL2MembershipWitness(
    [contractAddress]: ACVMField[],
    [messageHash]: ACVMField[],
    [secret]: ACVMField[],
  ): Promise<ACVMField[]> {
    const message = await this.typedOracle.getL1ToL2MembershipWitness(
      AztecAddress.fromString(contractAddress),
      fromACVMField(messageHash),
      fromACVMField(secret),
    );
    return message.toFields().map(toACVMField);
  }

  async storageRead(
    [contractAddress]: ACVMField[],
    [startStorageSlot]: ACVMField[],
    [blockNumber]: ACVMField[],
    [numberOfElements]: ACVMField[],
  ): Promise<ACVMField[]> {
    const values = await this.typedOracle.storageRead(
      fromACVMField(contractAddress),
      fromACVMField(startStorageSlot),
      +blockNumber,
      +numberOfElements,
    );
    return values.map(toACVMField);
  }

  async storageWrite([startStorageSlot]: ACVMField[], values: ACVMField[]): Promise<ACVMField[]> {
    const newValues = await this.typedOracle.storageWrite(fromACVMField(startStorageSlot), values.map(fromACVMField));
    return newValues.map(toACVMField);
  }

  emitEncryptedEventLog(
    [contractAddress]: ACVMField[],
    [randomness]: ACVMField[],
    encryptedEvent: ACVMField[],
    [counter]: ACVMField[],
  ): void {
    // Convert each field to a number and then to a buffer (1 byte is stored in 1 field)
    const processedInput = Buffer.from(encryptedEvent.map(fromACVMField).map(f => f.toNumber()));
    this.typedOracle.emitEncryptedEventLog(
      AztecAddress.fromString(contractAddress),
      Fr.fromString(randomness),
      processedInput,
      +counter,
    );
  }

  emitEncryptedNoteLog([noteHashCounter]: ACVMField[], encryptedNote: ACVMField[], [counter]: ACVMField[]): void {
    // Convert each field to a number and then to a buffer (1 byte is stored in 1 field)
    const processedInput = Buffer.from(encryptedNote.map(fromACVMField).map(f => f.toNumber()));
    this.typedOracle.emitEncryptedNoteLog(+noteHashCounter, processedInput, +counter);
  }

  computeEncryptedEventLog(
    [contractAddress]: ACVMField[],
    [randomness]: ACVMField[],
    [eventTypeId]: ACVMField[],
    [ovskApp]: ACVMField[],
    [ovpkMX]: ACVMField[],
    [ovpkMY]: ACVMField[],
    [ovpkMIsInfinite]: ACVMField[],
    [ivpkMX]: ACVMField[],
    [ivpkMY]: ACVMField[],
    [ivpkMIsInfinite]: ACVMField[],
    [recipient]: ACVMField[],
    preimage: ACVMField[],
  ): ACVMField[] {
    const ovpkM = new Point(fromACVMField(ovpkMX), fromACVMField(ovpkMY), !fromACVMField(ovpkMIsInfinite).isZero());
    const ovKeys = new KeyValidationRequest(ovpkM, Fr.fromString(ovskApp));
    const ivpkM = new Point(fromACVMField(ivpkMX), fromACVMField(ivpkMY), !fromACVMField(ivpkMIsInfinite).isZero());
    const encLog = this.typedOracle.computeEncryptedEventLog(
      AztecAddress.fromString(contractAddress),
      Fr.fromString(randomness),
      Fr.fromString(eventTypeId),
      ovKeys,
      ivpkM,
      AztecAddress.fromString(recipient),
      preimage.map(fromACVMField),
    );
    const bytes: ACVMField[] = [];
    encLog.forEach(v => {
      bytes.push(toACVMField(v));
    });
    return bytes;
  }

  computeEncryptedNoteLog(
    [contractAddress]: ACVMField[],
    [storageSlot]: ACVMField[],
    [noteTypeId]: ACVMField[],
    [ovskApp]: ACVMField[],
    [ovpkMX]: ACVMField[],
    [ovpkMY]: ACVMField[],
    [ovpkMIsInfinite]: ACVMField[],
    [ivpkMX]: ACVMField[],
    [ivpkMY]: ACVMField[],
    [ivpkMIsInfinite]: ACVMField[],
    [recipient]: ACVMField[],
    preimage: ACVMField[],
  ): ACVMField[] {
    const ovpkM = new Point(fromACVMField(ovpkMX), fromACVMField(ovpkMY), !fromACVMField(ovpkMIsInfinite).isZero());
    const ovKeys = new KeyValidationRequest(ovpkM, Fr.fromString(ovskApp));
    const ivpkM = new Point(fromACVMField(ivpkMX), fromACVMField(ivpkMY), !fromACVMField(ivpkMIsInfinite).isZero());
    const encLog = this.typedOracle.computeEncryptedNoteLog(
      AztecAddress.fromString(contractAddress),
      Fr.fromString(storageSlot),
      NoteSelector.fromField(Fr.fromString(noteTypeId)),
      ovKeys,
      ivpkM,
      AztecAddress.fromString(recipient),
      preimage.map(fromACVMField),
    );
    const bytes: ACVMField[] = [];
    encLog.forEach(v => {
      bytes.push(toACVMField(v));
    });
    return bytes;
  }

  emitUnencryptedLog([contractAddress]: ACVMField[], message: ACVMField[], [counter]: ACVMField[]): ACVMField {
    const logPayload = Buffer.concat(message.map(fromACVMField).map(f => f.toBuffer()));
    const log = new UnencryptedL2Log(AztecAddress.fromString(contractAddress), logPayload);

    this.typedOracle.emitUnencryptedLog(log, +counter);
    return toACVMField(0);
  }

  emitContractClassUnencryptedLog(
    [contractAddress]: ACVMField[],
    message: ACVMField[],
    [counter]: ACVMField[],
  ): ACVMField {
    const logPayload = Buffer.concat(message.map(fromACVMField).map(f => f.toBuffer()));
    const log = new UnencryptedL2Log(AztecAddress.fromString(contractAddress), logPayload);

    const logHash = this.typedOracle.emitContractClassUnencryptedLog(log, +counter);
    return toACVMField(logHash);
  }

  debugLog(message: ACVMField[], _ignoredFieldsSize: ACVMField[], fields: ACVMField[]): void {
    const messageStr = message.map(acvmField => String.fromCharCode(fromACVMField(acvmField).toNumber())).join('');
    const fieldsFr = fields.map(fromACVMField);
    this.typedOracle.debugLog(messageStr, fieldsFr);
  }

  async callPrivateFunction(
    [contractAddress]: ACVMField[],
    [functionSelector]: ACVMField[],
    [argsHash]: ACVMField[],
    [sideEffectCounter]: ACVMField[],
    [isStaticCall]: ACVMField[],
    [isDelegateCall]: ACVMField[],
  ): Promise<ACVMField[]> {
    const { endSideEffectCounter, returnsHash } = await this.typedOracle.callPrivateFunction(
      AztecAddress.fromField(fromACVMField(contractAddress)),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
      frToNumber(fromACVMField(sideEffectCounter)),
      frToBoolean(fromACVMField(isStaticCall)),
      frToBoolean(fromACVMField(isDelegateCall)),
    );
    return [endSideEffectCounter, returnsHash].map(toACVMField);
  }

  async callPublicFunction(
    [contractAddress]: ACVMField[],
    [functionSelector]: ACVMField[],
    [argsHash]: ACVMField[],
    [sideEffectCounter]: ACVMField[],
    [isStaticCall]: ACVMField[],
    [isDelegateCall]: ACVMField[],
  ): Promise<ACVMField[]> {
    const returnValues = await this.typedOracle.callPublicFunction(
      AztecAddress.fromField(fromACVMField(contractAddress)),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
      frToNumber(fromACVMField(sideEffectCounter)),
      frToBoolean(fromACVMField(isStaticCall)),
      frToBoolean(fromACVMField(isDelegateCall)),
    );
    return returnValues.map(toACVMField);
  }

  async enqueuePublicFunctionCall(
    [contractAddress]: ACVMField[],
    [functionSelector]: ACVMField[],
    [argsHash]: ACVMField[],
    [sideEffectCounter]: ACVMField[],
    [isStaticCall]: ACVMField[],
    [isDelegateCall]: ACVMField[],
  ) {
    await this.typedOracle.enqueuePublicFunctionCall(
      AztecAddress.fromString(contractAddress),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
      frToNumber(fromACVMField(sideEffectCounter)),
      frToBoolean(fromACVMField(isStaticCall)),
      frToBoolean(fromACVMField(isDelegateCall)),
    );
  }

  async setPublicTeardownFunctionCall(
    [contractAddress]: ACVMField[],
    [functionSelector]: ACVMField[],
    [argsHash]: ACVMField[],
    [sideEffectCounter]: ACVMField[],
    [isStaticCall]: ACVMField[],
    [isDelegateCall]: ACVMField[],
  ) {
    await this.typedOracle.setPublicTeardownFunctionCall(
      AztecAddress.fromString(contractAddress),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
      frToNumber(fromACVMField(sideEffectCounter)),
      frToBoolean(fromACVMField(isStaticCall)),
      frToBoolean(fromACVMField(isDelegateCall)),
    );
  }

  aes128Encrypt(input: ACVMField[], initializationVector: ACVMField[], key: ACVMField[]): ACVMField[] {
    // Convert each field to a number and then to a buffer (1 byte is stored in 1 field)
    const processedInput = Buffer.from(input.map(fromACVMField).map(f => f.toNumber()));
    const processedIV = Buffer.from(initializationVector.map(fromACVMField).map(f => f.toNumber()));
    const processedKey = Buffer.from(key.map(fromACVMField).map(f => f.toNumber()));

    // Encrypt the input
    const ciphertext = this.typedOracle.aes128Encrypt(processedInput, processedIV, processedKey);

    // Convert each byte of ciphertext to a field and return it
    return Array.from(ciphertext).map(byte => toACVMField(byte));
  }
}
