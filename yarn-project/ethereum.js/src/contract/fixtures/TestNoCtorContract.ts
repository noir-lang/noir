// THIS IS GENERATED CODE, DO NOT EDIT!
/* eslint-disable */
import { EthAddress } from "@aztec/foundation";
import { EthereumRpc } from "../../eth_rpc/index.js";
import { Contract, ContractTxReceipt, EventLog, Options, TxCall, TxSend } from "../../contract/index.js";
import * as Bytes from "../../contract/bytes.js";
import abi from "./TestNoCtorContractAbi.js";
interface TestNoCtorContractEvents {
}
interface TestNoCtorContractEventLogs {
}
interface TestNoCtorContractTxEventLogs {
}
export interface TestNoCtorContractTransactionReceipt extends ContractTxReceipt<TestNoCtorContractTxEventLogs> {
}
interface TestNoCtorContractMethods {
    addStruct(nestedStruct: {
        status: boolean;
    }): TxSend<TestNoCtorContractTransactionReceipt>;
}
export interface TestNoCtorContractDefinition {
    methods: TestNoCtorContractMethods;
    events: TestNoCtorContractEvents;
    eventLogs: TestNoCtorContractEventLogs;
}
export class TestNoCtorContract extends Contract<TestNoCtorContractDefinition> {
    constructor(eth: EthereumRpc, address?: EthAddress, options?: Options) {
        super(eth, abi, address, options);
    }
    deploy(): TxSend<TestNoCtorContractTransactionReceipt> {
        return super.deployBytecode("0x01234567") as any;
    }
}
export var TestNoCtorContractAbi = abi;
