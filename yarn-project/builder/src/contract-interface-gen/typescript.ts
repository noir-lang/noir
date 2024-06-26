import {
  type ABIParameter,
  type ContractArtifact,
  type FunctionArtifact,
  getDefaultInitializer,
  isAztecAddressStruct,
  isEthAddressStruct,
  isFunctionSelectorStruct,
  isWrappedFieldStruct,
} from '@aztec/foundation/abi';

/**
 * Returns the corresponding typescript type for a given Noir type.
 * @param type - The input Noir type.
 * @returns An equivalent typescript type.
 */
function abiTypeToTypescript(type: ABIParameter['type']): string {
  switch (type.kind) {
    case 'field':
      return 'FieldLike';
    case 'boolean':
      return 'boolean';
    case 'integer':
      return '(bigint | number)';
    case 'string':
      return 'string';
    case 'array':
      return `${abiTypeToTypescript(type.type)}[]`;
    case 'struct':
      if (isEthAddressStruct(type)) {
        return 'EthAddressLike';
      }
      if (isAztecAddressStruct(type)) {
        return 'AztecAddressLike';
      }
      if (isFunctionSelectorStruct(type)) {
        return 'FunctionSelectorLike';
      }
      if (isWrappedFieldStruct(type)) {
        return 'WrappedFieldLike';
      }
      return `{ ${type.fields.map(f => `${f.name}: ${abiTypeToTypescript(f.type)}`).join(', ')} }`;
    default:
      throw new Error(`Unknown type ${type}`);
  }
}

/**
 * Generates the typescript code to represent a Noir parameter.
 * @param param - A Noir parameter with name and type.
 * @returns The corresponding ts code.
 */
function generateParameter(param: ABIParameter) {
  return `${param.name}: ${abiTypeToTypescript(param.type)}`;
}

/**
 * Generates the typescript code to represent a Noir function as a type.
 * @param param - A Noir function.
 * @returns The corresponding ts code.
 */
function generateMethod(entry: FunctionArtifact) {
  const args = entry.parameters.map(generateParameter).join(', ');
  return `
    /** ${entry.name}(${entry.parameters.map(p => `${p.name}: ${p.type.kind}`).join(', ')}) */
    ${entry.name}: ((${args}) => ContractFunctionInteraction) & Pick<ContractMethod, 'selector'>;`;
}

/**
 * Generates a deploy method for this contract.
 * @param input - Build artifact of the contract.
 * @returns A type-safe deploy method in ts.
 */
function generateDeploy(input: ContractArtifact) {
  const ctor = getDefaultInitializer(input);
  const args = (ctor?.parameters ?? []).map(generateParameter).join(', ');
  const contractName = `${input.name}Contract`;
  const artifactName = `${contractName}Artifact`;

  return `
  /**
   * Creates a tx to deploy a new instance of this contract.
   */
  public static deploy(wallet: Wallet, ${args}) {
    return new DeployMethod<${contractName}>(Fr.ZERO, wallet, ${artifactName}, ${contractName}.at, Array.from(arguments).slice(1));
  }

  /**
   * Creates a tx to deploy a new instance of this contract using the specified public keys hash to derive the address.
   */
  public static deployWithPublicKeysHash(publicKeysHash: Fr, wallet: Wallet, ${args}) {
    return new DeployMethod<${contractName}>(publicKeysHash, wallet, ${artifactName}, ${contractName}.at, Array.from(arguments).slice(2));
  }

  /**
   * Creates a tx to deploy a new instance of this contract using the specified constructor method.
   */
  public static deployWithOpts<M extends keyof ${contractName}['methods']>(
    opts: { publicKeysHash?: Fr; method?: M; wallet: Wallet },
    ...args: Parameters<${contractName}['methods'][M]>
  ) {
    return new DeployMethod<${contractName}>(
      opts.publicKeysHash ?? Fr.ZERO,
      opts.wallet,
      ${artifactName},
      ${contractName}.at,
      Array.from(arguments).slice(1),
      opts.method ?? 'constructor',
    );
  }
  `;
}

