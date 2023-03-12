import { LogResponse } from '../../eth_rpc/index.js';
import { hexToBuffer } from '../../hex_string/index.js';
import { EventLog } from '../contract_tx_receipt.js';
import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';
import { ContractEntry } from './contract_entry.js';

export class ContractEventEntry extends ContractEntry {
  public readonly signature: string;

  constructor(entry: ContractEntryDefinition) {
    super(entry);
    this.signature = abiCoder.encodeEventSignature(abiCoder.abiMethodToString(entry));
  }

  public getEventTopics(filter: object = {}) {
    const topics: (Buffer | Buffer[])[] = [];

    if (!this.entry.anonymous && this.signature) {
      topics.push(hexToBuffer(this.signature));
    }

    const indexedTopics = (this.entry.inputs || [])
      .filter(input => input.indexed === true)
      .map(input => {
        const value = filter[input.name];
        if (!value) {
          return null;
        }

        // TODO: https://github.com/ethereum/web3.js/issues/344
        // TODO: deal properly with components

        if (Array.isArray(value)) {
          return value.map(v => abiCoder.encodeParameter(input.type, v));
        } else {
          return abiCoder.encodeParameter(input.type, value);
        }
      });

    return [...topics, ...indexedTopics];
  }

  public decodeEvent(log: LogResponse): EventLog<any> {
    const { data = '', topics = [], ...formattedLog } = log;
    const { anonymous, inputs = [], name = '' } = this.entry;

    const argTopics = anonymous ? topics : topics.slice(1);
    const returnValues = abiCoder.decodeLog(inputs, data, argTopics);
    delete returnValues.__length__;

    return {
      ...formattedLog,
      event: name,
      args: returnValues,
      signature: anonymous || !topics[0] ? null : topics[0],
      raw: {
        data,
        topics,
      },
    };
  }
}
