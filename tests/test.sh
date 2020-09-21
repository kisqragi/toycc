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

assert 0 'return 0;'
assert 42 'return 42;'
assert 5 'return 2+3;'
assert 5 'return 8-3;'
assert 9 'return 10+3-4;'
assert 41 'return  12 + 34 - 5 ;'
assert 14 'return 4 + 5 * 2;'
assert 18 'return (4 + 5) * 2;'
assert 4 'return (3+5)/2;'
assert 5 'return 10/(5-3);'
assert 10 'return -10 + 20;'
assert 10 'return - -10;'
assert 10 'return - - +10;'

assert 0 'return 0 == 1;'
assert 1 'return 1 == 1;'
assert 1 'return 0 != 1;'
assert 0 'return 1 != 1;'

assert 1 'return 0 < 1;'
assert 0 'return 1 < 0;'
assert 0 'return 0 > 1;'
assert 1 'return 1 > 0;'

assert 1 'return 1 <= 1;'
assert 1 'return 1 <= 2;'
assert 0 'return 1 <= 0;'
assert 0 'return 0 >= 1;'
assert 1 'return 1 >= 1;'
assert 1 'return 2 >= 1;'

assert 1 'return 1; 2; 3;'
assert 2 '1; return 2; 3;'
assert 3 '1; 2; return 3;'

assert 3 'a=3; return a;'
assert 3 'k = 3; return k;'
assert 8 'a=3; z=5; return a+z;'
assert 6 'a=b=3; return a+b;'

echo OK
