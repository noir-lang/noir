#!/bin/bash
(cd cpp && ./bootstrap.sh)
cd ts
yarn build
npm link
