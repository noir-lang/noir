import { solve_intermediate_witness as solveIntermediateWitness } from '@noir-lang/aztec_backend_wasm';
import { ACVMField, ACVMWitness } from './fields.js';

export interface ACIRCallback {
  getSecretKey(params: ACVMField[]): Promise<ACVMField[]>;
  getNotes2(params: ACVMField[]): Promise<ACVMField[]>;
  getRandomField(): Promise<ACVMField[]>;
  notifyCreatedNote(params: ACVMField[]): Promise<ACVMField[]>;
  notifyNullifiedNote(params: ACVMField[]): Promise<ACVMField[]>;
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
