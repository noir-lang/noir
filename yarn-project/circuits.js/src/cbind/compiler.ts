import camelCase from 'lodash.camelcase';

import { USES_MSGPACK_BUFFER_METHODS } from './type_data.js';

/**
 * Capitalize the first character of a given string.
 * This function takes a string input and returns a new string
 * with the first character converted to uppercase, while keeping
 * the rest of the characters unchanged.
 *
 * @param s - The input string to be capitalized.
 * @returns A new string with the first character capitalized.
 */
function capitalize(s: string) {
  return s.charAt(0).toUpperCase() + s.substring(1);
}

/**
 * Represents an object schema where keys are mapped to their corresponding type schemas, defining a structured data model.
 */
type ObjectSchema = { [key: string]: Schema };

/**
 * Represents the various data structures and types used to model schema definitions.
 * The Schema type supports primitive types, object schemas, tuples, maps, optional values,
 * fixed-size arrays, shared pointers, and custom type aliases (defined in schema_map_impl.hpp).
 */
type Schema =
  | string
  | ObjectSchema
  | ['tuple', Schema[]]
  | ['map', [Schema, Schema]]
  | ['optional', [Schema]]
  | ['vector', [Schema]]
  | ['variant', Schema[]]
  | ['shared_ptr', [Schema]]
  | ['array', [Schema, number]]
  | ['alias', [string, string]];

/**
 * Represents a detailed description of a schema's type information.
 * Provides metadata and conversion methods related to the TypeScript and Msgpack type names,
 * as well as any required dependencies or custom behavior for specific schemas.
 */
export interface TypeInfo {
  /**
   * High-level typescript type name.
   */
  typeName: string;
  /**
   * Msgpack type name. The actual type returned by raw C-binds.
   * Only given if different.
   */
  msgpackTypeName?: string;
  /**
   * Indicates if the schema requires an interface.
   */
  needsInterface?: boolean;
  /**
   * Indicates if the schema refers to an imported type.
   */
  isImport?: boolean;
  /**
   * Indicates if the type is an alias of another type.
   */
  isAlias?: boolean;
  /**
   * Indicates if the schema represents a tuple type.
   */
  isTuple?: boolean;
  /**
   * Indicates if the schama represents an array.
   * If so, stores the array's subtype elements.
   */
  arraySubtype?: TypeInfo;
  /**
   * Indicates if the schama represents a variant.
   * If so, stores the variant's subtype elements.
   */
  variantSubtypes?: TypeInfo[];
  /**
   * Was this used in a variant type?
   * Typically a variant in C++ will have an easy to distinguish type as
   * one of two structs e.g. [Error, T]. In that case, a isError method would be imported. Only if a third type was
   * added would we need to distinguish T as well.
   */
  usedInDiscriminatedVariant?: boolean;
  /**
   * Key-value pair of types that represent the keys and values in a map schema.
   */
  mapSubtypes?: [TypeInfo, TypeInfo];
  /**
   * Represents the TypeScript interface declaration for a specific schema type.
   */
  declaration?: string;
  /**
   * Conversion method to transform Msgpack data into a class instance.
   */
  toClassMethod?: string;
  /**
   * Converts a class instance to its Msgpack representation.
   */
  fromClassMethod?: string;
  /**
   * Represents the conversion method from class to Msgpack format.
   */
  toMsgpackMethod?: string;
}

/**
 * Generate a JavaScript expression to convert a given value from its Msgpack type representation to its
 * corresponding TypeScript type representation using the provided TypeInfo.
 *
 * @param typeInfo - Metadata and conversion methods related to the TypeScript and Msgpack type names.
 * @param value - The value to be converted in the generated expression.
 * @returns A JavaScript expression that converts the input value based on the provided TypeInfo.
 */
