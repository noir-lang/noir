import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';
import { solve_intermediate_witness as solveIntermediateWitness } from '@noir-lang/aztec_backend_wasm';

/**
 * The format for fields on the ACVM.
 */
export type ACVMField = `0x${string}`;
/**
 * The format for addresses on the ACVM.
 */
export type ACVMWitness = Map<number, ACVMField>;

export const ZERO_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES)}`;
export const ONE_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES - 1)}01`;

/**
 * The callback interface for the ACIR.
 */
export interface ACIRCallback {
  getSecretKey(params: ACVMField[]): Promise<[ACVMField]>;
  getNotes2(params: ACVMField[]): Promise<ACVMField[]>;
  getRandomField(): Promise<[ACVMField]>;
  notifyCreatedNote(params: ACVMField[]): Promise<[ACVMField]>;
  notifyNullifiedNote(params: ACVMField[]): Promise<[ACVMField]>;
  callPrivateFunction(params: ACVMField[]): Promise<ACVMField[]>;
  callPublicFunction(params: ACVMField[]): Promise<ACVMField[]>;
  enqueuePublicFunctionCall(params: ACVMField[]): Promise<ACVMField[]>;
  storageRead(params: ACVMField[]): Promise<[ACVMField]>;
  storageWrite(params: ACVMField[]): Promise<[ACVMField]>;
  viewNotesPage(params: ACVMField[]): Promise<ACVMField[]>;
  getL1ToL2Message(params: ACVMField[]): Promise<ACVMField[]>;
  /**
   * Oracle call used to emit an encrypted log.
   */
  emitEncryptedLog: (params: ACVMField[]) => Promise<ACVMField[]>;
  /**
   * Debugging utility for printing out info from Noir (i.e. console.log).
   */
  debugLog: (params: ACVMField[]) => Promise<ACVMField[]>;
}

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
export type execute = (acir: Buffer, initialWitness: ACVMWitness, oracle: ACIRCallback) => Promise<ACIRExecutionResult>;

export const acvm: execute = async (acir, initialWitness, callback) => {
  const partialWitness = await solveIntermediateWitness(
    acir,
    initialWitness,
    async (name: string, args: ACVMField[]) => {
      if (!(name in callback)) throw new Error(`Callback ${name} not found`);
      const result = await callback[name as keyof ACIRCallback](args);
      return result;
    },
  );
  return Promise.resolve({ partialWitness });
};

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
export function convertACVMFieldToBuffer(field: `0x${string}`): Buffer {
  return Buffer.from(field.slice(2), 'hex');
}

/**
 * Converts an ACVM field to a Fr.
 * @param field - The ACVM field to convert.
 * @returns The Fr.
 */
export function fromACVMField(field: `0x${string}`): Fr {
  return Fr.fromBuffer(convertACVMFieldToBuffer(field));
}

// TODO this should use an unconstrained fn in the future.
/**
 * Creates a dummy note.
 * @returns The dummy note.
 */
export function createDummyNote() {
  return [Fr.ZERO, Fr.random(), Fr.ZERO, Fr.ZERO, Fr.random(), Fr.ZERO];
}
