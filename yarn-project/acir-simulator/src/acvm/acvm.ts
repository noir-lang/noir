import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';
import { solve_intermediate_witness as solveIntermediateWitness } from '@noir-lang/aztec_backend_wasm';

export type ACVMField = `0x${string}`;
export type ACVMWitness = Map<number, ACVMField>;

export const ZERO_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES)}`;
export const ONE_ACVM_FIELD: ACVMField = `0x${'00'.repeat(Fr.SIZE_IN_BYTES - 1)}01`;

export interface ACIRCallback {
  getSecretKey(params: ACVMField[]): Promise<[ACVMField]>;
  getNotes2(params: ACVMField[]): Promise<ACVMField[]>;
  getRandomField(): Promise<[ACVMField]>;
  notifyCreatedNote(params: ACVMField[]): Promise<[ACVMField]>;
  notifyNullifiedNote(params: ACVMField[]): Promise<[ACVMField]>;
  callPrivateFunction(params: ACVMField[]): Promise<ACVMField[]>;
  storageRead(params: ACVMField[]): Promise<[ACVMField]>;
  storageWrite(params: ACVMField[]): Promise<[ACVMField]>;
  viewNotesPage(params: ACVMField[]): Promise<ACVMField[]>;
}

export interface ACIRExecutionResult {
  partialWitness: ACVMWitness;
}

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

function adaptBufferSize(originalBuf: Buffer) {
  const buffer = Buffer.alloc(Fr.SIZE_IN_BYTES);
  if (originalBuf.length > buffer.length) {
    throw new Error('Buffer does not fit in field');
  }
  originalBuf.copy(buffer, buffer.length - originalBuf.length);
  return buffer;
}

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

export function fromACVMField(field: `0x${string}`): Fr {
  const buffer = Buffer.from(field.slice(2), 'hex');
  return Fr.fromBuffer(buffer);
}

// TODO this should use an unconstrained fn in the future
export function createDummyNote() {
  return [Fr.ZERO, Fr.random(), Fr.ZERO, Fr.ZERO, Fr.random(), Fr.ZERO];
}