function msgpackConverterExpr(typeInfo: TypeInfo, value: string): string {
  const { typeName } = typeInfo;
  if (typeInfo.isAlias) {
    if (USES_MSGPACK_BUFFER_METHODS.includes(typeInfo.typeName)) {
      // TODO(AD) Temporary hack while two serialization systems exist for these classes
      return `${typeName}.fromMsgpackBuffer(${value})`;
    }
    if (typeInfo.msgpackTypeName === 'number') {
      return `${value} as ${typeName}`;
    }
    return `${typeName}.fromBuffer(${value})`;
  } else if (typeInfo.arraySubtype) {
    const { typeName, msgpackTypeName } = typeInfo.arraySubtype;
    const convFn = `(v: ${msgpackTypeName || typeName}) => ${msgpackConverterExpr(typeInfo.arraySubtype, 'v')}`;
    if (typeInfo.isTuple) {
      return `mapTuple(${value}, ${convFn})`;
    } else {
      return `${value}.map(${convFn})`;
    }
  } else if (typeInfo.variantSubtypes) {
    const { variantSubtypes } = typeInfo;
    // Handle the last variant type: just assume it is this type...
    let expr = msgpackConverterExpr(variantSubtypes[variantSubtypes.length - 1], 'v');
    // ... because we check every other type:
    for (let i = 0; i < variantSubtypes.length - 1; i++) {
      // mark this as needing an import
      variantSubtypes[i].usedInDiscriminatedVariant = true;
      // make the expr a compound expression with a discriminator
      expr = `(is${variantSubtypes[i].typeName}(v) ? ${msgpackConverterExpr(variantSubtypes[i], 'v')} : ${expr})`;
    }
    return `((v: ${typeInfo.msgpackTypeName}) => ${expr})(${value})`;
  } else if (typeInfo.mapSubtypes) {
    const { typeName, msgpackTypeName } = typeInfo.mapSubtypes[1];
    const convFn = `(v: ${msgpackTypeName || typeName}) => ${msgpackConverterExpr(typeInfo.mapSubtypes[1], 'v')}`;
    return `mapValues(${value}, ${convFn})`;
  } else if (typeInfo.isImport) {
    return `to${typeName}(${value})`;
  } else {
    return value;
  }
}

/**
 * Generate a JavaScript expression to convert a given value from its TypeScript class representation to its
 * corresponding Msgpack type representation using the provided TypeInfo.
 *
 * @param typeInfo - Metadata and conversion methods related to the TypeScript and Msgpack type names.
 * @param value - The value to be converted in the generated expression.
 * @returns A JavaScript expression that converts the input value based on the provided TypeInfo.
 */
function classConverterExpr(typeInfo: TypeInfo, value: string): string {
  const { typeName } = typeInfo;
  if (typeInfo.isAlias) {
    // TODO other aliases besides Buffer?
    if (USES_MSGPACK_BUFFER_METHODS.includes(typeInfo.typeName)) {
      // TODO(AD) Temporary hack while two serialization systems exist for these classes
      return `${value}.toMsgpackBuffer()`;
    }
    if (typeInfo.msgpackTypeName === 'number') {
      return `${value}`; // Should be a branded number alias
    }
    return `toBuffer(${value})`;
  } else if (typeInfo.arraySubtype) {
    const { typeName } = typeInfo.arraySubtype;
    const convFn = `(v: ${typeName}) => ${classConverterExpr(typeInfo.arraySubtype, 'v')}`;
    if (typeInfo.isTuple) {
      return `mapTuple(${value}, ${convFn})`;
    } else {
      return `${value}.map(${convFn})`;
    }
  } else if (typeInfo.variantSubtypes) {
    throw new Error('TODO - variant parameters to C++ not yet supported');
  } else if (typeInfo.mapSubtypes) {
    const { typeName } = typeInfo.mapSubtypes[1];
    const convFn = `(v: ${typeName}) => ${classConverterExpr(typeInfo.mapSubtypes[1], 'v')}`;
    return `mapValues(${value}, ${convFn})`;
  } else if (typeInfo.isImport) {
    return `from${typeName}(${value})`;
  } else {
    return value;
  }
}
/**
 * Converts a spec emitted from the WASM.
 * Creates typescript code.
 */
