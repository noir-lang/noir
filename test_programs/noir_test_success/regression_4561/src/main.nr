// Regression test for issue #4561
use dep::std::test::OracleMock;

type TReturnElem = [Field; 3];
type TReturn = [TReturnElem; 2];

#[oracle(repro)]
unconstrained fn repro_oracle() -> TReturn {}

unconstrained fn repro_unconstrained() -> TReturn {
    repro_oracle()
}

#[test]
fn test_mock() {
    OracleMock::mock("repro").returns([1, 2, 3, 4, 5, 6]);
    assert_eq(repro_unconstrained(), [[1, 2, 3], [4, 5, 6]]);
}
