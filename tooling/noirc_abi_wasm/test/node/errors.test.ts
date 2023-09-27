import { expect } from "chai";
import { abiEncode } from "@noir-lang/noirc_abi";

it("errors when an integer input overflows", async () => {
  const { abi, inputs } = await import("../shared/uint_overflow");

  expect(() => abiEncode(abi, inputs, null)).to.throw(
    "The parameter foo is expected to be a Integer { sign: Unsigned, width: 32 } but found incompatible value Field(2³⁸)",
  );
});

it("errors when passing a field in place of an array", async () => {
  const { abi, inputs } = await import("../shared/field_as_array");

  expect(() => abiEncode(abi, inputs, null)).to.throw(
    "cannot parse value into Array { length: 2, typ: Field }",
  );
});

it("errors when passing an array in place of a field", async () => {
  const { abi, inputs } = await import("../shared/array_as_field");

  expect(() => abiEncode(abi, inputs, null)).to.throw(
    "cannot parse value into Field",
  );
});
