# KV Store

The Aztec KV store is an implementation of a durable key-value database with a pluggable backend. THe only supported backend right now is LMDB by using the [`lmdb-js` package](https://github.com/kriszyp/lmdb-js).

This package exports a number of primitive data structures that can be used to build domain-specific databases in each node component (e.g. a PXE database or an Archiver database). The data structures supported:

- singleton - holds a single value. Great for when a value needs to be stored but it's not a collection (e.g. the latest block header or the length of an array)
- array - works like a normal in-memory JS array. It can't contain holes and it can be used as a stack (push-pop mechanics).
- map - a hashmap where keys can be numbers or strings
- multi-map - just like a map but each key holds multiple values. Can be used for indexing into other data structures
