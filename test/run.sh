#!/bin/bash

trap 'exit 130' INT

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
LILSAT="$(cargo --config "target.'cfg(unix)'.runner = 'ls'" run --release)"
TIMEOUT=10
RESULTS_DIR=$(mktemp -d)

getSATLIBTests() {
    suite="$1"
    satorunsat="$2"
    url="$3"
    path="$SCRIPT_DIR/satlib/$suite/$satorunsat/"
    if [ -d "$path" ]; then
	# echo "Skipping $path"
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

runTest() {
    satorunsat=$1
    cnf=$2

    timeout $TIMEOUT "$LILSAT" "$cnf" > "$cnf.stdout" 2> "$cnf.stderr"
    exit_code=$?

    if [ $exit_code -eq 124 ]; then
	echo "$cnf" >> "$RESULTS_DIR/timeout"
	return 1
    fi

    result=$(grep --no-filename SAT "$cnf.stdout" "$cnf.stderr")
    # Compare in lowercase
    if [ "${result,,}" = "${satorunsat,,}" ]; then
	echo "$cnf" >> "$RESULTS_DIR/success"
	return 0
    else
	echo "$cnf" >> "$RESULTS_DIR/failed"
	return 1
    fi
}

export -f runTest
export LILSAT TIMEOUT RESULTS_DIR

getSATLIBTests "flat30-60" "sat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/GCP/flat30-60.tar.gz"
getSATLIBTests "sw100-8-lp0-c5" "sat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/SW-GCP/sw100-8-lp0-c5.tar.gz"
getSATLIBTests "planning" "sat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/PLANNING/BlocksWorld/blocksworld.tar.gz"
getSATLIBTests "uniform-unsat75" "unsat" "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/RND3SAT/uuf75-325.tar.gz"

# Run all tests in parallel
{
  getTests sat | awk '{print "sat\t" $0}'
  getTests unsat | awk '{print "unsat\t" $0}'
} | parallel -j $(nproc) --bar --colsep '\t' runTest {1} {2}

# Collect results
echo
echo "======== RESULTS ========"
if [ -f "$RESULTS_DIR/success" ]; then
    success_count=$(wc -l < "$RESULTS_DIR/success")
else
    success_count=0
fi
if [ -f "$RESULTS_DIR/failed" ]; then
    fail_count=$(wc -l < "$RESULTS_DIR/failed")
else
    fail_count=0
fi
if [ -f "$RESULTS_DIR/timeout" ]; then
    timeout_count=$(wc -l < "$RESULTS_DIR/timeout")
else
    timeout_count=0
fi

echo "$success_count OK | $fail_count FAILED | $timeout_count TIMEOUT"
echo

if [ -f "$RESULTS_DIR/failed" ]; then
    while read cnf; do
	echo "FAIL $cnf"
    done < "$RESULTS_DIR/failed"
fi

if [ -f "$RESULTS_DIR/timeout" ]; then
    while read cnf; do
	echo "TIMEOUT $cnf"
    done < "$RESULTS_DIR/timeout"
fi

rm -rf "$RESULTS_DIR"
