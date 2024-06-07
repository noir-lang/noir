const { execSync } = require("child_process");

describe("Proof of Path (Part 3)", function() {

  describe("Public Test 1", function() {

    it("Should have valid proof", async function() {
      execSync("nargo verify -v test/TestVerifier.toml");
    });

  });

});