/**
 * Generates the constructor by supplying the ABI to the parent class so the user doesn't have to.
 * @param name - Name of the contract to derive the ABI name from.
 * @returns A constructor method.
 * @remarks The constructor is private because we want to force the user to use the create method.
 */
function generateConstructor(name: string) {
  return `
  private constructor(
    instance: ContractInstanceWithAddress,
    wallet: Wallet,
  ) {
    super(instance, ${name}ContractArtifact, wallet);
  }
  `;
}

/**
 * Generates the at method for this contract.
 * @param name - Name of the contract to derive the ABI name from.
 * @returns An at method.
 * @remarks We don't use constructor directly because of the async `wallet.getContractData` call.
 */
function generateAt(name: string) {
  return `
  /**
   * Creates a contract instance.
   * @param address - The deployed contract's address.
   * @param wallet - The wallet to use when interacting with the contract.
   * @returns A promise that resolves to a new Contract instance.
   */
  public static async at(
    address: AztecAddress,
    wallet: Wallet,
  ) {
    return Contract.at(address, ${name}Contract.artifact, wallet) as Promise<${name}Contract>;
  }`;
}

/**
 * Generates a static getter for the contract's artifact.
 * @param name - Name of the contract used to derive name of the artifact import.
 */
function generateArtifactGetter(name: string) {
  const artifactName = `${name}ContractArtifact`;
  return `
  /**
   * Returns this contract's artifact.
   */
  public static get artifact(): ContractArtifact {
    return ${artifactName};
  }
  `;
}

/**
 * Generates statements for importing the artifact from json and re-exporting it.
 * @param name - Name of the contract.
 * @param artifactImportPath - Path to load the ABI from.
 * @returns Code.
 */
function generateAbiStatement(name: string, artifactImportPath: string) {
  const stmts = [
    `import ${name}ContractArtifactJson from '${artifactImportPath}' assert { type: 'json' };`,
    `export const ${name}ContractArtifact = loadContractArtifact(${name}ContractArtifactJson as NoirCompiledContract);`,
  ];
  return stmts.join('\n');
}

/**
 * Generates a getter for the contract's storage layout.
 * @param input - The contract artifact.
 */
function generateStorageLayoutGetter(input: ContractArtifact) {
  const entries = Object.entries(input.storageLayout);

  if (entries.length === 0) {
    return '';
  }

  const storageFieldsUnionType = entries.map(([name]) => `'${name}'`).join(' | ');
  const layout = entries
    .map(
      ([name, { slot }]) =>
        `${name}: {
      slot: new Fr(${slot.toBigInt()}n),
    }`,
    )
    .join(',\n');

  return `public static get storage(): ContractStorageLayout<${storageFieldsUnionType}> {
      return {
        ${layout}
      } as ContractStorageLayout<${storageFieldsUnionType}>;
    }
    `;
}

/**
 * Generates a getter for the contract notes
 * @param input - The contract artifact.
 */
function generateNotesGetter(input: ContractArtifact) {
  const entries = Object.entries(input.notes);

  if (entries.length === 0) {
    return '';
  }

  const notesUnionType = entries.map(([name]) => `'${name}'`).join(' | ');
  const noteMetadata = entries
    .map(
      ([name, { id }]) =>
        `${name}: {
          id: new NoteSelector(${id.value}),
        }`,
    )
    .join(',\n');

  return `public static get notes(): ContractNotes<${notesUnionType}> {
    return {
      ${noteMetadata}
    } as ContractNotes<${notesUnionType}>;
  }
  `;
}

