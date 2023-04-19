import { EthAddress } from '@aztec/foundation';
import { EthereumRpc, LogRequest } from '../eth_rpc/index.js';
import { AbiDataTypes, ContractAbi, ContractEntryDefinition, ContractFunctionEntry } from './abi/index.js';
import { EventLog } from './contract_tx_receipt.js';
import { FunctionInteraction } from './function_interaction.js';
import { ConstructorInteraction } from './constructor_interaction.js';

/**
 * Represents configuration options for interacting with an Ethereum contract.
 * Provides optional settings for specifying the sender address, gas price, and gas limit when creating contract transactions.
 */
export interface ContractOptions {
  /**
   * The Ethereum address initiating the contract interaction.
   */
  from?: EthAddress;
  /**
   * Gas price for executing contract transactions.
   */
  gasPrice?: string | number;
  /**
   * The maximum amount of gas units allowed for the contract execution.
   */
  gas?: number;
}

/**
 * Represents a contract definition for interacting with Ethereum smart contracts.
 * Provides a structure to define methods, events, and event logs associated with a specific contract.
 * Enables type safety when calling contract methods, accessing event logs, and return values.
 */
interface ContractDefinition {
  /**
   * Collection of named functions to interact with the contract methods.
   */
  methods: any;
  /**
   * Collection of contract event definitions for ease of interaction.
   */
  events?: any;
  /**
   * A collection of event logs for the contract.
   */
  eventLogs?: any;
}

/**
 * TxFactory is a type representing a factory function that produces FunctionInteraction instances.
 * It takes any number of arguments and returns a FunctionInteraction instance for interacting with
 * the smart contract methods based on the provided arguments.
 */
type TxFactory = (...args: any[]) => FunctionInteraction;

/**
 * Type representing the names of the events present in a given contract definition.
 * Used for accessing event logs and interacting with specific events on the contract.
 */
type Events<T extends ContractDefinition | void> = T extends ContractDefinition
  ? Extract<keyof T['eventLogs'], string>
  : string;

/**
 * Type representing the event log for a specific event in a contract definition.
 * Extracts the event log type based on the given contract definition and event name.
 */
type GetEventLog<T extends ContractDefinition | void, P extends Events<T>> = T extends ContractDefinition
  ? T['eventLogs'][P]
  : EventLog<any>;

/**
 * GetEvent type represents a contract event type from the given ContractDefinition.
 * Used to extract appropriate event information for a specific event within the contract.
 */
type GetEvent<T extends ContractDefinition | void, P extends Events<T>> = T extends ContractDefinition
  ? T['events'][P]
  : any;

/**
 * Type representing the contract methods available for interaction.
 * It extracts the 'methods' property from the given ContractDefinition type parameter,
 * providing a mapping of method names to their respective FunctionInteraction instances.
 */
type GetContractMethods<T> = T extends ContractDefinition
  ? T['methods']
  : { [key: string]: (...args: any[]) => FunctionInteraction };

/**
 * Provides a class to interact with a contract.
 *
 * Given an ABI, it generates a collection of named functions on the public `methods` property.
 *
 * Can take a `ContractDefinition` as a type parameter to provide full type safety when calling methods, accessing
 * event logs, return values, etc.
 * The `gen_def` tool will generate this definiton and then extend this class to use it, giving the user the ability
 * to just use that class directly.
 *
 * Default options can be provided, these can be used to save having on to specify e.g. `from` and `gas` on every call.
 */
export class Contract<T extends ContractDefinition | void = void> {
  /**
   * Collection of named functions for interacting with the contract methods.
   */
  public readonly methods: GetContractMethods<T>;
  // public readonly events: GetContractEvents<T>;
  private linkTable: { [name: string]: EthAddress } = {};

  constructor(
    private eth: EthereumRpc,
    private contractAbi: ContractAbi,
    /**
     * Ethereum contract address for interaction.
     */
    public address = EthAddress.ZERO,
    private defaultOptions: ContractOptions = {},
  ) {
    this.methods = this.buildMethods();
    // this.events = this.buildEvents();
  }

  /**
   * When deploying bytecode that needs to be linked first, call this function to establish the link name to address
   * mapping. When `deployBytecode` is called it will perform the substitutions first.
   */
  public link(name: string, address: EthAddress) {
    this.linkTable[name] = address;
  }

