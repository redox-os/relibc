#!/bin/sh

rm -rf gen || exit 1
mkdir -p gen || exit 1

while [ "$#" -gt 0 ]
do
    name="$1"
    shift

	echo "# ${name} #"
	mkdir -p "gen/$(dirname ${name})" || exit 1

	"bins/${name}" test args > "gen/${name}.stdout" 2> "gen/${name}.stderr"
    status="$?"

    for output in stdout stderr
    do
        gen="$(sha256sum "gen/${name}.${output}" | cut -d " " -f 1)"
        expected="$(sha256sum "expected/${name}.${output}" | cut -d " " -f 1)"
        if [ "${gen}" != "${expected}" ]
        then
            echo "# ${name}: ${output}: expected #"
            cat "expected/${name}.${output}"

            echo "# ${name}: ${output}: generated #"
            cat "gen/${name}.${output}"

            # FIXME: Make diff available on Redox
            if [ $(uname) != "Redox" ]
            then
                echo "# ${name}: ${output}: diff #"
                diff --color -u "expected/${name}.${output}" "gen/${name}.${output}"
            fi

            status="${status}, ${output} mismatch"
        fi
    done

    if [ "${status}" != "0" ]
    then
        echo "# ${name}: failed with status ${status} #"
    fi
done
