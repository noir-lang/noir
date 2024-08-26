export enum ReqRespType {
  Status = 'status',
  Ping = 'ping',
  /** Ask peers for specific transactions */
  TxsByHash = 'txs_by_hash',
}

export const PING_PROTOCOL = '/aztec/ping/0.1.0';
export const STATUS_PROTOCOL = '/aztec/status/0.1.0';

export type SubProtocol = typeof PING_PROTOCOL | typeof STATUS_PROTOCOL;

export type SubProtocolHandler = (msg: string) => Uint8Array;