  /**
   * DeployBytecode.
   * @param data - Contract bytecode as a hex string.
   * @param args - Constructor arguments.
   * @returns ConstructorInteraction.
   */
  public deployBytecode(data: string, ...args: any[]) {
    const linkedData = Object.entries(this.linkTable).reduce(
      (data, [name, address]) =>
        data.replace(new RegExp(`_+${name}_+`, 'gi'), address.toString().slice(2).toLowerCase()),
      data,
    );

    if (linkedData.includes('_')) {
      throw new Error('Bytecode has not been fully linked.');
    }

    return new ConstructorInteraction(
      this.eth,
      this.contractAbi.ctor,
      this.contractAbi,
      Buffer.from(linkedData.slice(2), 'hex'),
      args,
      this.defaultOptions,
      addr => (this.address = addr),
    );
  }

  /**
   * Retrieves event logs from the contract based on the specified event and options.
   * If 'allevents' is passed as the event, it will return logs for all events in the contract.
   * Otherwise, it returns logs for the specific event name or signature provided.
   * The LogRequest options allow filtering, such as setting a block range or topics to search for logs.
   *
   * @param event - The event name, signature, or 'allevents' to retrieve logs for.
   * @param options - Optional LogRequest object to filter the log results.
   * @returns An array of EventLog objects for the specified event(s) and filtered based on the provided options.
   */
  public async getLogs<Event extends Events<T>>(
    event: Event,
    options: LogRequest<GetEvent<T, Event>>,
  ): Promise<GetEventLog<T, Event>[]>;
  /**
   * Retrieves event logs for the specified event or all events emitted by the contract.
   * This function takes an event name and optional log request options as parameters, then
   * fetches the matching event logs from the Ethereum blockchain. If the event name is 'allevents',
   * it will retrieve logs for all events emitted by the contract. The resulting event logs are
   * decoded according to the contract's ABI before being returned as an array.
   *
   * @param event - The name of the event or 'allevents' to retrieve logs for all events.
   * @param options - Optional log request options such as filter, address, and topics.
   * @returns An array of decoded event logs matching the specified event name and options.
   */
  public async getLogs(event: 'allevents', options: LogRequest): Promise<EventLog<any>[]>;
  /**
   * Fetches event logs from the blockchain based on the given event and options.
   * This function can either retrieve logs for a specific event or all events in a contract.
   * It returns an array of decoded event logs based on the ContractDefinition type parameter.
   * The `eventName` parameter should be the name or signature of the event, or 'allevents' to fetch
   * logs for all events in a contract. The `options` parameter allows filtering logs by block range,
   * address, and other criteria.
   *
   * @param eventName - The name, signature, or 'allevents' string representing the event(s) to fetch logs for.
   * @param options - A LogRequest object with optional properties to filter event logs.
   * @returns A Promise that resolves to an array of decoded event logs.
   */
  public async getLogs(event: Events<T> & 'allevents', options: LogRequest = {}): Promise<EventLog<any>[]> {
    const logOptions = this.getLogOptions(event, options);
    const result = await this.eth.getLogs(logOptions);
    return result.map(log => this.contractAbi.decodeEvent(log));
  }

  /**
   * Retrieves a contract method by name and input/output types as an executor factory.
   * The method can be called with the specified arguments to create a FunctionInteraction instance.
   * Throws an error if no contract address is available or if there is no matching method with the provided arguments length.
   *
   * @param name - The name of the contract method.
   * @param inputTypes - An array of input data types for the method.
   * @param outputTypes - An array of output data types for the method.
   * @returns A TxFactory instance representing the contract method.
   */
  public getMethod(name: string, inputTypes: AbiDataTypes[], outputTypes: AbiDataTypes[]) {
    const abiEntry: ContractEntryDefinition = {
      inputs: inputTypes.map((type, i) => ({ name: `a${i}`, type })),
      name,
      outputs: outputTypes.map((type, i) => ({ name: `a${i}`, type })),
      stateMutability: 'nonpayable',
      type: 'function',
    };
    return this.executorFactory([new ContractFunctionEntry(abiEntry)]);
  }

  /**
   * PRIVATE METHODS.
   */

  // eslint-disable-next-line jsdoc/require-jsdoc
  private executorFactory(functions: ContractFunctionEntry[]): TxFactory {
    return (...args: any[]): FunctionInteraction => {
      if (this.address.equals(EthAddress.ZERO)) {
        throw new Error('No contract address.');
      }

      const firstMatchingOverload = functions.find(f => args.length === f.numArgs());

      if (!firstMatchingOverload) {
        throw new Error(`No matching method with ${args.length} arguments for ${functions[0].name}.`);
      }

      return new FunctionInteraction(
        this.eth,
        firstMatchingOverload,
        this.contractAbi,
        this.address,
        args,
        this.defaultOptions,
      );
    };
  }

