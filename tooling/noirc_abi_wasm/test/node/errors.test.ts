import { expect } from 'chai';
import { abiEncode } from '@noir-lang/noirc_abi';
import { abi as abi_uint_overflow, inputs as inputs_uint_overflow } from '../shared/uint_overflow';
import { abi as abi_field_as_array, inputs as inputs_field_as_array } from '../shared/field_as_array';
import { abi as abi_array_as_field, inputs as inputs_array_as_field } from '../shared/array_as_field';

it('errors when an integer input overflows', () => {
  expect(() => abiEncode(abi_uint_overflow, inputs_uint_overflow)).to.throw(
    'The value passed for parameter `foo` does not match the specified type:\nValue Field(274877906944) does not fall within range of allowable values for a Integer { sign: Unsigned, width: 32 }',
  );
});

it('errors when passing a field in place of an array', () => {
  expect(() => abiEncode(abi_field_as_array, inputs_field_as_array)).to.throw(
    'cannot parse value `String("1")` into Array { length: 2, typ: Field }',
  );
});

it('errors when passing an array in place of a field', () => {
  expect(() => abiEncode(abi_array_as_field, inputs_array_as_field)).to.throw(
    'cannot parse value `Array([String("1"), String("2")])` into Field',
  );
});
