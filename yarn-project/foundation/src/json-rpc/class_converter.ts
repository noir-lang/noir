import { assert } from './js_utils.js';

/**
 * Represents a class compatible with our class conversion system.
 * E.g. PublicKey here satisfies 'StringIOClass'.
 * ```
 *    class PublicKey {
 *      toString() {
 *        return '...';
 *      }
 *      static fromString(str) {
 *        return new PublicKey(...);
 *      }
 *    }
 * ```
 */
interface StringIOClass {
  new (...args: any): any;

  // TODO(#4254): Ensure that toString method is checked for as well.

  /**
   * Creates an IOClass from a given string.
   */
  fromString: (str: string) => any;
}

/**
 * Represents a class compatible with our class conversion system.
 * E.g. PublicKey here satisfies 'ObjIOClass'.
 * ```
 *    class PublicKey {
 *      toJSON() {
 *        return {...};
 *      }
 *      static fromJSON(obj) {
 *        return new PublicKey({...});
 *      }
 *    }
 * ```
 */
interface ObjIOClass {
  new (...args: any): any;

  // TODO(#4254): Ensure that toJSON method is checked for as well.

  /**
   * Creates an IOClass from a given JSON object.
   */
  fromJSON: (str: object) => any;
}

/**
 * Either a StringIOClass or ObjIOClass
 */
type IOClass = ObjIOClass | StringIOClass;

/**
 * Registered classes available for conversion.
 */
export interface StringClassConverterInput {
  [className: string]: StringIOClass;
}

/**
 * Registered classes available for conversion.
 */
export interface JsonClassConverterInput {
  [className: string]: ObjIOClass;
}

/**
 * Represents a class in a JSON-friendly encoding.
 */
export interface StringEncodedClass {
  /**
   * The class type.
   */
  type: string;
  /**
   * The class data string.
   */
  data: string;
}

/**
 * Represents a class in a JSON-friendly encoding.
 */
export interface JsonEncodedClass {
  /**
   * The class type.
   */
  type: string;
  /**
   * The class data string.
   */
  data: object;
}
/**
 * Whether a class is a complex object or simply represented by a string.
 */
export type ClassEncoding = 'string' | 'object';

/**
 * Handles mapping of classes to names, and calling toString and fromString to convert to and from JSON-friendly formats.
 * Takes a class map as input.
 */
export class ClassConverter {
  private toClass = new Map<string, [IOClass, ClassEncoding]>();
  private toName = new Map<IOClass, [string, ClassEncoding]>();

  /**
   * Create a class converter from a table of classes.
   * @param stringClassMap - The class table of string encoded classes.
   * @param objectClassMap - The class table of complex object classes
   */
  constructor(stringClassMap?: StringClassConverterInput, objectClassMap?: JsonClassConverterInput) {
    if (stringClassMap) {
      for (const key of Object.keys(stringClassMap)) {
        this.register(key, stringClassMap[key], 'string');
      }
    }
    if (objectClassMap) {
      for (const key of Object.keys(objectClassMap)) {
        this.register(key, objectClassMap[key], 'object');
      }
    }
  }

  /**
   * Register a class with a certain name.
   * This name is used for conversion from and to this class.
   * @param type - The class name to use for serialization.
   * @param class_ - The class object.
   * @param encoding - Whether the class is a complex object or simply represented by a string.
   */
  register(type: string, class_: IOClass, encoding: ClassEncoding) {
    assert(type !== 'Buffer', "'Buffer' handling is hardcoded. Cannot use as name.");
    assert(
      class_.prototype['toString'] || class_.prototype['toJSON'],
      `Class ${type} must define a toString() OR toJSON() method.`,
    );
    assert(
      (class_ as StringIOClass)['fromString'] || (class_ as ObjIOClass)['fromJSON'],
      `Class ${type} must define a fromString() OR fromJSON() static method.`,
    );
    this.toName.set(class_, [type, encoding]);
    this.toClass.set(type, [class_, encoding]);
  }

  /**
   * Does this type name have a registered class?
   * @param type - The type name.
   * @returns If there's a registered class.
   */
  isRegisteredClassName(type: string) {
    return this.toClass.has(type);
  }
  /**
   * Is this class object registered?
   * @param obj - The class object.
   * @returns If it is a registered class.
   */
  isRegisteredClass(obj: any) {
    const name = obj.prototype.constructor.name;
    return this.toName.has(obj) || this.isRegisteredClassName(name);
  }
  /**
   * Convert a JSON-like object to a class object.
   * @param jsonObj - An object encoding a class.
   * @returns The class object.
   */
  toClassObj(jsonObj: JsonEncodedClass | StringEncodedClass): any {
    const result = this.toClass.get(jsonObj.type);
    assert(result, `Could not find type in lookup.`);

    const [class_, encoding] = result;
    if (encoding === 'string' && typeof jsonObj.data === 'string') {
      return (class_ as StringIOClass)!.fromString!(jsonObj.data);
    } else {
      return (class_ as ObjIOClass)!.fromJSON!(jsonObj.data as object);
    }
  }
  /**
   * Convert a class object to a JSON object.
   * @param classObj - A JSON encoding a class.
   * @returns The class object.
   */
  toJsonObj(classObj: any): JsonEncodedClass | StringEncodedClass {
    const { type, encoding } = this.lookupObject(classObj);
    const data = encoding === 'string' ? classObj.toString() : classObj.toJSON();
    return { type: type!, data };
  }

  /**
   * Loads the corresponding type for this class based on constructor first and constructor name if not found.
   * Constructor match works in the event of a minifier changing function names, and constructor name match
   * works in the event of duplicated instances of node modules being loaded (see #1826).
   * @param classObj - Object to lookup in the registered types.
   * @returns Registered type name and encoding.
   */
  private lookupObject(classObj: any) {
    const nameResult = this.toName.get(classObj.constructor);
    if (nameResult) {
      return { type: nameResult[0], encoding: nameResult[1] };
    }
    const classResult = this.toClass.get(classObj.constructor.name);
    if (classResult) {
      return { type: classObj.constructor.name, encoding: classResult[1] };
    }
    throw new Error(`Could not find class ${classObj.constructor.name} in lookup.`);
  }
}
