import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import { ForeignCallInput, ForeignCallOutput, WitnessMap, executeCircuit } from 'acvm_js';

/**
 * The format for fields on the ACVM.
 */
export type ACVMField = string;
/**
 * The format for witnesses of the ACVM.
 */
export type ACVMWitness = WitnessMap;

export const ZERO_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES)}`;
export const ONE_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES - 1)}01`;

/**
 * The supported oracle names.
 */
type ORACLE_NAMES =
  | 'packArguments'
  | 'getSecretKey'
  | 'getNote'
  | 'getNotes'
  | 'getRandomField'
  | 'notifyCreatedNote'
  | 'notifyNullifiedNote'
  | 'callPrivateFunction'
  | 'callPublicFunction'
  | 'enqueuePublicFunctionCall'
  | 'storageRead'
  | 'storageWrite'
  | 'createCommitment'
  | 'createL2ToL1Message'
  | 'createNullifier'
  | 'getCommitment'
  | 'getL1ToL2Message'
  | 'getPortalContractAddress'
  | 'emitEncryptedLog'
  | 'emitUnencryptedLog'
  | 'getPublicKey'
  | 'debugLog'
  | 'debugLogWithPrefix';

/**
 * A type that does not require all keys to be present.
 */
type PartialRecord<K extends keyof any, T> = Partial<Record<K, T>>;

/**
 * The callback interface for the ACIR.
 */
export type ACIRCallback = PartialRecord<ORACLE_NAMES, (...args: ForeignCallInput[]) => Promise<ForeignCallOutput>>;

/**
 * The result of executing an ACIR.
 */
export interface ACIRExecutionResult {
  /**
   * The partial witness of the execution.
   */
  partialWitness: ACVMWitness;
}

/**
 * The function call that executes an ACIR.
 */
export async function acvm(
  acir: Buffer,
  initialWitness: ACVMWitness,
  callback: ACIRCallback,
): Promise<ACIRExecutionResult> {
  const logger = createDebugLogger('aztec:simulator:acvm');
  const partialWitness = await executeCircuit(acir, initialWitness, async (name: string, args: ForeignCallInput[]) => {
    try {
      logger(`Oracle callback ${name}`);
      const oracleFunction = callback[name as ORACLE_NAMES];
      if (!oracleFunction) {
        throw new Error(`Oracle callback ${name} not found`);
      }

      const result = await oracleFunction.call(callback, ...args);
      return [result];
    } catch (err: any) {
      logger.error(`Error in oracle callback ${name}: ${err.message ?? err ?? 'Unknown'}`);
      throw err;
    }
  });
  return Promise.resolve({ partialWitness });
}

/**
 * Adapts the buffer to the field size.
 * @param originalBuf - The buffer to adapt.
 * @returns The adapted buffer.
 */
function adaptBufferSize(originalBuf: Buffer) {
  const buffer = Buffer.alloc(Fr.SIZE_IN_BYTES);
  if (originalBuf.length > buffer.length) {
    throw new Error('Buffer does not fit in field');
  }
  originalBuf.copy(buffer, buffer.length - originalBuf.length);
  return buffer;
}

/**
 * Converts a value to an ACVM field.
 * @param value - The value to convert.
 * @returns The ACVM field.
 */
export function toACVMField(value: AztecAddress | EthAddress | Fr | Buffer | boolean | number | bigint): ACVMField {
  if (typeof value === 'boolean') {
    return value ? ONE_ACVM_FIELD : ZERO_ACVM_FIELD;
  }

  let buffer;

  if (Buffer.isBuffer(value)) {
    buffer = value;
  } else if (typeof value === 'number') {
    buffer = Buffer.alloc(Fr.SIZE_IN_BYTES);
    buffer.writeUInt32BE(value, Fr.SIZE_IN_BYTES - 4);
  } else if (typeof value === 'bigint') {
    buffer = new Fr(value).toBuffer();
  } else {
    buffer = value.toBuffer();
  }

  return `0x${adaptBufferSize(buffer).toString('hex')}`;
}

/**
 * Converts an ACVM field to a Buffer.
 * @param field - The ACVM field to convert.
 * @returns The Buffer.
 */
export function convertACVMFieldToBuffer(field: ACVMField): Buffer {
  return Buffer.from(field.slice(2), 'hex');
}

/**
 * Converts an ACVM field to a Fr.
 * @param field - The ACVM field to convert.
 * @returns The Fr.
 */
export function fromACVMField(field: ACVMField): Fr {
  return Fr.fromBuffer(convertACVMFieldToBuffer(field));
}
