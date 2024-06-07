const { runNargoTest, runNargoCheck } = require("./testsuites/testNoir.js");

describe("Is Safe Path (Part 2)", function() {

  const testFile = "safe_path_test_2";

  before(async function() {
    await runNargoCheck();
  });

  describe("Public Test 1", function() {

  it("Should flag unsafe path with one Watcher", async function() {
    runNargoTest(testFile, "test_one_watcher_unsafe");
  });

  it("Should flag unsafe path with many Watchers", async function() {
    runNargoTest(testFile, "test_many_watchers_unsafe_1");
    runNargoTest(testFile, "test_many_watchers_unsafe_2")
  });
    
  });

  describe("Public Test 2", function() {
    it("Should allow safe path with no watchers", async function() {
      runNargoTest(testFile, "test_no_watchers_safe");
    });

    it("Should allow safe path with one watcher", async function() {
      runNargoTest(testFile, "test_one_watcher_safe");
    });

    it("Should allow safe path with many watchers", async function() {
      runNargoTest(testFile, "test_many_watchers_safe");
    });
  });

});
