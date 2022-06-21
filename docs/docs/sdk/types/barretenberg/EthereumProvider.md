```ts
interface EthereumProvider {
    request(args: RequestArguments): Promise<any>;
    on(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
    on(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
    on(notification: 'chainChanged', listener: (chainId: string) => void): this;
    on(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
    on(notification: 'message', listener: (message: ProviderMessage) => void): this;
    removeListener(notification: 'connect', listener: (connectInfo: ProviderConnectInfo) => void): this;
    removeListener(notification: 'disconnect', listener: (error: ProviderRpcError) => void): this;
    removeListener(notification: 'chainChanged', listener: (chainId: string) => void): this;
    removeListener(notification: 'accountsChanged', listener: (accounts: string[]) => void): this;
    removeListener(notification: 'message', listener: (message: ProviderMessage) => void): this;
}
```