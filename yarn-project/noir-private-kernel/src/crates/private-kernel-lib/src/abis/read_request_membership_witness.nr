struct ReadRequestMembershipWitness<N> {
    leaf_index : Field,
    sibling_path : [Field; N],
    
    // whether or not the read request corresponds to a pending commitment
    // In case we change the default to true, we have to adapt is_empty() method
    // hint to point kernel to the commitment this rr corresponds to
    is_transient : bool,

    hint_to_commitment : Field,
}

