import {Vm} from "forge-std/Vm.sol";
import {strings} from "stringutils/strings.sol";
import {Strings} from "@openzeppelin/contracts/utils/Strings.sol";
import {TestBase} from "./TestBase.sol";

contract DifferentialFuzzer is TestBase {
    using strings for *;
    using Strings for uint256;

    enum PlonkFlavour {
        Invalid,
        Standard,
        Ultra,
        Honk
    }
    enum CircuitFlavour {
        Invalid,
        Blake,
        Add2,
        Ecdsa,
        Recursive
    }

    constructor() {}

    /// @notice the fuzzing flavour
    PlonkFlavour public plonkFlavour;

    /// @notice the circuit flavour
    CircuitFlavour public circuitFlavour;

    /// @notice the proofs public inputs
    uint256[] public inputs;

    function with_plonk_flavour(PlonkFlavour _flavour) public returns (DifferentialFuzzer) {
        plonkFlavour = _flavour;
        return this;
    }

    function with_circuit_flavour(CircuitFlavour _flavour) public returns (DifferentialFuzzer) {
        circuitFlavour = _flavour;
        return this;
    }

    function with_inputs(uint256[] memory _inputs) public returns (DifferentialFuzzer) {
        inputs = _inputs;
        return this;
    }

    function get_plonk_flavour() internal view returns (string memory) {
        if (plonkFlavour == PlonkFlavour.Standard) {
            return "standard";
        } else if (plonkFlavour == PlonkFlavour.Ultra) {
            return "ultra";
        } else if (plonkFlavour == PlonkFlavour.Honk) {
            return "honk";
        } else {
            revert("Invalid flavour");
        }
    }

    function get_circuit_flavour() internal view returns (string memory) {
        if (circuitFlavour == CircuitFlavour.Blake) {
            return "blake";
        } else if (circuitFlavour == CircuitFlavour.Add2) {
            return "add2";
        } else if (circuitFlavour == CircuitFlavour.Recursive) {
            return "recursive";
        } else if (circuitFlavour == CircuitFlavour.Ecdsa) {
            return "ecdsa";
        } else {
            revert("Invalid circuit flavour");
        }
    }

    // Encode inputs as a comma separated string for the ffi call
    function get_inputs() internal view returns (string memory input_params) {
        input_params = "";
        if (inputs.length > 0) {
            input_params = inputs[0].toHexString();
            for (uint256 i = 1; i < inputs.length; i++) {
                input_params = string.concat(input_params, ",", inputs[i].toHexString());
            }
        }
    }

    function generate_proof() public returns (bytes memory proof) {
        // Craft an ffi call to the prover binary
        string memory prover_path = "./scripts/run_fuzzer.sh";
        string memory plonk_flavour = get_plonk_flavour();
        string memory circuit_flavour = get_circuit_flavour();
        string memory input_params = get_inputs();

        // Execute the c++ prover binary
        string[] memory ffi_cmds = new string[](4);
        ffi_cmds[0] = prover_path;
        ffi_cmds[1] = plonk_flavour;
        ffi_cmds[2] = circuit_flavour;
        ffi_cmds[3] = input_params;

        proof = vm.ffi(ffi_cmds);
    }
}
