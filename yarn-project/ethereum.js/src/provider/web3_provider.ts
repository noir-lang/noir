interface JsonRpcRequest {
  jsonrpc: string;
  method: string;
  params: any[];
  id: number;
}

interface JsonRpcResponse {
  jsonrpc: string;
  id: number;
  result?: any;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

type Callback = (err?: Error, result?: JsonRpcResponse) => void;

export interface Web3Provider {
  send(payload: JsonRpcRequest, callback: Callback): any;
}
