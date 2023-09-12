import { expect } from '@esm-bundle/chai';
import { initialiseResolver } from "@noir-lang/noir-source-resolver";
import newCompiler, {
    compile,
    init_log_level as compilerLogLevel
} from "@noir-lang/noir_wasm";
import { decompressSync as gunzip } from 'fflate';
import newABICoder, { abiEncode } from "@noir-lang/noirc_abi";
import initACVM, {
    executeCircuit,
    WitnessMap,
    compressWitness,
} from "@noir-lang/acvm_js";

// @ts-ignore
import { Barretenberg, RawBuffer, Crs } from '@aztec/bb.js';

import * as TOML from 'smol-toml'


await newCompiler();
await newABICoder();
await initACVM();

compilerLogLevel("DEBUG");

async function getFile(url: URL): Promise<string> {

    const response = await fetch(url)

    if (!response.ok) throw new Error('Network response was not OK');

    return await response.text();
}

const CIRCUIT_SIZE = 2 ** 19;


const test_cases = [
    {
        case: "tooling/nargo_cli/tests/execution_success/1_mul"
    },
    {
        case: "tooling/nargo_cli/tests/execution_success/double_verify_proof"
    }
];

const numberOfThreads = navigator.hardwareConcurrency || 1;

let suite = Mocha.Suite.create(mocha.suite, "Noir end to end test");

suite.timeout(60*20e3);//20mins

test_cases.forEach((testInfo) => {
    const test_name = testInfo.case.split("/").pop();
    const mochaTest = new Mocha.Test(`${test_name} (Compile, Execute, Prove, Verify)`, async () => {

        const base_relative_path = "../../../../..";
        const test_case = testInfo.case;

        const noir_source_url = new URL(`${base_relative_path}/${test_case}/src/main.nr`, import.meta.url);
        const prover_toml_url = new URL(`${base_relative_path}/${test_case}/Prover.toml`, import.meta.url);

        const noir_source = await getFile(noir_source_url);
        const prover_toml = await getFile(prover_toml_url);

        expect(noir_source).to.be.a.string;

        initialiseResolver((id: String) => {
            console.log("Resolving:", id);
            return noir_source;
        });

        const inputs = TOML.parse(prover_toml);

        expect(inputs, "Prover.toml").to.be.an('object');

        let compile_output;

        try {

            compile_output = await compile({});

            expect(await compile_output, "Compile output ").to.be.an('object');

        } catch (e) {
            expect(e, "Compilation Step").to.not.be.an('error');
            throw e;
        }


        let witnessMap: WitnessMap;
        try {

            witnessMap = abiEncode(compile_output.abi, inputs, null);

        } catch (e) {
            expect(e, "Abi Encoding Step").to.not.be.an('error');
            throw e;
        }

        let solvedWitness: WitnessMap;
        let compressedByteCode;
        try {
            compressedByteCode = Uint8Array.from(atob(compile_output.circuit), c => c.charCodeAt(0));

            solvedWitness = await executeCircuit(
                compressedByteCode,
                witnessMap,
                () => {
                    throw Error("unexpected oracle");
                }
            );

        } catch (e) {
            expect(e, "Abi Encoding Step").to.not.be.an('error');
            throw e;
        }

        try {
            const compressedWitness = compressWitness(solvedWitness);
            const acirUint8Array = gunzip(compressedByteCode);
            const witnessUint8Array = gunzip(compressedWitness);

            const isRecursive = true;
            const api = await Barretenberg.new(numberOfThreads);
            await api.commonInitSlabAllocator(CIRCUIT_SIZE);

            // Plus 1 needed!
            const crs = await Crs.new(CIRCUIT_SIZE + 1);
            await api.srsInitSrs(new RawBuffer(crs.getG1Data()), crs.numPoints, new RawBuffer(crs.getG2Data()));

            const acirComposer = await api.acirNewAcirComposer(CIRCUIT_SIZE);

            // This took ~6.5 minutes!
            const proof = await api.acirCreateProof(
                acirComposer,
                acirUint8Array,
                witnessUint8Array,
                isRecursive
            );

            // And this took ~5 minutes!
            const verified = await api.acirVerifyProof(acirComposer, proof, isRecursive);

            expect(verified).to.be.true;

        } catch (e) {
            expect(e, "Proving and Verifying").to.not.be.an('error');
            throw e;
        }

    });

    suite.addTest(mochaTest);

});
