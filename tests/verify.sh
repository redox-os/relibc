#!/bin/sh

rm -rf gen || exit 1
mkdir -p gen || exit 1

summary=""

while [ "$#" -gt 0 ]
do
    bin="$1"
    shift

    echo "# ${bin} #"
    mkdir -p "gen/$(dirname ${bin})" || exit 1

    "${bin}" test args > "gen/${bin}.stdout" 2> "gen/${bin}.stderr"
    retcode="$?"
    status=""

    for output in stdout stderr
    do
        gen="$(sha256sum "gen/${bin}.${output}" | cut -d " " -f 1)"

        # look for expected output file that is specific to binary type (either static or dynamic)
        expected_file="expected/${bin}.${output}"
        if [ ! -e $expected_file ]
        then
            # if unable to find above, the expected output file is the same to both static and dynamic binary
            name=$(echo $bin | cut -d "/" -f2-)
            expected_file="expected/${name}.${output}"
        fi
        expected="$(sha256sum "${expected_file}" | cut -d " " -f 1)"
        if [ "${gen}" != "${expected}" ]
        then
            echo "# ${bin}: ${output}: expected #"
            cat "${expected_file}"

            echo "# ${bin}: ${output}: generated #"
            cat "gen/${bin}.${output}"

            # FIXME: Make diff available on Redox
            if [ $(uname) != "Redox" ]
            then
                echo "# ${bin}: ${output}: diff #"
                diff --color -u "${expected_file}" "gen/${bin}.${output}"
            fi

            status="${bin} failed - retcode ${retcode}, ${output} mismatch"
            summary="${summary}${status}\n"
        fi
    done

    if [ -n "${status}" ]
    then
        echo "# ${status} #"
    fi
done

if [ -n "$summary" ]
then
    echo -e "$summary"
    exit 1
fi
