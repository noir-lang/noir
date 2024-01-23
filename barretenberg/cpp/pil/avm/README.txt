This folder contains PIL relations for the AVM.

Applied heuristic to assess the cost of a relation is given below.
This will be used to determine whether it is worth to merge some
relations into one.

N_mul = number of mulitplication in the relation
N_add = number of additions/subtraction in the relation
deg = degree of the relation (total degree of the polynomial)

Relation cost: degree * (N_mul + N_add/4)

Remark: addition/multiplication with a constant counts as well in the above metrics
Remark: For edge case, we prefer keep a good readability rather than merging.

Future: There is an optimization in sumcheck protocol allowing to skip some
        rows for relations which are not enabled for them (applies when not
        enabled over 2 adjacent rows). However, this feature is not yet enabled
        in the AVM context. This might change the decision on whether some relations
        should be merged or not. Basically, merging relations would decrease the
        likelihood to be disabled over adjacent rows.