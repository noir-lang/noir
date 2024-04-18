struct SelfReferential
{
    prop : Option<SelfReferential>
}

#[test]
unconstrained
fn test_self_referential()
{
    let self_ref = SelfReferential { prop : Option::none() };
}
