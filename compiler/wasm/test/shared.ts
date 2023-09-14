import { initialiseResolver } from "@noir-lang/noir-source-resolver";
import { compile } from "@noir-lang/noir_wasm";

export const noirSourcePath = "../../noir-script/src/main.nr";
export const nargoArtifactPath =
  "../../noir-script/target/noir_wasm_testing.json";

export async function compileNoirSource(noir_source: string): Promise<any> {
  console.log("Compiling Noir source...");

  initialiseResolver((id: String) => {
    console.log(`Resolving source ${id}`);

    const source = noir_source;

    if (typeof source === "undefined") {
      throw Error(`Could not resolve source for '${id}'`);
    } else {
      return source;
    }
  });

  try {
    const compiled_noir = compile({});

    console.log("Noir source compilation done.");

    return compiled_noir.circuit;
  } catch (e) {
    console.log("Error while compiling:", e);
  }
}
