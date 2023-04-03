import { EthAddress } from '@aztec/foundation';
import { EthereumRpc, LogRequest } from '../eth_rpc/index.js';
import { AbiDataTypes, ContractAbi, ContractEntryDefinition, ContractFunctionEntry } from './abi/index.js';
import { EventLog } from './contract_tx_receipt.js';
import { FunctionInteraction } from './function_interaction.js';
import { ConstructorInteraction } from './constructor_interaction.js';

export interface ContractOptions {
  from?: EthAddress;
  gasPrice?: string | number;
  gas?: number;
}

interface ContractDefinition {
  methods: any;
  events?: any;
  eventLogs?: any;
}

type TxFactory = (...args: any[]) => FunctionInteraction;

type Events<T extends ContractDefinition | void> = T extends ContractDefinition
  ? Extract<keyof T['eventLogs'], string>
  : string;

type GetEventLog<T extends ContractDefinition | void, P extends Events<T>> = T extends ContractDefinition
  ? T['eventLogs'][P]
  : EventLog<any>;

type GetEvent<T extends ContractDefinition | void, P extends Events<T>> = T extends ContractDefinition
  ? T['events'][P]
  : any;

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
  public readonly methods: GetContractMethods<T>;
  // public readonly events: GetContractEvents<T>;
  private linkTable: { [name: string]: EthAddress } = {};

  constructor(
    private eth: EthereumRpc,
    private contractAbi: ContractAbi,
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
   *
   * @param data Contract bytecode as a hex string.
   * @param args Constructor arguments.
   * @returns
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

  public async getLogs<Event extends Events<T>>(
    event: Event,
    options: LogRequest<GetEvent<T, Event>>,
  ): Promise<GetEventLog<T, Event>[]>;
  public async getLogs(event: 'allevents', options: LogRequest): Promise<EventLog<any>[]>;
  public async getLogs(event: Events<T> & 'allevents', options: LogRequest = {}): Promise<EventLog<any>[]> {
    const logOptions = this.getLogOptions(event, options);
    const result = await this.eth.getLogs(logOptions);
    return result.map(log => this.contractAbi.decodeEvent(log));
  }

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

  // PRIVATE METHODS

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
