import {
  type AbiTypeWithGenerics,
  type ArrayType,
  BindingId,
  type Constant,
  type StringType,
  type Struct,
  type StructType,
  type Tuple,
  findAllStructsInType,
  findStructsInType,
} from './abi_type_with_generics.js';

export interface DemonomorphizerConfig {
  leaveArrayLengthsUnbounded: boolean;
  leaveStringLengthsUnbounded: boolean;
}

/**
 * Demonomorphizes a vector of ABI types adding generics to structs.
 * Since monomorphization of the generics destroys information, this process is not guaranteed to return the original structure.
 * However, it should successfully unify all struct types that share the same name and field names.
 */
export class Demonomorphizer {
  private variantsMap: Map<string, Struct[]>;
  private visitedStructs: Map<string, StructType>;
  private lastBindingId = 0;

  /**
   * Demonomorphizes the passed in ABI types, mutating them.
   */
  public static demonomorphize(abiTypes: AbiTypeWithGenerics[], config: DemonomorphizerConfig) {
    new Demonomorphizer(abiTypes, config);
  }

  private constructor(
    private types: AbiTypeWithGenerics[],
    private config: DemonomorphizerConfig,
  ) {
    this.variantsMap = new Map<string, Struct[]>();
    this.fillVariantsMap();

    this.visitedStructs = new Map<string, StructType>();
    this.demonomorphizeStructs();
  }

  /**
   * Finds all the variants of the structs in the types.
   * A variant is every use of a struct with the same name and fields.
   */
  private fillVariantsMap() {
    const allStructs = this.types.flatMap(findAllStructsInType);
    for (const struct of allStructs) {
      const id = Demonomorphizer.buildIdForStruct(struct.structType);
      const variants = this.variantsMap.get(id) ?? [];
      variants.push(struct);
      this.variantsMap.set(id, variants);
    }
  }

  private demonomorphizeStructs() {
    for (const type of this.types) {
      const topLevelStructs = findStructsInType(type);
      for (const struct of topLevelStructs) {
        this.demonomorphizeStruct(struct);
      }
    }
  }

  /**
   * Demonomorphizes a struct, by demonomorphizing its dependencies first.
   * Then it'll unify the types of the variants generating a unique generic type.
   * It'll also generate args that instantiate the generic type with the concrete arguments for each variant.
   */
  private demonomorphizeStruct(struct: Struct) {
    const id = Demonomorphizer.buildIdForStruct(struct.structType);
    if (this.visitedStructs.has(id)) {
      return;
    }
    const dependencies = struct.structType.fields.flatMap((field) => findStructsInType(field.type));
    for (const dependency of dependencies) {
      this.demonomorphizeStruct(dependency);
    }
    if (this.visitedStructs.has(id)) {
      throw new Error('Circular dependency detected');
    }

    const variants = this.variantsMap.get(id)!;
    const mappedStructType = struct.structType;

    for (let i = 0; i < struct.structType.fields.length; i++) {
      const variantTypes = variants.map((variant) => variant.structType.fields[i].type);
      const mappedType = this.unifyTypes(variantTypes, mappedStructType.generics, variants);
      mappedStructType.fields[i].type = mappedType;
    }

    // Mutate variants setting the new struct type
    variants.forEach((variant) => (variant.structType = mappedStructType));

    this.visitedStructs.set(id, mappedStructType);
  }

