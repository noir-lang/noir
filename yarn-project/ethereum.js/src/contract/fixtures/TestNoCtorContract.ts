// THIS IS GENERATED CODE, DO NOT EDIT!
/* eslint-disable */
import { EthAddress } from "@aztec/foundation";
import { EthereumRpc } from "../../eth_rpc/index.js";
import { Contract, ContractTxReceipt, EventLog, Options, TxCall, TxSend } from "../../contract/index.js";
import * as Bytes from "../../contract/bytes.js";
import abi from "./TestNoCtorContractAbi.js";
/**
 * Represents the events emitted by the TestNoCtorContract.
 * Provides an interface for accessing the event log data associated with the contract.
 */
interface TestNoCtorContractEvents {
}
/**
 * Represents the event logs for the TestNoCtorContract.
 * Provides an interface for accessing and managing the emitted events in the contract.
 */
interface TestNoCtorContractEventLogs {
}
/**
 * Represents the transaction event logs for the TestNoCtorContract.
 * Provides a structured interface for accessing event logs emitted during contract transactions.
 */
interface TestNoCtorContractTxEventLogs {
}
/**
 * Represents the transaction receipt for a TestNoCtorContract operation.
 * Contains detailed information about the transaction status and event logs specific to the TestNoCtorContract instance.
 */
export interface TestNoCtorContractTransactionReceipt extends ContractTxReceipt<TestNoCtorContractTxEventLogs> {
}
/**
 * Represents the TestNoCtorContract methods interface.
 * Provides a collection of methods for interacting with and executing transactions on the TestNoCtorContract smart contract.
 */
interface TestNoCtorContractMethods {
    addStruct(nestedStruct: {
        /**
 * Indicates the active state of an element.
 */
status: boolean;
    }): TxSend<TestNoCtorContractTransactionReceipt>;
}
/**
 * Represents a TestNoCtorContract definition.
 * Provides functionality for interacting with methods and events specific to the TestNoCtorContract smart contract.
 */
export interface TestNoCtorContractDefinition {
    /**
 * Collection of smart contract methods.
 */
methods: TestNoCtorContractMethods;
    /**
 * Collection of contract event definitions.
 */
events: TestNoCtorContractEvents;
    /**
 * Holds the event logs data for the contract.
 */
eventLogs: TestNoCtorContractEventLogs;
}
/**
 * The TestNoCtorContract class represents a smart contract on the Ethereum blockchain.
 * It provides methods to interact with the contract both for reading data and sending transactions.
 * The class also defines the structure of events emitted by the contract and facilitates listening to those events.
 * This class is particularly useful when working with contracts that do not have a constructor function.
 */
export class TestNoCtorContract extends Contract<TestNoCtorContractDefinition> {
    constructor(eth: EthereumRpc, address?: EthAddress, options?: Options) {
        super(eth, abi, address, options);
    }
    /**
 * Deploy the TestNoCtorContract smart contract to the Ethereum blockchain.
 * Uses a fixed bytecode "0x01234567" as the deployment data.
 * Returns a transaction object with the contract deployment details and receipt.
 *
 * @returns {TxSend<TestNoCtorContractTransactionReceipt>} A transaction object for the deployment of TestNoCtorContract.
 */
deploy(): TxSend<TestNoCtorContractTransactionReceipt> {
        return super.deployBytecode("0x01234567") as any;
    }
}
export var TestNoCtorContractAbi = abi;
