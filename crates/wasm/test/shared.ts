// it's the trick to provide mocha testing. The module `@noir-lang/noir-source-resolver` has no typings to resolve
// import { initialiseResolver } from "@noir-lang/noir-source-resolver";
const initialiseResolver =
    require("@noir-lang/noir-source-resolver").initialiseResolver;
import { compile } from "../result/";

export const noirSourcePath = "../../noir-script/src/main.nr";
export const nargoArtifactPath =
  "../../noir-script/target/noir_wasm_testing.json";

export async function compileNoirSource(noir_source: string): Promise<any> {
  console.log("Compiling Noir source...");

  initialiseResolver((id: string) => {
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
