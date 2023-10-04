import { RETURN_VALUES_LENGTH } from '@aztec/circuits.js';
import { FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr, Point } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { UnencryptedL2Log } from '@aztec/types';

import { ACVMField } from '../acvm.js';
import { convertACVMFieldToBuffer, fromACVMField } from '../deserialize.js';
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

  computeSelector(...args: ACVMField[][]): ACVMField {
    const signature = oracleDebugCallToFormattedStr(args);
    const selector = this.typedOracle.computeSelector(signature);
    return toACVMField(selector);
  }

  getRandomField(): ACVMField {
    const val = this.typedOracle.getRandomField();
    return toACVMField(val);
  }

  async packArguments(args: ACVMField[]): Promise<ACVMField> {
    const packed = await this.typedOracle.packArguments(args.map(fromACVMField));
    return toACVMField(packed);
  }

  async getSecretKey([publicKeyX]: ACVMField[], [publicKeyY]: ACVMField[]): Promise<ACVMField[]> {
    const publicKey = new Point(fromACVMField(publicKeyX), fromACVMField(publicKeyY));
    const secretKey = await this.typedOracle.getSecretKey(publicKey);
    return [toACVMField(secretKey.low), toACVMField(secretKey.high)];
  }

  async getPublicKey([address]: ACVMField[]) {
    const { publicKey, partialAddress } = await this.typedOracle.getPublicKey(
      AztecAddress.fromField(fromACVMField(address)),
    );
    return [publicKey.x, publicKey.y, partialAddress].map(toACVMField);
  }

  async getAuthWitness([messageHash]: ACVMField[]): Promise<ACVMField[]> {
    const messageHashField = fromACVMField(messageHash);
    const witness = await this.typedOracle.getAuthWitness(messageHashField);
    if (!witness) throw new Error(`Authorization not found for message hash ${messageHashField}`);
    return witness.map(toACVMField);
  }

  async getNotes(
    [storageSlot]: ACVMField[],
    [numSelects]: ACVMField[],
    selectBy: ACVMField[],
    selectValues: ACVMField[],
    sortBy: ACVMField[],
    sortOrder: ACVMField[],
    [limit]: ACVMField[],
    [offset]: ACVMField[],
    [returnSize]: ACVMField[],
  ): Promise<ACVMField[]> {
    const notes = await this.typedOracle.getNotes(
      fromACVMField(storageSlot),
      +numSelects,
      selectBy.map(s => +s),
      selectValues.map(fromACVMField),
      sortBy.map(s => +s),
      sortOrder.map(s => +s),
      +limit,
      +offset,
    );

    const preimageLength = notes?.[0]?.preimage.length ?? 0;
    if (!notes.every(({ preimage }) => preimageLength === preimage.length)) {
      throw new Error('Preimages for a particular note type should all be the same length.');
    }

    const contractAddress = notes[0]?.contractAddress ?? Fr.ZERO;

    // Values indicates whether the note is settled or transient.
    const noteTypes = {
      isSettled: new Fr(0),
      isTransient: new Fr(1),
    };
    const flattenData = notes.flatMap(({ nonce, preimage, index }) => [
      nonce,
      index === undefined ? noteTypes.isTransient : noteTypes.isSettled,
      ...preimage,
    ]);

    const returnFieldSize = +returnSize;
    const returnData = [notes.length, contractAddress, ...flattenData].map(v => toACVMField(v));
    if (returnData.length > returnFieldSize) {
      throw new Error(`Return data size too big. Maximum ${returnFieldSize} fields. Got ${flattenData.length}.`);
    }

    const paddedZeros = Array(returnFieldSize - returnData.length).fill(toACVMField(0));
    return returnData.concat(paddedZeros);
  }

  notifyCreatedNote([storageSlot]: ACVMField[], preimage: ACVMField[], [innerNoteHash]: ACVMField[]): ACVMField {
    this.typedOracle.notifyCreatedNote(
      fromACVMField(storageSlot),
      preimage.map(fromACVMField),
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
    const { root, ...message } = await this.typedOracle.getL1ToL2Message(fromACVMField(msgKey));
    return toAcvmL1ToL2MessageLoadOracleInputs(message, root);
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
    preimage: ACVMField[],
  ): ACVMField {
    const publicKey = new Point(fromACVMField(publicKeyX), fromACVMField(publicKeyY));
    this.typedOracle.emitEncryptedLog(
      AztecAddress.fromString(contractAddress),
      Fr.fromString(storageSlot),
      publicKey,
      preimage.map(fromACVMField),
    );
    return toACVMField(0);
  }

  emitUnencryptedLog([contractAddress]: ACVMField[], [eventSelector]: ACVMField[], message: ACVMField[]): ACVMField {
    // https://github.com/AztecProtocol/aztec-packages/issues/885
    const logPayload = Buffer.concat(message.map(charBuffer => convertACVMFieldToBuffer(charBuffer).subarray(-1)));
    const log = new UnencryptedL2Log(
      AztecAddress.fromString(contractAddress),
      FunctionSelector.fromField(fromACVMField(eventSelector)),
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
  ): Promise<ACVMField[]> {
    const callStackItem = await this.typedOracle.callPrivateFunction(
      AztecAddress.fromField(fromACVMField(contractAddress)),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
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
  ) {
    const enqueuedRequest = await this.typedOracle.enqueuePublicFunctionCall(
      AztecAddress.fromString(contractAddress),
      FunctionSelector.fromField(fromACVMField(functionSelector)),
      fromACVMField(argsHash),
    );
    return toAcvmEnqueuePublicFunctionResult(enqueuedRequest);
  }
}
