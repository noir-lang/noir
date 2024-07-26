#! /bin/bash
set -eu
export DEBUG='aztec:*'
export ARCHIVER_POLLING_INTERVAL=500
export P2P_CHECK_INTERVAL=50
export WS_CHECK_INTERVAL=50
export SEQ_TX_POLLING_INTERVAL=50
export SEQ_MAX_TX_PER_BLOCK=32
export SEQ_MIN_TX_PER_BLOCK=32
export SEQ_MAX_SECONDS_BETWEEN_BLOCKS=0
export SEQ_MIN_SECONDS_BETWEEN_BLOCKS=0
export BOOTSTRAP_NODES='/ip4/127.0.0.1/tcp/40400/p2p/12D3KooWGBpbC6qQFkaCYphjNeY6sV99o4SnEWyTeBigoVriDn4D'
export P2P_TCP_LISTEN_ADDR='0.0.0.0:40400'
export P2P_NAT_ENABLED='false'
export P2P_ENABLED='true'
export DEBUG='aztec:*,libp2p:*'

yarn test e2e_p2p_network.test.ts
