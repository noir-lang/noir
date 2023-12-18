# Aztec Boxes
A minimal framework for building full stack applications for Aztec (using React).

## Introduction

This folder contains the "boxes" that are meant for quickstarts for Aztec smart contract developers, including simple Noir smart contracts and frontends.

Note because this depends on packages in the parallel workspace `yarn-project`, it uses "portal" dependencies which requires yarn version 2+.

This was installed with 
```
$yarn set version berry
```

which also required a node version of 18.12 or higher, via

```
nvm use 18.12.0
```

## Debugging

If CI is failing, it may be due to incompatibility with previous build artifacts - running the following command inside this boxes folder should regenerate the artifacts.

```
./bootstrap.sh
```
