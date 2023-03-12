import { LogResponse } from '../../eth_rpc/types/log_response.js';
import { bufferToHex } from '../../hex_string/index.js';
import { ContractAbiDefinition, ContractErrorEntry, ContractEventEntry, ContractFunctionEntry } from './index.js';

export class ContractAbi {
  public functions: ContractFunctionEntry[];
  public events: ContractEventEntry[];
  public errors: ContractErrorEntry[];
  public ctor: ContractFunctionEntry;
  public fallback?: ContractFunctionEntry;

  constructor(definition: ContractAbiDefinition) {
    this.functions = definition.filter(e => e.type === 'function').map(entry => new ContractFunctionEntry(entry));
    this.events = definition.filter(e => e.type === 'event').map(entry => new ContractEventEntry(entry));
    this.errors = definition.filter(e => e.type === 'error').map(entry => new ContractErrorEntry(entry));
    const ctor = definition.find(e => e.type === 'constructor');
    this.ctor = new ContractFunctionEntry(ctor || { type: 'constructor' });
    const fallback = definition.find(e => e.type === 'fallback');
    if (fallback) {
      this.fallback = new ContractFunctionEntry(fallback);
    }
  }

  public findEntryForLog(log: LogResponse) {
    return this.events.find(abiDef => abiDef.signature === log.topics[0]);
  }

  public decodeEvent(log: LogResponse) {
    const event = this.findEntryForLog(log);
    if (!event) {
      throw new Error(`Unable to find matching event signature for log: ${log.id}`);
    }
    return event.decodeEvent(log);
  }

  public decodeFunctionData(data: Buffer) {
    const funcSig = bufferToHex(data.subarray(0, 4));
    const func = this.functions.find(f => f.signature === funcSig);
    return func ? func.decodeParameters(data.slice(4)) : undefined;
  }
}
