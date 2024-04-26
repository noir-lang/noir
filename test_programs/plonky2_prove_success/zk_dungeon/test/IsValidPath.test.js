const { runNargoTest, runNargoCheck } = require("./testsuites/testNoir.js");

describe("Is Valid Path (Part 1)", function() {

  const testFile = "valid_path_test_1";

  before(async function() {
    await runNargoCheck();
  });

  describe("Public Test 1", function() {

    it("Should flag out-of-bounds", async function() {
      runNargoTest(testFile, "test_out_of_bounds");
    });

    it("Should flag invalid move", async function() {
      runNargoTest(testFile, "test_invalid_move_1");
      runNargoTest(testFile, "test_invalid_move_2");
    });
    
    it("Should flag invalid start", async function() {
      runNargoTest(testFile, "test_invalid_start");
    });

    it("Should flag invalid end", async function() {
      runNargoTest(testFile, "test_invalid_end");
    });

  });

  describe("Public Test 2", function() {
    it("Should allow valid path", async function() {
      runNargoTest(testFile, "test_valid_path");
    });
  });

});
