BUILD?=.
ifeq ($(shell uname),Redox)
IS_REDOX=1
else
IS_REDOX=0
endif

include ./Makefile.tests.mk

CARGO_TEST?=cargo
TEST_RUNNER?=

.PHONY: all clean run run-once

all: run

clean:
	rm -rf $(BUILD)/gen

run: $(BUILD)/bins_verify/relibc-tests $(BINS) $(EXPECT_BINS) $(EXPECT_INPUT_BINS)
	@echo "\033[1;36;49mRunning tests\033[0m"
	$(TEST_RUNNER) $< $(patsubst %,-s%,$(BINS)) $(EXPECT_BINS)
	for exp in $(EXPECT_INPUT_BINS); \
	do \
		echo "# expect $$(readlink -e $${exp}.exp) $$(readlink -e $${exp}) #"; \
		expect "$$(readlink -e $${exp}.exp)" "$$(readlink -e $${exp})" test args || exit $$?; \
	done

run-once: $(BUILD)/bins_verify/relibc-tests $(BUILD)/$(TESTBIN)
	@echo "\033[1;36;49mRunning single test $(TESTBIN) $*\033[0m"
	@if [ -f "expected/$(TESTBIN).stdout" ]; then \
		$(TEST_RUNNER) $< $(BUILD)/$(TESTBIN); \
	else $(TEST_RUNNER) $< -s$(BUILD)/$(TESTBIN); fi
