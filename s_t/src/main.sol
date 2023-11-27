
contract NoirTest {

    /// A message that can be called on instantiated contracts.
    /// This one flips the value of the stored `bool` from `true`
    /// to `false` and vice versa.
    function main(uint256 x, uint256 y) public pure returns (uint256) {
        return x + y;
    }
}