export class CbindCompiler {
  // Function and declaration output fragments
  private typeInfos: Record<string, TypeInfo> = {};
  // cbind outputs, put at end
  private funcDecls: string[] = [];

  /**
   * Retrieve the TypeScript type name for a given schema.
   * This function utilizes the TypeInfo cache to obtain the appropriate type name
   * and handles any necessary type compilation along the way.
   *
   * @param type - The input schema for which to retrieve the TypeScript type name.
   * @returns The corresponding TypeScript type name as a string.
   */
  private getTypeName(type: Schema): string {
    return this.getTypeInfo(type).typeName;
  }
  /**
   * Derive the TypeScript type name of a schema, compiling anything needed along the way.
   * @param type - A schema.
   * @returns The type name.
   */
  private getTypeInfo(type: Schema): TypeInfo {
    if (Array.isArray(type)) {
      if (type[0] === 'array') {
        // fixed-size array case
        const [_array, [subtype, size]] = type;
        const typeName = `Tuple<${this.getTypeName(subtype)}, ${size}>`;
        const msgpackTypeName = `Tuple<${this.getMsgpackTypename(subtype)}, ${size}>`;
        return {
          typeName,
          msgpackTypeName,
          isTuple: true,
          arraySubtype: this.getTypeInfo(subtype),
        };
      } else if (type[0] === 'variant') {
        // fixed-size array case
        const [_array, variantSchemas] = type;
        const typeName = variantSchemas.map(vs => this.getTypeName(vs)).join(' | ');
        const msgpackTypeName = variantSchemas.map(vs => this.getMsgpackTypename(vs)).join(' | ');
        return {
          typeName,
          msgpackTypeName,
          variantSubtypes: variantSchemas.map(vs => this.getTypeInfo(vs)),
        };
      } else if (type[0] === 'vector') {
        // vector case
        const [_vector, [subtype]] = type;
        if (subtype == 'unsigned char') {
          // buffer special case
          return { typeName: 'Buffer' };
        }
        const subtypeInfo = this.getTypeInfo(subtype);
        return {
          typeName: `${subtypeInfo.typeName}[]`,
          msgpackTypeName: `${this.getMsgpackTypename(subtype)}[]`,
          arraySubtype: subtypeInfo,
        };
      } else if (type[0] === 'alias') {
        // alias case
        const [_alias, [rawTypeName, msgpackName]] = type;
        let msgpackTypeName: string;
        if (msgpackName.startsWith('bin')) {
          msgpackTypeName = 'Buffer';
        } else if (msgpackName === 'int' || msgpackName === 'unsigned int' || msgpackName === 'unsigned short') {
          msgpackTypeName = 'number';
        } else {
          throw new Error('Unsupported alias type ' + msgpackName);
        }
        const typeName = capitalize(camelCase(rawTypeName));
        this.typeInfos[typeName] = {
          typeName,
          isImport: true,
          isAlias: true,
          msgpackTypeName,
        };
        return this.typeInfos[typeName];
      } else if (type[0] === 'shared_ptr') {
        // shared_ptr case
        const [_sharedPtr, [subtype]] = type;
        return this.getTypeInfo(subtype);
      } else if (type[0] === 'map') {
        // map case
        const [_map, [keyType, valueType]] = type;
        return {
          typeName: `Record<${this.getTypeName(keyType)}, ${this.getTypeName(valueType)}>`,
          msgpackTypeName: `Record<${this.getMsgpackTypename(keyType)}, ${this.getMsgpackTypename(valueType)}>`,
          mapSubtypes: [this.getTypeInfo(keyType), this.getTypeInfo(valueType)],
        };
      }
    } else if (typeof type === 'string') {
      switch (type) {
        case 'bool':
          return { typeName: 'boolean' };
        case 'int':
        case 'unsigned int':
        case 'unsigned short':
          return { typeName: 'number' };
        case 'string':
          return { typeName: 'string' };
        case 'bin32':
          return { typeName: 'Buffer' };
      }
      const typeName = capitalize(camelCase(type));
      if (!this.typeInfos[typeName]) {
        throw new Error(
          'Unexpected type: ' +
            typeName +
            '. This is likely due to returning a struct without a MSGPACK_FIELDS macro, and without a msgpack_schema method.',
        );
      }
      return this.typeInfos[typeName];
    } else if (typeof type === 'object') {
      const typeName = capitalize(camelCase(type.__typename as string));
      // Set our typeInfos object to either what it already was, or, if not yet defined
      // the resolved type info (which will generate interfaces and helper methods)
      return (this.typeInfos[typeName] = this.typeInfos[typeName] || {
        typeName,
        msgpackTypeName: 'Msgpack' + typeName,
        isImport: true,
        declaration: this.generateInterface(typeName, type),
        toClassMethod: this.generateMsgpackConverter(typeName, type),
        fromClassMethod: this.generateClassConverter(typeName, type),
      });
    }

    throw new Error(`Unsupported type: ${type}`);
  }

