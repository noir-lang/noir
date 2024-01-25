import { MerkleTreeId, UnencryptedL2Log } from '@aztec/circuit-types';
import { RETURN_VALUES_LENGTH } from '@aztec/circuits.js';
import { EventSelector, FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import { ACVMField } from '../acvm_types.js';
import { frToNumber, fromACVMField } from '../deserialize.js';
import {
  toACVMField,
  toAcvmCallPrivateStackItem,
  toAcvmEnqueuePublicFunctionResult,
  toAcvmL1ToL2MessageLoadOracleInputs,
} from '../serialize.js';
import { acvmFieldMessageToString, oracleDebugCallToFormattedStr } from './debug.js';
import { TypedOracle } from './typed_oracle.js';

/**
 * A data source that has all the apis required by Aztec.nr.
 */
export class Oracle {
  constructor(private typedOracle: TypedOracle, private log = createDebugLogger('aztec:simulator:oracle')) {}

  getRandomField(): ACVMField {
    const val = this.typedOracle.getRandomField();
    return toACVMField(val);
  }

  async packArguments(args: ACVMField[]): Promise<ACVMField> {
    const packed = await this.typedOracle.packArguments(args.map(fromACVMField));
    return toACVMField(packed);
  }

  async getNullifierKeyPair([accountAddress]: ACVMField[]): Promise<ACVMField[]> {
    const { publicKey, secretKey } = await this.typedOracle.getNullifierKeyPair(fromACVMField(accountAddress));
    return [
      toACVMField(publicKey.x),
      toACVMField(publicKey.y),
      toACVMField(secretKey.high),
      toACVMField(secretKey.low),
    ];
  }

  async getPublicKeyAndPartialAddress([address]: ACVMField[]) {
    const { publicKey, partialAddress } = await this.typedOracle.getCompleteAddress(
      AztecAddress.fromField(fromACVMField(address)),
    );
    return [publicKey.x, publicKey.y, partialAddress].map(toACVMField);
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
      throw new Error(
        `Low nullifier witness not found for nullifier ${parsedNullifier} at block ${parsedBlockNumber}.`,
      );
    }
    return witness.toFieldArray().map(toACVMField);
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
    return witness.toFieldArray().map(toACVMField);
  }

  async getPublicDataTreeWitness([blockNumber]: ACVMField[], [leafSlot]: ACVMField[]): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));
    const parsedLeafSlot = fromACVMField(leafSlot);

    const witness = await this.typedOracle.getPublicDataTreeWitness(parsedBlockNumber, parsedLeafSlot);
    if (!witness) {
      throw new Error(`Public data witness not found for slot ${parsedLeafSlot} at block ${parsedBlockNumber}.`);
    }
    return witness.toFieldArray().map(toACVMField);
  }

  async getBlockHeader([blockNumber]: ACVMField[]): Promise<ACVMField[]> {
    const parsedBlockNumber = frToNumber(fromACVMField(blockNumber));

    const blockHeader = await this.typedOracle.getBlockHeader(parsedBlockNumber);
    if (!blockHeader) {
      throw new Error(`Block header not found for block ${parsedBlockNumber}.`);
    }
    return blockHeader.toArray().map(toACVMField);
  }

  // TODO(#3564) - Nuke this oracle and inject the number directly to context
  async getNullifierRootBlockNumber([nullifierTreeRoot]: ACVMField[]): Promise<ACVMField> {
    const parsedRoot = fromACVMField(nullifierTreeRoot);

    const blockNumber = await this.typedOracle.getNullifierRootBlockNumber(parsedRoot);
    if (!blockNumber) {
      throw new Error(`Block header not found for block ${parsedRoot}.`);
    }
    return toACVMField(blockNumber);
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

  async getNotes(
    [storageSlot]: ACVMField[],
    [numSelects]: ACVMField[],
    selectBy: ACVMField[],
    selectValues: ACVMField[],
    selectComparators: ACVMField[],
    sortBy: ACVMField[],
    sortOrder: ACVMField[],
    [limit]: ACVMField[],
    [offset]: ACVMField[],
    [returnSize]: ACVMField[],
  ): Promise<ACVMField[]> {
    const noteDatas = await this.typedOracle.getNotes(
      fromACVMField(storageSlot),
      +numSelects,
      selectBy.map(s => +s),
      selectValues.map(fromACVMField),
      selectComparators.map(s => +s),
      sortBy.map(s => +s),
      sortOrder.map(s => +s),
      +limit,
      +offset,
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

  notifyCreatedNote([storageSlot]: ACVMField[], note: ACVMField[], [innerNoteHash]: ACVMField[]): ACVMField {
    this.typedOracle.notifyCreatedNote(
      fromACVMField(storageSlot),
      note.map(fromACVMField),
      fromACVMField(innerNoteHash),
    );
    return toACVMField(0);
  }

  async notifyNullifiedNote([innerNullifier]: ACVMField[], [innerNoteHash]: ACVMField[]): Promise<ACVMField> {
    await this.typedOracle.notifyNullifiedNote(fromACVMField(innerNullifier), fromACVMField(innerNoteHash));
    return toACVMField(0);
  }

  async checkNullifierExists([innerNullifier]: ACVMField[]): Promise<ACVMField> {
    const exists = await this.typedOracle.checkNullifierExists(fromACVMField(innerNullifier));
    return toACVMField(exists);
  }

  async getL1ToL2Message([msgKey]: ACVMField[]): Promise<ACVMField[]> {
    const { ...message } = await this.typedOracle.getL1ToL2Message(fromACVMField(msgKey));
    return toAcvmL1ToL2MessageLoadOracleInputs(message);
  }

  async getPortalContractAddress([aztecAddress]: ACVMField[]): Promise<ACVMField> {
    const contractAddress = AztecAddress.fromString(aztecAddress);
    const portalContactAddress = await this.typedOracle.getPortalContractAddress(contractAddress);
    return toACVMField(portalContactAddress);
  }

  async storageRead([startStorageSlot]: ACVMField[], [numberOfElements]: ACVMField[]): Promise<ACVMField[]> {
    const values = await this.typedOracle.storageRead(fromACVMField(startStorageSlot), +numberOfElements);
    return values.map(toACVMField);
  }

  async storageWrite([startStorageSlot]: ACVMField[], values: ACVMField[]): Promise<ACVMField[]> {
    const newValues = await this.typedOracle.storageWrite(fromACVMField(startStorageSlot), values.map(fromACVMField));
    return newValues.map(toACVMField);
  }

  emitEncryptedLog(
    [contractAddress]: ACVMField[],
    [storageSlot]: ACVMField[],
    [publicKeyX]: ACVMField[],
    [publicKeyY]: ACVMField[],
    log: ACVMField[],
  ): ACVMField {
    const publicKey = new Point(fromACVMField(publicKeyX), fromACVMField(publicKeyY));
    this.typedOracle.emitEncryptedLog(
      AztecAddress.fromString(contractAddress),
      Fr.fromString(storageSlot),
      publicKey,
      log.map(fromACVMField),
    );
    return toACVMField(0);
  }

  emitUnencryptedLog([contractAddress]: ACVMField[], [eventSelector]: ACVMField[], message: ACVMField[]): ACVMField {
    const logPayload = Buffer.concat(message.map(charBuffer => Fr.fromString(charBuffer).toBuffer().subarray(-1)));
    const log = new UnencryptedL2Log(
      AztecAddress.fromString(contractAddress),
      EventSelector.fromField(fromACVMField(eventSelector)),
      logPayload,
    );

    this.typedOracle.emitUnencryptedLog(log);
    return toACVMField(0);
  }

  debugLog(...args: ACVMField[][]): ACVMField {
    this.log(oracleDebugCallToFormattedStr(args));
    return toACVMField(0);
  }

  debugLogWithPrefix(arg0: ACVMField[], ...args: ACVMField[][]): ACVMField {
    this.log(`${acvmFieldMessageToString(arg0)}: ${oracleDebugCallToFormattedStr(args)}`);
    return toACVMField(0);
  }

  async callPrivateFunction(
    [contractAddress]: ACVMField[],
    [functionSelector]: ACVMField[],
    [argsHash]: ACVMField[],
    [sideffectCounter]: ACVMField[],
  ): Promise<ACVMField[]> {
    const callStackItem = await this.typedOracle.callPrivateFunction(
      AztecAddress.fromField(fromACVMField(contractAddress)),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
      frToNumber(fromACVMField(sideffectCounter)),
    );
    return toAcvmCallPrivateStackItem(callStackItem);
  }

  async callPublicFunction(
    [contractAddress]: ACVMField[],
    [functionSelector]: ACVMField[],
    [argsHash]: ACVMField[],
  ): Promise<ACVMField[]> {
    const returnValues = await this.typedOracle.callPublicFunction(
      AztecAddress.fromField(fromACVMField(contractAddress)),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
    );
    return padArrayEnd(returnValues, Fr.ZERO, RETURN_VALUES_LENGTH).map(toACVMField);
  }

  async enqueuePublicFunctionCall(
    [contractAddress]: ACVMField[],
    [functionSelector]: ACVMField[],
    [argsHash]: ACVMField[],
    [sideffectCounter]: ACVMField[],
  ) {
    const enqueuedRequest = await this.typedOracle.enqueuePublicFunctionCall(
      AztecAddress.fromString(contractAddress),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
      frToNumber(fromACVMField(sideffectCounter)),
    );
    return toAcvmEnqueuePublicFunctionResult(enqueuedRequest);
  }
}
