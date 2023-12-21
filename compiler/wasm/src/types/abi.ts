/**
 * A named type.
 */
export interface ABIVariable {
  /**
   * The name of the variable.
   */
  name: string;
  /**
   * The type of the variable.
   */
  type: ABIType;
}

/**
 * Indicates whether a parameter is public or secret/private.
 */
export enum ABIParameterVisibility {
  PUBLIC = 'public',
  SECRET = 'secret',
}

/**
 * A function parameter.
 */
export interface ABIParameter extends ABIVariable {
  /**
   * Indicates whether a parameter is public or secret/private.
   */
  visibility: ABIParameterVisibility;
}

/**
 * A basic type.
 */
export interface BasicType<T extends string> {
  /**
   * The kind of the type.
   */
  kind: T;
}

/**
 * A variable type.
 */
export type ABIType = BasicType<'field'> | BasicType<'boolean'> | IntegerType | ArrayType | StringType | StructType;

/**
 * An integer type.
 */
export interface IntegerType extends BasicType<'integer'> {
  /**
   * The sign of the integer.
   */
  sign: string;
  /**
   * The width of the integer in bits.
   */
  width: number;
}

/**
 * An array type.
 */
export interface ArrayType extends BasicType<'array'> {
  /**
   * The length of the array.
   */
  length: number;
  /**
   * The type of the array elements.
   */
  type: ABIType;
}

/**
 * A string type.
 */
export interface StringType extends BasicType<'string'> {
  /**
   * The length of the string.
   */
  length: number;
}

/**
 * A struct type.
 */
export interface StructType extends BasicType<'struct'> {
  /**
   * The fields of the struct.
   */
  fields: ABIVariable[];
  /**
   * Fully qualified name of the struct.
   */
  path: string;
}

/**
 * A contract event.
 */
export interface EventAbi {
  /**
   * The event name.
   */
  name: string;
  /**
   * Fully qualified name of the event.
   */
  path: string;
  /**
   * The fields of the event.
   */
  fields: ABIVariable[];
}