  /**
   * Retrieve the Msgpack type name for a given schema.
   * This function returns the MsgpackTypeName if available, or the default TypeName otherwise.
   * It is useful for handling cases where the Msgpack type representation differs from the TypeScript type,
   * ensuring proper serialization and deserialization between the two formats.
   *
   * @param schema - The schema for which the Msgpack type name is required.
   * @returns The Msgpack type name corresponding to the input schema.
   */
  private getMsgpackTypename(schema: Schema): string {
    const { msgpackTypeName, typeName } = this.getTypeInfo(schema);
    return msgpackTypeName || typeName;
  }
  /**
   * Generate an interface with the name 'name'.
   * @param name - The interface name.
   * @param type - The object schema with properties of the interface.
   * @returns the interface body.
   */
  private generateInterface(name: string, type: ObjectSchema) {
    // Raw object, used as return value of fromType() generated functions.
    let result = `interface Msgpack${name} {\n`;
    for (const [key, value] of Object.entries(type)) {
      if (key === '__typename') continue;
      result += `  ${key}: ${this.getMsgpackTypename(value)};\n`;
    }
    result += '}';
    return result;
  }

  /**
   * Generate conversion method 'toName' for a specific type 'name'.
   * @param name - The class name.
   * @param type - The object schema with properties of the interface.
   * @returns The toName method.
   */
  private generateMsgpackConverter(name: string, type: ObjectSchema): string {
    const typename = capitalize(camelCase(type.__typename as string));

    const checkerSyntax = () => {
      const statements: string[] = [];
      for (const [key] of Object.entries(type)) {
        if (key === '__typename') continue;
        statements.push(
          `  if (o.${key} === undefined) { throw new Error("Expected ${key} in ${typename} deserialization"); }`,
        );
      }
      return statements.join('\n');
    };

    // TODO should we always just call constructor?
    const constructorBodySyntax = () => {
      const statements: string[] = [];
      for (const [key, value] of Object.entries(type)) {
        if (key === '__typename') continue;
        statements.push(`  ${msgpackConverterExpr(this.getTypeInfo(value), `o.${key}`)},`);
      }
      return statements.join('\n');
    };

    const callSyntax = () => {
      // return `${name}.from({\n${objectBodySyntax()}})`;
      return `new ${name}(\n${constructorBodySyntax()})`;
    };

    return `export function to${name}(o: Msgpack${name}): ${name} {
${checkerSyntax()};
return ${callSyntax.call(this)};
}`;
  }