  /**
   * Tries to unify the types of a set of variants recursively.
   * Unification will imply replacing some properties with bindings and pushing bindings to the generics of the struct.
   */
  private unifyTypes(
    types: AbiTypeWithGenerics[],
    generics: BindingId[], // Mutates generics adding new bindings
    variants: Struct[], // mutates variants adding different args to the variants
  ): AbiTypeWithGenerics {
    const kinds = new Set(types.map((type) => type.kind));
    if (kinds.size > 1) {
      return this.buildBindingAndPushToVariants(types, generics, variants);
    }
    switch (types[0].kind) {
      case 'field':
      case 'boolean':
      case 'binding':
        return types[0];
      case 'integer': {
        if (allDeepEqual(types)) {
          return types[0];
        } else {
          return this.buildBindingAndPushToVariants(types, generics, variants);
        }
      }
      case 'string': {
        const strings = types as StringType[];
        const unifiedStringType = strings[0];
        if (strings.every((string) => string.length === unifiedStringType.length)) {
          return unifiedStringType;
        } else if (!this.config.leaveStringLengthsUnbounded) {
          unifiedStringType.length = this.buildNumericBindingAndPushToVariants(
            strings.map((string) => {
              if (typeof string.length !== 'number') {
                throw new Error('Trying to unify strings with bindings');
              }
              return string.length;
            }),
            generics,
            variants,
          );
          return unifiedStringType;
        } else {
          unifiedStringType.length = null;
          return unifiedStringType;
        }
      }
      case 'array': {
        const arrays = types as ArrayType[];
        const unifiedArrayType: ArrayType = arrays[0];
        if (!arrays.every((array) => array.length === unifiedArrayType.length)) {
          if (!this.config.leaveArrayLengthsUnbounded) {
            unifiedArrayType.length = this.buildNumericBindingAndPushToVariants(
              arrays.map((array) => {
                if (typeof array.length !== 'number') {
                  throw new Error('Trying to unify arrays with bindings');
                }
                return array.length;
              }),
              generics,
              variants,
            );
          } else {
            unifiedArrayType.length = null;
          }
        }

        unifiedArrayType.type = this.unifyTypes(
          arrays.map((array) => array.type),
          generics,
          variants,
        );
        return unifiedArrayType;
      }
      case 'tuple': {
        const tuples = types as Tuple[];
        const unifiedTupleType: Tuple = tuples[0];
        for (let i = 0; i < unifiedTupleType.fields.length; i++) {
          unifiedTupleType.fields[i] = this.unifyTypes(
            tuples.map((tuple) => tuple.fields[i]),
            generics,
            variants,
          );
        }
        return unifiedTupleType;
      }
      case 'struct': {
        const structs = types as Struct[];
        const ids = new Set(structs.map((struct) => Demonomorphizer.buildIdForStruct(struct.structType)));
        if (ids.size > 1) {
          // If the types are different structs, we can only unify them by creating a new binding.
          // For example, if we have a struct A { x: u32 } and a struct A { x: Field }, the only possible unification is A<T> { x: T }
          return this.buildBindingAndPushToVariants(types, generics, variants);
        } else {
          // If the types are the same struct, we must unify the arguments to the struct.
          // For example, if we have A<Field> and A<u32>, we need to unify to A<T> and push T to the generics of the struct type.
          const unifiedStruct = structs[0];

          if (!structs.every((struct) => struct.args.length === unifiedStruct.args.length)) {
            throw new Error('Same struct with different number of args encountered');
          }
          for (let i = 0; i < unifiedStruct.args.length; i++) {
            const argTypes = structs.map((struct) => struct.args[i]);
            unifiedStruct.args[i] = this.unifyTypes(argTypes, generics, variants);
          }
          return unifiedStruct;
        }
      }

      case 'constant': {
        const constants = types as Constant[];
        if (constants.every((constant) => constant.value === constants[0].value)) {
          return constants[0];
        } else {
          return this.buildBindingAndPushToVariants(types, generics, variants, true);
        }
      }

      default: {
        const exhaustiveCheck: never = types[0];
        throw new Error(`Unhandled abi type: ${exhaustiveCheck}`);
      }
    }
  }

  /**
   * We consider a struct to be the same if it has the same name and field names.
   * Structs with the same id will be unified into a single type by the demonomorphizer.
   */
  public static buildIdForStruct(struct: StructType): string {
    const name = struct.path.split('::').pop()!;
    const fields = struct.fields.map((field) => field.name).join(',');
    return `${name}(${fields})`;
  }

  private buildBindingAndPushToVariants(
    concreteTypes: AbiTypeWithGenerics[],
    generics: BindingId[],
    variants: Struct[],
    isNumeric = false,
  ): AbiTypeWithGenerics {
    const bindingId = new BindingId(this.lastBindingId++, isNumeric);

    for (let i = 0; i < variants.length; i++) {
      const variant = variants[i];
      const concreteType = concreteTypes[i];
      variant.args.push(concreteType);
    }

    generics.push(bindingId);
    return { kind: 'binding', id: bindingId };
  }

  private buildNumericBindingAndPushToVariants(
    concreteNumbers: number[],
    generics: BindingId[],
    variants: Struct[],
  ): BindingId {
    const bindingId = new BindingId(this.lastBindingId++, true);

    for (let i = 0; i < variants.length; i++) {
      const variant = variants[i];
      variant.args.push({ kind: 'constant', value: concreteNumbers[i] });
    }

    generics.push(bindingId);
    return bindingId;
  }
}

function allDeepEqual<T>(arr: T[]): boolean {
  if (arr.length === 0) {
    return true;
  }
  const first = JSON.stringify(arr[0]);
  for (let i = 0; i < arr.length; i++) {
    if (JSON.stringify(arr[i]) !== first) {
      return false;
    }
  }
  return true;
}
