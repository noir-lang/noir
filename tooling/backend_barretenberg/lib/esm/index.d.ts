import { Backend, CompiledCircuit } from '@noir-lang/types';
export declare function base64Decode(input: string): Uint8Array;
export declare function acirToUint8Array(base64EncodedBytecode: any): Uint8Array;
export declare class BarretenbergBackend implements Backend {
    private api;
    private acirComposer;
    private acirUncompressedBytecode;
    private numberOfThreads;
    constructor(acirCircuit: CompiledCircuit, numberOfThreads?: number);
    instantiate(): Promise<void>;
    generateProof(decompressedWitness: Uint8Array, optimizeForVerifyInCircuit?: boolean): Promise<Uint8Array>;
    generateChildProof(witness: Uint8Array): Promise<Uint8Array>;
    generateChildProofArtifacts(proof: Uint8Array, numOfPublicInputs?: number): Promise<{
        proofAsFields: string[];
        vkAsFields: string[];
        vkHash: string;
    }>;
    verifyChildProof(proof: Uint8Array): Promise<boolean>;
    verifyProof(proof: Uint8Array, optimizeForVerifyInCircuit?: boolean): Promise<boolean>;
    destroy(): Promise<void>;
}
