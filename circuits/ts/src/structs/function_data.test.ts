import { expectSerializeToMatchSnapshot } from "../tests/expectSerializeToMatchSnapshot.js";
import { FunctionData } from "./function_data.js";

describe("basic FunctionData serialization", () => {
  it(`serializes a trivial FunctionData and prints it`, async () => {
    // Test the data case: writing (mostly) sequential numbers
    await expectSerializeToMatchSnapshot(
      new FunctionData(123, true, true).toBuffer(),
      "abis__test_roundtrip_serialize_function_data"
    );
  });
});