  /**
   * Generate conversion method 'fromName' for a specific type 'name'.
   * @param name - The class name.
   * @param type - The object schema with properties of the interface.
   * @returns the fromName method string.
   */
  private generateClassConverter(name: string, type: ObjectSchema): string {
    const typename = capitalize(camelCase(type.__typename as string));

    const checkerSyntax = () => {
      const statements: string[] = [];
      for (const [key] of Object.entries(type)) {
        if (key === '__typename') continue;
        statements.push(
          `  if (o.${camelCase(key)} === undefined) { throw new Error("Expected ${camelCase(
            key,
          )} in ${typename} serialization"); }`,
        );
      }
      return statements.join('\n');
    };
    const bodySyntax = () => {
      const statements: string[] = [];
      for (const [key, value] of Object.entries(type)) {
        if (key === '__typename') continue;
        statements.push(`  ${key}: ${classConverterExpr(this.getTypeInfo(value), `o.${camelCase(key)}`)},`);
      }
      return statements.join('\n');
    };

    const callSyntax = () => {
      return `{\n${bodySyntax()}}`;
    };

    return `export function from${name}(o: ${name}): Msgpack${name} {
${checkerSyntax()};
return ${callSyntax.call(this)};
}`;
  }
  /**
   * Process a cbind schema.
   * @param name - The cbind name.
   * @param cbind - The cbind schema.
   * @returns The compiled schema.
   */
  processCbind(
    name: string,
    cbind: {
      /**
       * An array of Schema representing the argument types for a cbind function.
       */
      args: ['tuple', Schema[]];
      /**
       * The returned value's schema after processing the cbind.
       */
      ret: Schema;
    },
  ) {
    const [_tuple, args] = cbind.args;
    const typeInfos = args.map(arg => this.getTypeInfo(arg));
    const argStrings = typeInfos.map((typeInfo, i) => `arg${i}: ${typeInfo.typeName}`);
    const callStrings = typeInfos.map((typeInfo, i) => `${classConverterExpr(typeInfo, `arg${i}`)}`);
    const innerCall = `callCbind(wasm, '${name}', [${callStrings.join(', ')}])`;
    const retType = this.getTypeInfo(cbind.ret);
    this.funcDecls.push(`export function ${camelCase(name)}(wasm: IWasmModule, ${argStrings.join(', ')}): ${
      retType.typeName
    } {
return ${msgpackConverterExpr(retType, innerCall)};
}`);
  }

  /**
   * Compile the generated TypeScript code from processed cbind schemas into a single string.
   * The output string consists of necessary imports, type declarations, and helper methods
   * for serialization and deserialization between TypeScript classes and Msgpack format,
   * as well as the compiled cbind function calls.
   *
   * @returns A string containing the complete compiled TypeScript code.
   */
  compile(): string {
    const imports: string[] = [];
    const outputs: string[] = [
      `
/* eslint-disable */
// GENERATED FILE DO NOT EDIT, RUN yarn remake-bindings
import { Buffer } from "buffer";
import { callCbind } from './cbind.js';
import { IWasmModule } from '@aztec/foundation/wasm';
`,
    ];
    for (const typeInfo of Object.values(this.typeInfos)) {
      if (typeInfo.isImport) {
        imports.push(typeInfo.typeName);
      }
      if (typeInfo.usedInDiscriminatedVariant) {
        imports.push(`is${typeInfo.typeName}`);
      }
      if (typeInfo.declaration) {
        outputs.push(typeInfo.declaration);
        outputs.push('\n');
      }
      if (typeInfo.toClassMethod) {
        outputs.push(typeInfo.toClassMethod);
        outputs.push('\n');
      }
      if (typeInfo.fromClassMethod) {
        outputs.push(typeInfo.fromClassMethod);
        outputs.push('\n');
      }
    }

    outputs[0] += `
import {toBuffer, ${imports.join(', ')}} from './types.js';
import {Tuple, mapTuple} from '@aztec/foundation/serialize';
import mapValues from 'lodash.mapvalues';
       `;

    for (const funcDecl of Object.values(this.funcDecls)) {
      outputs.push(funcDecl);
    }
    return outputs.join('\n');
  }
}
