import { Backend, CompiledCircuit } from '@noir-lang/types';
export declare class Program {
    private circuit;
    private backend;
    constructor(circuit: CompiledCircuit, backend: Backend);
    generateProof(inputs: any, optimizeForVerifyInCircuit?: boolean): Promise<Uint8Array>;
    verifyProof(proof: Uint8Array, optimizeForVerifyInCircuit?: boolean): Promise<boolean>;
}
