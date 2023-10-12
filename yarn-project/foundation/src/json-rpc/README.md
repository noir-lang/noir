# json-rpc

json-rpc

```
-- src
    -- client
      Code to use by a client wishing to use a json-rpc server
      Includes syntax sugar for making requests with normal method syntax
    -- server
      Code for easily turning a class into an exposed RPC with one endpoint per method
```

Each createJsonRpcClient and JsonRpcServer call needs a map of classes that will be translated in input and output values.
By default, Buffer is handled, but other user-made classes need to define toString() and static fromString() like so:

```
   class PublicKey {
     toString() {
       return '...';
     }
     static fromString(str) {
       return new PublicKey(...);
     }
   }
```

## Usage

In Dapp:

```
const wallet = createJsonRpcClient<WalletImplementation>('wallet-server.com', /*register classes*/ {PublicKey, TxRequest});
const response = await wallet.signTxRequest(accountPubKey, txRequest);
```

The client will send `[{ name: 'PublicKey', value: accountPubKey.toString() }, { name: 'TxRequest', txRequest.toString() }]` to the server.

In wallet:

```
const publicClient = createJsonRpcClient<PublicClient>('public-client.com',  /*register classes*/ ...);
const keyStore = createJsonRpcClient<KeyStore>('key-store.com',  /*register classes*/ ...);
```

Different clients for different services.

-- server
Running a wallet server:

```
const wallet = new WalletImplementation();
const server = new JsonRpcServer(wallet,  /*register classes*/ ...);
server.start(8080);
```
