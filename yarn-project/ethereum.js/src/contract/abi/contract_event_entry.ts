import { LogResponse } from '../../eth_rpc/index.js';
import { hexToBuffer } from '../../hex_string/index.js';
import { EventLog } from '../contract_tx_receipt.js';
import { abiCoder } from './abi-coder/index.js';
import { ContractEntryDefinition } from './contract_abi_definition.js';
import { ContractEntry } from './contract_entry.js';

/**
 * The ContractEventEntry class represents a single event entry within a smart contract.
 * It provides functionality to encode and decode event topics and logs, as well as
 * handling filter parameters for indexed inputs of the event. This class extends the
 * ContractEntry base class, adding specific features for event handling in Ethereum
 * contracts. By utilizing this class, users can seamlessly interact with events emitted
 * by a smart contract, making it easier to track and process data related to those events.
 */
export class ContractEventEntry extends ContractEntry {
  /**
   * The unique event identifier derived from ABI.
   */
  public readonly signature: string;

  constructor(entry: ContractEntryDefinition) {
    super(entry);
    this.signature = abiCoder.encodeEventSignature(abiCoder.abiMethodToString(entry));
  }

  /**
   * Generate an array of event topics by encoding the filter values provided for indexed inputs.
   * For events which are not anonymous, the first topic will be the event's signature.
   * Each subsequent topic corresponds to an indexed input, with null values for missing filters.
   * Supports array values for indexed inputs, which will generate multiple topics for that input.
   *
   * @param filter - An object containing the filter values to encode as event topics.
   * @returns An array of encoded event topics (Buffer or Buffer[]), including the event signature if not anonymous.
   */
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

  /**
   * Decodes an event log response from a contract execution.
   * The input 'log' is an object containing data and topics received from the Ethereum transaction receipt.
   * This method returns an EventLog object containing the decoded event along with its metadata.
   *
   * @param log - The LogResponse object containing data and topics from the contract execution.
   * @returns An EventLog object with the decoded event, signature, arguments, and raw data.
   */
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
