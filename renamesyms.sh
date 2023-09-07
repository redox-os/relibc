#!/bin/bash
target=$1
deps_dir=$2

if [ -z "$target" ] || [ -z "$deps_dir" ]; then
    echo "Usage:\n\t./renamesyms.sh TARGET DEPS_DIR"
    exit 1
fi

symbols_file=`mktemp`
special_syms=(
    __rg_alloc
    __rg_alloc_zeroed
    __rg_dealloc
    __rg_oom
    __rg_realloc
    __rust_alloc
    __rust_alloc_zeroed
    __rust_dealloc
    __rust_realloc
)

for dep in `find $deps_dir -type f -name "*.rlib"`; do
    "${TARGET}-nm" --format=posix -g "$dep" 2>/dev/null | sed 's/.*:.*//g' | awk '{if ($2 == "T") print $1}' | sed 's/^\(.*\)$/\1 __relibc_\1/g' >> $symbols_file
done

for special_sym in "${special_syms[@]}"; do
    echo "$special_sym __relibc_$special_sym" >> $symbols_file
done

sorted_file=`mktemp`
sort -u "$symbols_file" > "$sorted_file"
rm -f "$symbols_file"

"${TARGET}-objcopy" --redefine-syms="$sorted_file" "$target"

rm -f "$sorted_file"