  /**
   * Generates a collection of named functions on the public `methods` property based on the contract ABI.
   * It groups and assigns contract functions to their respective method names.
   * In case of function overloads, it will create an executor factory for all matching functions.
   *
   *
   * @returns An object containing the generated methods mapped to their respective names.
   */
  private buildMethods() {
    const methods: any = {};

    this.contractAbi.functions.forEach(f => {
      const executor = this.executorFactory([f]);
      methods[f.asString()] = executor;
      methods[f.signature] = executor;
    });

    const grouped = this.contractAbi.functions.reduce((acc, method) => {
      const funcs = [...(acc[method.name!] || []), method];
      return { ...acc, [method.name!]: funcs };
    }, {} as { [name: string]: ContractFunctionEntry[] });

    Object.entries(grouped).map(([name, funcs]) => {
      methods[name] = this.executorFactory(funcs);
    });

    return methods;
  }

  /**
   * Generates a LogRequest object for the specified event and request options.
   * This is used to filter and retrieve logs related to a contract event.
   * Throws an error if no contract address is available or the specified event is not found in the ABI.
   *
   * @param eventName - The name or signature of the contract event.
   * @param options - A LogRequest object containing filter and topic options for the log query.
   * @returns A LogRequest object with the specified event and request options combined.
   */
  private getLogOptions(eventName = 'allevents', options: LogRequest): LogRequest {
    if (!this.address) {
      throw new Error('No contract address.');
    }

    if (eventName.toLowerCase() === 'allevents') {
      return {
        ...options,
        address: this.address,
      };
    }

    const event = this.contractAbi.events.find(
      e => e.name === eventName || e.signature === '0x' + eventName.replace('0x', ''),
    );

    if (!event) {
      throw new Error(`Event ${eventName} not found.`);
    }

    return {
      ...options,
      address: this.address,
      topics: event.getEventTopics(options.filter),
    };
  }

  // This class used to provide the ability to register for events using subscriptions.
  // We don't need this functionality right now so have commented out.

  // export type EventSubscriptionFactory<Result = EventLog<any>> = (
  //   options?: object,
  //   callback?: (
  //     err: Error,
  //     result: Result,
  //     subscription: Subscription<Result>
  //   ) => void
  // ) => Subscription<Result>;
  // type GetContractEvents<T> = T extends ContractDefinition
  //   ? T['events'] & {
  //       allEvents: EventSubscriptionFactory<T['eventLogs'][Events<T>]>;
  //     }
  //   : { [key: string]: EventSubscriptionFactory };

  // public once<Event extends Events<T>>(
  //   event: Event,
  //   options: {
  //     filter?: object;
  //     topics?: string[];
  //   },
  //   callback: (err, res: GetEventLog<T, Event>, sub) => void,
  // );
  // public once(event: Events<T>, options: LogRequest, callback: (err, res, sub) => void): void {
  //   this.on(event, options, (err, res, sub) => {
  //     sub.unsubscribe();
  //     callback(err, res, sub);
  //   });
  // }

  // private buildEvents() {
  //   const events: any = {};

  //   this.contractAbi.events.forEach((e) => {
  //     const event = this.on.bind(this, e.signature!);

  //     if (!events[e.name!]) {
  //       events[e.name!] = event;
  //     }

  //     events[e.asString()] = event;
  //     events[e.signature] = event;
  //   });

  //   events.allEvents = this.on.bind(this, "allevents");

  //   return events;
  // }

  // private on(
  //   event: string,
  //   options: LogRequest = {},
  //   callback?: (err, res, sub) => void
  // ) {
  //   const logOptions = this.getLogOptions(event, options);
  //   const { fromBlock, ...subLogOptions } = logOptions;
  //   const params = [toRawLogRequest(subLogOptions)];

  //   const subscription = new Subscription<LogResponse, RawLogResponse>(
  //     "eth",
  //     "logs",
  //     params,
  //     this.eth.provider,
  //     (result, sub) => {
  //       const output = fromRawLogResponse(result);
  //       const eventLog = this.contractAbi.decodeEvent(output);
  //       sub.emit(output.removed ? "changed" : "data", eventLog);
  //       if (callback) {
  //         callback(undefined, eventLog, sub);
  //       }
  //     },
  //     false
  //   );

  //   subscription.on("error", (err) => {
  //     if (callback) {
  //       callback(err, undefined, subscription);
  //     }
  //   });

  //   if (fromBlock !== undefined) {
  //     this.eth
  //       .getPastLogs(logOptions)
  //       .then((logs) => {
  //         logs.forEach((result) => {
  //           const output = this.contractAbi.decodeEvent(result);
  //           subscription.emit("data", output);
  //         });
  //         subscription.subscribe();
  //       })
  //       .catch((err) => {
  //         subscription.emit("error", err);
  //       });
  //   } else {
  //     subscription.subscribe();
  //   }

  //   return subscription;
  // }
}
