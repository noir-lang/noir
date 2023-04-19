// THIS IS GENERATED CODE, DO NOT EDIT!
/* eslint-disable */
import { EthAddress } from "@aztec/foundation";
import { EthereumRpc } from "../../eth_rpc/index.js";
import { Contract, ContractTxReceipt, EventLog, Options, TxCall, TxSend } from "../../contract/index.js";
import * as Bytes from "../../contract/bytes.js";
import abi from "./TestContractAbi.js";
/**
 * Type representing the 'ChangedEvent' that contains details about the change in balance,
 * such as the sender's address, the amount changed, and two timestamps for tracking.
 */
export type ChangedEvent = {
    /**
 * The Ethereum address initiating the event.
 */
from: EthAddress;
    /**
 * The amount involved in the transaction.
 */
amount: bigint;
    /**
 * Timestamp of the first event occurrence.
 */
t1: bigint;
    /**
 * Timestamp indicating the end time of an event.
 */
t2: bigint;
};
/**
 * Type representing the 'UnchangedEvent', which is emitted when a specific condition in the contract remains unchanged.
 */
export type UnchangedEvent = {
    /**
 * The monetary amount associated with the transaction.
 */
value: bigint;
    /**
 * The originating Ethereum address.
 */
addressFrom: EthAddress;
    /**
 * The timestamp of the first event.
 */
t1: bigint;
};
/**
 * Represents the ChangedEventLog interface for the TestContract.
 * Contains all the properties of a Changed event log, including event name and payload.
 */
export interface ChangedEventLog extends EventLog<ChangedEvent, "Changed"> {
}
/**
 * Represents the log interface for UnchangedEvent in the TestContract.
 * Provides event details such as value, addressFrom, and t1 from the emitted Unchanged event.
 */
export interface UnchangedEventLog extends EventLog<UnchangedEvent, "Unchanged"> {
}
/**
 * Represents the event types for the TestContract.
 * Provides a mapped definition of each event with its respective data structure.
 */
interface TestContractEvents {
    /**
 * Event emitted when a change occurs in the contract state.
 */
Changed: ChangedEvent;
    /**
 * An event indicating no change in the value.
 */
Unchanged: UnchangedEvent;
}
/**
 * Represents the event logs for the TestContract interface.
 * Contains the mapping of event names to their corresponding event log interfaces for easy access and organization.
 */
interface TestContractEventLogs {
    /**
 * Represents an event triggered when a change occurs in the contract state.
 */
Changed: ChangedEventLog;
    /**
 * Event triggered when a value remains unchanged.
 */
Unchanged: UnchangedEventLog;
}
/**
 * Represents the event logs for all transactions involving the TestContract.
 * Contains an organized collection of event logs for each specific event in the contract, allowing easy access to relevant transaction information.
 */
interface TestContractTxEventLogs {
    /**
 * Triggered when a state change occurs in the contract.
 */
Changed: ChangedEventLog[];
    /**
 * An event representing unaltered data.
 */
Unchanged: UnchangedEventLog[];
}
/**
 * Represents a TestContract transaction receipt.
 * Provides details about the transaction events, status, and other relevant information after executing a method call on the TestContract.
 */
export interface TestContractTransactionReceipt extends ContractTxReceipt<TestContractTxEventLogs> {
}
/**
 * Represents the methods available in the TestContract.
 * Provides functionality for interacting with and invoking smart contract functions,
 * handling various use cases such as adding structs, managing balances, performing transactions,
 * and working with event logs.
 */
interface TestContractMethods {
    addStruct(nestedStruct: {
        /**
 * Represents the active state of a specific object or process.
 */
status: boolean;
    }): TxSend<TestContractTransactionReceipt>;
    listOfNestedStructs(a0: EthAddress): TxCall<{
        /**
 * Represents the active state of an entity.
 */
status: boolean;
    }>;
    balance(who: EthAddress): TxCall<bigint>;
    hasALotOfParams(_var1: number, _var2: string, _var3: Bytes.Bytes32[]): TxSend<TestContractTransactionReceipt, EthAddress>;
    getStr(): TxCall<string>;
    owner(): TxCall<EthAddress>;
    mySend(to: EthAddress, value: bigint): TxSend<TestContractTransactionReceipt>;
    myDisallowedSend(to: EthAddress, value: bigint): TxSend<TestContractTransactionReceipt>;
    testArr(value: bigint[]): TxCall<bigint>;
    overloadedFunction(a: bigint): TxCall<bigint>;
    overloadedFunction(): TxCall<bigint>;
}
/**
 * Represents a TestContract definition interface.
 * Contains methods, events, and eventLogs related to the TestContract.
 * Provides functionality for interacting with the TestContract on the Ethereum network.
 */
export interface TestContractDefinition {
    /**
 * Collection of smart contract methods.
 */
methods: TestContractMethods;
    /**
 * A collection of event definitions for the TestContract.
 */
events: TestContractEvents;
    /**
 * Collection of logs for emitted events.
 */
eventLogs: TestContractEventLogs;
}
/**
 * The TestContract class represents a smart contract deployed on the Ethereum blockchain.
 * This class provides methods to interact with the contract, including invoking its functions,
 * querying its state, and listening for events. It extends the Contract base class and implements
 * the TestContractDefinition interface, which defines the structure of the contract's ABI.
 * Instances of this class can be created with an optional address and options, allowing the user
 * to easily connect to existing contracts or deploy new ones.
 */
export class TestContract extends Contract<TestContractDefinition> {
    constructor(eth: EthereumRpc, address?: EthAddress, options?: Options) {
        super(eth, abi, address, options);
    }
    /**
 * Deploy a new instance of the TestContract smart contract to the Ethereum network.
 * The 'deploy' function takes the initial 'who' address and 'myValue' as arguments for the constructor of the TestContract.
 * Returns a transaction receipt containing the contract address, gas used, and other details on successful deployment.
 *
 * @param who - The Ethereum address that will be set as the 'owner' of the newly deployed contract.
 * @param myValue - The initial value (in bigint) to be set in the smart contract's internal state.
 * @returns A promise that resolves to a TestContractTransactionReceipt with information about the deployed contract.
 */
deploy(who: EthAddress, myValue: bigint): TxSend<TestContractTransactionReceipt> {
        return super.deployBytecode("0x01234567", who, myValue) as any;
    }
}
export var TestContractAbi = abi;
