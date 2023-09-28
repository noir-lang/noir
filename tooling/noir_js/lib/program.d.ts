import { Backend, CompiledCircuit } from '@noir-lang/types';
export declare class Program {
    private circuit;
    private backend;
    constructor(circuit: CompiledCircuit, backend: Backend);
    generateProof(inputs: any): Promise<Uint8Array>;
    verifyProof(proof: Uint8Array): Promise<boolean>;
}
