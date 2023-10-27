#[abi(export)]
unconstrained fn abi__compute_hash() {

}

#[abi(export)]
unconstrained fn abi__compute_block_hash() {
    abis::compute_hash()
}

The above allows us to export noir methods to be called from cpp