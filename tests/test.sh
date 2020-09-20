cargo build

#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  ./target/debug/toy "$input" > ./target/tmp.s || exit
  gcc -static -o ./target/tmp ./target/tmp.s
  ./target/tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 0 0
assert 42 42
assert 5 '2+3'
assert 5 '8-3'
assert 9 '10+3-4'
assert 41 ' 12 + 34 - 5 '
assert 14 '4 + 5 * 2'
assert 18 '(4 + 5) * 2'
assert 4 '(3+5)/2'
assert 5 '10/(5-3)'
assert 10 '-10 + 20'
assert 10 '- -10'
assert 10 '- - +10'


echo OK