// events is of type AbiType
function generateEvents(events: any[] | undefined) {
  if (events === undefined) {
    return { events: '', eventDefs: '' };
  }

  const eventsMetadata = events.map(event => {
    const eventName = event.path.split('::').at(-1);

    const eventDefProps = event.fields.map((field: any) => `${field.name}: Fr`);
    const eventDef = `
      export type ${eventName} = {
        ${eventDefProps.join('\n')}
      }
    `;

    const fieldNames = event.fields.map((field: any) => `"${field.name}"`);
    const eventType = `${eventName}: {decode: (payload: L1EventPayload | undefined) => ${eventName} | undefined, eventSelector: EventSelector, fieldNames: string[] }`;

    const eventImpl = `${eventName}: {
        decode: this.decodeEvent(${event.fields.length}, EventSelector.fromSignature('${eventName}(${event.fields
      .map(() => 'Field')
      .join(',')})'), [${fieldNames}]),
      eventSelector: EventSelector.fromSignature('${eventName}(${event.fields.map(() => 'Field').join(',')})'),
      fieldNames: [${fieldNames}],
      }`;

    return {
      eventDef,
      eventType,
      eventImpl,
    };
  });

  return {
    eventDefs: eventsMetadata.map(({ eventDef }) => eventDef).join('\n'),
    events: `
    // Partial application is chosen is to avoid the duplication of so much codegen.
  private static decodeEvent<T>(fieldsLength: number, eventSelector: EventSelector, fields: string[]): (payload: L1EventPayload | undefined) => T | undefined {
    return (payload: L1EventPayload | undefined): T | undefined => {
      if (payload === undefined) {
        return undefined;
      }
      if (!eventSelector.equals(payload.eventTypeId)) {
        return undefined;
      }
      if (payload.event.items.length !== fieldsLength) {
        throw new Error(
          'Something is weird here, we have matching EventSelectors, but the actual payload has mismatched length',
        );
      }

      return fields.reduce(
        (acc, curr, i) => ({
          ...acc,
          [curr]: payload.event.items[i],
        }),
        {} as T,
      );
    };
  }

  public static get events(): { ${eventsMetadata.map(({ eventType }) => eventType).join(', ')} } {
    return {
      ${eventsMetadata.map(({ eventImpl }) => eventImpl).join(',\n')}
    };
  }
  `,
  };
}

/**
 * Generates the typescript code to represent a contract.
 * @param input - The compiled Noir artifact.
 * @param artifactImportPath - Optional path to import the artifact (if not set, will be required in the constructor).
 * @returns The corresponding ts code.
 */
export function generateTypescriptContractInterface(input: ContractArtifact, artifactImportPath?: string) {
  const methods = input.functions.filter(f => !f.isInternal).map(generateMethod);
  const deploy = artifactImportPath && generateDeploy(input);
  const ctor = artifactImportPath && generateConstructor(input.name);
  const at = artifactImportPath && generateAt(input.name);
  const artifactStatement = artifactImportPath && generateAbiStatement(input.name, artifactImportPath);
  const artifactGetter = artifactImportPath && generateArtifactGetter(input.name);
  const storageLayoutGetter = artifactImportPath && generateStorageLayoutGetter(input);
  const notesGetter = artifactImportPath && generateNotesGetter(input);
  const { eventDefs, events } = generateEvents(input.outputs.structs?.events);

  return `
/* Autogenerated file, do not edit! */

/* eslint-disable */
import {
  AztecAddress,
  AztecAddressLike,
  CompleteAddress,
  Contract,
  ContractArtifact,
  ContractBase,
  ContractFunctionInteraction,
  ContractInstanceWithAddress,
  ContractMethod,
  ContractStorageLayout,
  ContractNotes,
  DeployMethod,
  EthAddress,
  EthAddressLike,
  EventSelector,
  FieldLike,
  Fr,
  FunctionSelectorLike,
  L1EventPayload,
  loadContractArtifact,
  NoirCompiledContract,
  NoteSelector,
  Point,
  PublicKey,
  Wallet,
  WrappedFieldLike,
} from '@aztec/aztec.js';
${artifactStatement}

${eventDefs}

/**
 * Type-safe interface for contract ${input.name};
 */
export class ${input.name}Contract extends ContractBase {
  ${ctor}

  ${at}

  ${deploy}

  ${artifactGetter}

  ${storageLayoutGetter}

  ${notesGetter}

  /** Type-safe wrappers for the public methods exposed by the contract. */
  public override methods!: {
    ${methods.join('\n')}
  };

  ${events}
}
`;
}
