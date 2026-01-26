#!/bin/bash

trap 'exit 130' INT

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
LILSAT="$(cargo --config "target.'cfg(unix)'.runner = 'ls'" run --release)"
TIMEOUT=10

getSATLIBTests() {
    suite="$1"
    satorunsat="$2"
    url="$3"
    path="$SCRIPT_DIR/satlib/$suite/$satorunsat/"
    if [ -d "$path" ]; then
	echo "Skipping $path"
	return 0
    fi
    mkdir -p "$path"
    cd "$path"
    tmpdir=$(mktemp -d)
    curl -L "$url" -o "$tmpdir/download.tar.gz"
    tar -xf "$tmpdir/download.tar.gz" -C "$tmpdir"
    find "$tmpdir" -name '*.cnf' -exec mv {} "$path" \;
    rm -rf "$tmpdir"
}

getTests() {
    satorunsat=$1
    find "$SCRIPT_DIR" -path "*/$satorunsat/*" -name '*.cnf'
}

SUCCEEDED=( )
FAILED=( )
TIMEDOUT=( )

runTest() {
    satorunsat=$1
    cnf=$2
    # printf "${cnf}..."
    timeout $TIMEOUT "$LILSAT" "$cnf" > "$cnf.stdout" 2> "$cnf.stderr"
    if [ $? -eq 124 ]; then
	# printf " TIMEOUT\n"
	TIMEDOUT+=($cnf)
	return 1
    fi
    result=$(grep --no-filename SAT "$cnf.stdout" "$cnf.stderr")
    # Compare in lowercase
    if [ "${result,,}" = "${satorunsat,,}" ]; then
	# printf " OK (%s)\n" $result
	SUCCEEDED+=($cnf)
	return 0
    else
	# printf " FAIL (%s)\n" $result
	FAILED+=($cnf)
	return 1
    fi
}

updateStatus() {
    success_count=${#SUCCEEDED[@]}
    fail_count=${#FAILED[@]}
    timeout_count=${#TIMEDOUT[@]}
    cols=$(tput cols)
    printf '\r%-*.*s' "$cols" "$cols" "${success_count} OK | ${fail_count} FAILED | ${timeout_count} TIMEOUT | Running $1"
}

getSATLIBTests "flat30-60" "sat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/GCP/flat30-60.tar.gz"
getSATLIBTests "sw100-8-lp0-c5" "sat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/SW-GCP/sw100-8-lp0-c5.tar.gz"
getSATLIBTests "planning" "sat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/PLANNING/BlocksWorld/blocksworld.tar.gz"
getSATLIBTests "uniform-unsat75" "unsat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/RND3SAT/uuf75-325.tar.gz"

for cnf in $(getTests sat); do
    updateStatus "$cnf"
    runTest sat "$cnf"
done

for cnf in $(getTests unsat); do
    updateStatus "$cnf"
    runTest unsat "$cnf"
done

echo
for cnf in "${FAILED[@]}"; do
    echo "FAIL $cnf"
done
for cnf in "${TIMEDOUT[@]}"; do
    echo "TIMEOUT $cnf"
done
