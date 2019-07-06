#!/bin/sh

set -e

rm -rf gen
mkdir -p gen

while [ "$#" -gt 0 ]
do
    name="$1"
    shift

	echo "# ${name} #"
	mkdir -p "gen/$(dirname ${name})"
	"bins/${name}" test args > "gen/${name}.stdout" 2> "gen/${name}.stderr"
    for output in stdout stderr
    do
        if [ "$(uname)" = "Redox" ]
        then
            gen="$(sha256sum "gen/${name}.${output}" | cut -d " " -f 1)"
            expected="$(sha256sum "expected/${name}.${output}" | cut -d " " -f 1)"
            if [ "$gen" != "$expected" ]
            then
                echo "# $output: $gen != $expected #"

                echo "# $output generated #"
                cat "gen/${name}.${output}"

                echo "# $output expected #"
                cat "expected/${name}.${output}"

                exit 1
            fi
        else
        	diff -u "gen/${name}.${output}" "expected/${name}.${output}"
        fi
    done
done
