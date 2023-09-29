import { Backend, CompiledCircuit } from '@noir-lang/types';
export declare class Noir {
    private circuit;
    private backend;
    constructor(circuit: CompiledCircuit, backend: Backend);
    generateFinalProof(inputs: any): Promise<Uint8Array>;
    verifyFinalProof(proof: Uint8Array): Promise<boolean>;
}
