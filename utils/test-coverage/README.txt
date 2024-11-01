This directory includes scripts that help measure the test coverage of Noir.

They should be run from the main directory of the repo like this:

utils/test-coverage/0_cleanup.sh
utils/test-coverage/1_test.sh
...
etc.

Short description of what each of these scripts does:

0_cleanup.sh:
  Cleans up the files from a previous test coverage measurement.

1_test.sh:
  Runs the testsuite with test coverage measurement enabled. Testsuite run is
  about 8 times slower than normal. Produces lots of *.profraw files all over
  the place. A typical test run produced: 6,966,994 KiB in 1331 files

2_profdata.sh:
  Combines all the *.profraw files into a single file, called
  'noir_tests.profdata'.

3_cov.sh:
  Produces a test coverage report. What the test does is just an example. You
  can use the command, called by the script and modify its parameters, to
  produce all kinds of reports. For specific details, you should consult the
  documentation for the llvm-cov command.
