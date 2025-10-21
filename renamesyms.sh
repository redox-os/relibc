#!/usr/bin/env bash

set -e

target=$1
deps_dir=$2

if [ -z "$target" ] || [ -z "$deps_dir" ]; then
    echo "Usage:\n\t./renamesyms.sh TARGET DEPS_DIR"
    exit 1
fi

if [ ! -f "$target" ]; then
    echo "Target file '$target' does not exist"
    exit 1
fi
if [ ! -d "$deps_dir" ] ; then
    echo "Deps dir '$deps_dir' does not exist or not a directory"
    exit 1
fi

symbols_file=`mktemp`
special_syms=(
    __rdl_oom
    __rg_alloc
    __rg_alloc_zeroed
    __rg_dealloc
    __rg_oom
    __rg_realloc
    __rust_alloc
    __rust_alloc_error_handler
    __rust_alloc_error_handler_should_panic
    __rust_alloc_zeroed
    __rust_dealloc
    __rust_no_alloc_shim_is_unstable
    __rust_realloc
    _RNvCsdRcTtHUCxqF_7___rustc12___rust_alloc
    _RNvCsdRcTtHUCxqF_7___rustc19___rust_alloc_zeroed
    _RNvCsdRcTtHUCxqF_7___rustc14___rust_dealloc
    _RNvCsdRcTtHUCxqF_7___rustc14___rust_realloc
    _RNvCsdRcTtHUCxqF_7___rustc8___rg_oom
)

for dep in `find $deps_dir -type f -name "*.rlib"`; do
    "${NM}" --format=posix -g "$dep" 2>/dev/null | sed 's/.*:.*//g' | awk '{if ($2 == "T") print $1}' | sed 's/^\(.*\)$/\1 __relibc_\1/g' >> $symbols_file
done

for special_sym in "${special_syms[@]}"; do
    echo "$special_sym __relibc_$special_sym" >> $symbols_file
done

sorted_file=`mktemp`
sort -u "$symbols_file" > "$sorted_file"
rm -f "$symbols_file"

"${OBJCOPY}" --redefine-syms="$sorted_file" "$target"

rm -f "$sorted_file"
