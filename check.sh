#!/bin/bash

RED='\033[1;38;5;196m'
GREEN='\033[1;38;5;46m'
NC='\033[0m'

show_help() {
    echo "Usage: $(basename "$0") [OPTIONS]"
    echo ""
    echo "Description:"
    echo "  Wrapper for Makefile / Cargo to run checks or tests on Redox OS targets."
    echo ""
    echo "Options:"
    echo "  --test              Run 'make test' instead of 'make all'"
    echo "  --cargo             Run 'cargo check' / 'cargo test' instead"
    echo "                         (note: cargo test is currently not maintained for relibc)"
    echo "  --host              Run the command on host (linux) target"
    echo "  --all-target        Run the command on all supported Redox architectures"
    echo "  --target=<target>   Override the target architecture (e.g., i586-unknown-redox)"
    echo "  --arch=<arch>       Override the target architecture using arch (e.g., i586)"
    echo "  --help              Show this help message"
    echo ""
    echo "Supported Targets:"
    for t in "${SUPPORTED_TARGETS[@]}"; do
        echo "  - $t"
    done
    echo "  - $(uname -m)-unknown-linux-gnu"
    echo ""
    echo "Environment:"
    echo "  TARGET              Sets the default target (overridden by --target)"
}

if ! command -v redoxer &> /dev/null; then
    echo "Error: 'redoxer' CLI not found."
    echo "Please install it: cargo install redoxer"
    exit 1
fi

if ! command -v cbindgen &> /dev/null; then
    echo "Error: 'cbindgen' CLI not found."
    echo "Please install it: cargo install cbindgen"
    exit 1
fi

SUPPORTED_TARGETS=(
    "x86_64-unknown-redox"
    "i586-unknown-redox"
    "aarch64-unknown-redox"
    "riscv64gc-unknown-redox"
)

CURRENT_TARGET="${TARGET:-x86_64-unknown-redox}"
CHECK_ALL=false
CMD_ACTION="make"
CARGO_ACTION="check"
MAKE_ACTION="all"
IS_HOST=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --all-target)
            CHECK_ALL=true
            ;;
        --test)
            MAKE_ACTION="test"
            CARGO_ACTION="test"
            ;;
        --cargo)
            CMD_ACTION="cargo"
            ;;
        --host)
            CURRENT_TARGET="$(uname -m)-unknown-linux-gnu"
            IS_HOST=1
            ;;
        --target=*)
            CURRENT_TARGET="${1#*=}"
            ;;
        --arch=*)
            CURRENT_TARGET="${1#*=}-unknown-redox"
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option '$1'${NC}"
            show_help
            exit 1
            ;;
    esac
    shift
done

run_redoxer() {
    export TARGET=$1
    REDOXER_ENV="redoxer env"
    if [ "$IS_HOST" -eq 0 ]; then
        redoxer toolchain || { echo -e "${RED}Fail: redoxer toolchain for: $target.${NC}" && exit 1; }
        export CARGOFLAGS=""
        export CARGO_TEST="redoxer"
        export TEST_RUNNER="redoxer exec --folder ../sysroot/$TARGET:/usr --folder . --"
        MAKE_ACTION="$MAKE_ACTION IS_REDOX=1"
    else
        REDOXER_ENV=""
    fi

    if [ "$CMD_ACTION" == "make" ]; then
        CMD_OPT="-j $(nproc) $MAKE_ACTION"
    else
        CMD_OPT="$CARGO_ACTION"
    fi

    echo "----------------------------------------"
    echo "Running $REDOXER_ENV $CMD_ACTION $CMD_OPT for: $TARGET"
    
    if $REDOXER_ENV $CMD_ACTION $CMD_OPT; then
        return 0
    else
        echo -e "${RED}Fail: $CMD_ACTION $CMD_OPT for $TARGET failed.${NC}"
        return 1
    fi
}

if [ "$CHECK_ALL" = true ]; then
    echo "Running $CMD_ACTION for all supported Redox targets..."
    
    has_error=false
    
    for target in "${SUPPORTED_TARGETS[@]}"; do
        if ! run_redoxer "$target"; then
            has_error=true
        fi
    done
    
    echo "----------------------------------------"
    if [ "$has_error" = true ]; then
        echo -e "${RED}Summary: One or more targets failed.${NC}"
        exit 1
    else
        echo -e "${GREEN}Summary: All targets passed!${NC}"
        exit 0
    fi
else
    if run_redoxer "$CURRENT_TARGET"; then
        echo -e "${GREEN}Success: $CARGO_ACTION for $CURRENT_TARGET passed.${NC}"
        exit 0
    else
        exit 1
    fi
fi
