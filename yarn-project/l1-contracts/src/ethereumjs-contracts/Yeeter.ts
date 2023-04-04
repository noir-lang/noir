// THIS IS GENERATED CODE, DO NOT EDIT!
/* eslint-disable */
import { EthAddress } from '@aztec/foundation';
import { EthereumRpc } from '@aztec/ethereum.js/eth_rpc';
import { Contract, ContractTxReceipt, EventLog, Options, TxCall, TxSend } from '@aztec/ethereum.js/contract';
import * as Bytes from '@aztec/ethereum.js/contract/bytes.js';
import abi from './YeeterAbi.js';
export type ContractDeploymentEvent = {
  aztecAddress: Bytes.Bytes32;
  portalAddress: EthAddress;
  acir: Bytes.Bytes;
};
export type YeetEvent = {
  blockNum: bigint;
  sender: EthAddress;
  blabber: Bytes.Bytes;
};
export interface ContractDeploymentEventLog extends EventLog<ContractDeploymentEvent, 'ContractDeployment'> {}
export interface YeetEventLog extends EventLog<YeetEvent, 'Yeet'> {}
interface YeeterEvents {
  ContractDeployment: ContractDeploymentEvent;
  Yeet: YeetEvent;
}
interface YeeterEventLogs {
  ContractDeployment: ContractDeploymentEventLog;
  Yeet: YeetEventLog;
}
interface YeeterTxEventLogs {
  ContractDeployment: ContractDeploymentEventLog[];
  Yeet: YeetEventLog[];
}
export interface YeeterTransactionReceipt extends ContractTxReceipt<YeeterTxEventLogs> {}
interface YeeterMethods {
  yeet(_blockNum: bigint, _blabber: Bytes.Bytes): TxSend<YeeterTransactionReceipt>;
  yeetContractDeployment(
    _aztecAddress: Bytes.Bytes32,
    _portalAddress: EthAddress,
    _acir: Bytes.Bytes,
  ): TxSend<YeeterTransactionReceipt>;
}
export interface YeeterDefinition {
  methods: YeeterMethods;
  events: YeeterEvents;
  eventLogs: YeeterEventLogs;
}
export class Yeeter extends Contract<YeeterDefinition> {
  constructor(eth: EthereumRpc, address?: EthAddress, options?: Options) {
    super(eth, abi, address, options);
  }
  deploy(): TxSend<YeeterTransactionReceipt> {
    return super.deployBytecode(
      '0x608060405234801561001057600080fd5b50610258806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c80634bcfab1e1461003b5780639ac5c19314610050575b600080fd5b61004e61004936600461013f565b610063565b005b61004e61005e3660046101a7565b6100ad565b826001600160a01b0316847f30735f52bf1ff4d542583f894902a2775489f644723a4b3cca1de6d6eabd0c4a848460405161009f9291906101f3565b60405180910390a350505050565b336001600160a01b0316837f6b104311c3f2fcdacda72ada5cd1341c211de62acdedc2e26b94d3cc8aaea2bb84846040516100e99291906101f3565b60405180910390a3505050565b60008083601f84011261010857600080fd5b50813567ffffffffffffffff81111561012057600080fd5b60208301915083602082850101111561013857600080fd5b9250929050565b6000806000806060858703121561015557600080fd5b8435935060208501356001600160a01b038116811461017357600080fd5b9250604085013567ffffffffffffffff81111561018f57600080fd5b61019b878288016100f6565b95989497509550505050565b6000806000604084860312156101bc57600080fd5b83359250602084013567ffffffffffffffff8111156101da57600080fd5b6101e6868287016100f6565b9497909650939450505050565b60208152816020820152818360408301376000818301604090810191909152601f909201601f1916010191905056fea2646970667358221220b1bbb4c30aab6249a2552fe4b484e7f71877415dcac9acaf18fa7f273354cd4064736f6c63430008120033',
    ) as any;
  }
}
export var YeeterAbi = abi;
