/* eslint-disable @typescript-eslint/ban-ts-comment */
import { expect } from "@esm-bundle/chai";
import { TEST_LOG_LEVEL } from "../../environment.js";
import { Logger } from "tslog";
import { initializeResolver } from "@noir-lang/source-resolver";
import newCompiler, {
  compile,
  init_log_level as compilerLogLevel,
} from "@noir-lang/noir_wasm";
import { Noir } from "@noir-lang/noir_js";
import { BarretenbergBackend } from "@noir-lang/backend_barretenberg";
import * as TOML from "smol-toml";
const logger = new Logger({ name: "test", minLevel: TEST_LOG_LEVEL });

await newCompiler();

compilerLogLevel("INFO");

const base_relative_path = "../../../../..";
const one_mul = "tooling/nargo_cli/tests/execution_success/1_mul";
const circuit_recursion = "compiler/integration-tests/test/circuits/recursion";

async function getFile(url: URL): Promise<string> {
  const response = await fetch(url);

  if (!response.ok) throw new Error("Network response was not OK");

  return await response.text();
}

async function getCircuit(noirSource: string) {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  initializeResolver((id: string) => {
    logger.debug("source-resolver: resolving:", id);
    return noirSource;
  });

  return compile({});
}

describe("It compiles noir program code, receiving circuit bytes and abi object.", () => {
  let circuit_main_source;
  let circuit_main_toml;
  let circuit_recursion_source;

  before(async () => {
    const circuit_main_source_url = new URL(
      `${base_relative_path}/${one_mul}/src/main.nr`,
      import.meta.url,
    );
    const circuit_main_toml_url = new URL(
      `${base_relative_path}/${one_mul}/Prover.toml`,
      import.meta.url,
    );

    circuit_main_source = await getFile(circuit_main_source_url);
    circuit_main_toml = await getFile(circuit_main_toml_url);

    const circuit_recursion_source_url = new URL(
      `${base_relative_path}/${circuit_recursion}/src/main.nr`,
      import.meta.url,
    );

    circuit_recursion_source = await getFile(circuit_recursion_source_url);
  });

  it("Should generate valid inner proof for correct input, then verify proof within a proof", async () => {
    const compiled_main_circuit = await getCircuit(circuit_main_source);

    const main_inputs = TOML.parse(circuit_main_toml);

    const backend = new BarretenbergBackend({
      bytecode: compiled_main_circuit.circuit,
      abi: compiled_main_circuit.abi,
    });

    const program = new Noir(
      {
        bytecode: compiled_main_circuit.circuit,
        abi: compiled_main_circuit.abi,
      },
      backend,
    );

    const main_proof = await program.generateFinalProof(main_inputs);

    const main_verification = await program.verifyFinalProof(main_proof);

    logger.debug("main_verification", main_verification);

    expect(main_verification).to.be.true;
  }).timeout(60 * 20e3);
});
