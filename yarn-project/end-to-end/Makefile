
# Just running the targets with LOCALLY will unfortunately not be parallel with earthly naively
# We have to use a Makefile to run them all parallel interleaved while preserving error codes.
# perform an end-to-end test with an earthly build
+%:
		earthly $@

DEPS := $(shell earthly ls)
# perform all, meant to be used with -j <concurrency>
all: $(DEPS)
