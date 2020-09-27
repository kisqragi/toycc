cargo build
if [ $? != 0 ]; then
    exit 1
fi

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

assert 0 '{ return 0; }'
assert 42 '{ return 42; }'
assert 5 '{ return 2+3; }'
assert 5 '{ return 8-3; }'
assert 9 '{ return 10+3-4; }'
assert 41 '{ return  12 + 34 - 5 ; }'
assert 14 '{ return 4 + 5 * 2; }'
assert 18 '{ return (4 + 5) * 2; }'
assert 4 '{ return (3+5)/2; }'
assert 5 '{ return 10/(5-3); }'
assert 10 '{ return -10 + 20; }'
assert 10 '{ return - -10; }'
assert 10 '{ return - - +10; }'

assert 0 '{ return 0 == 1; }'
assert 1 '{ return 1 == 1; }'
assert 1 '{ return 0 != 1; }'
assert 0 '{ return 1 != 1; }'

assert 1 '{ return 0 < 1; }'
assert 0 '{ return 1 < 0; }'
assert 0 '{ return 0 > 1; }'
assert 1 '{ return 1 > 0; }'

assert 1 '{ return 1 <= 1; }'
assert 1 '{ return 1 <= 2; }'
assert 0 '{ return 1 <= 0; }'
assert 0 '{ return 0 >= 1; }'
assert 1 '{ return 1 >= 1; }'
assert 1 '{ return 2 >= 1; }'

assert 1 '{ return 1; 2; 3; }'
assert 2 '{ 1; return 2; 3; }'
assert 3 '{ 1; 2; return 3; }'

assert 3 '{ int a=3; return a; }'
assert 5 '{ int a=3; a=5; return a; }'
assert 3 '{ int k = 3; return k; }'
assert 8 '{ int a=3; int z=5; return a+z; }'
assert 6 '{ int a; int b; a=b=3; return a+b; }'

assert 3 '{ int foo=3; return foo; }'
assert 7 '{ int foo=3; int hoge=4; return foo+hoge; }'
assert 1 '{ int foo=3; int hoge=4; return hoge-foo; }'
assert 2 '{ int foo=5; int hoge=3; return foo-hoge; }'

assert 123 '{ int foo123=123; return foo123; }'
assert 1 '{ int _foo=1; return _foo; }'

assert 3 '{ if (0) return 2; return 3; }'
assert 3 '{ if (1-1) return 2; return 3; }'
assert 2 '{ if (1) return 2; return 3; }'
assert 2 '{ if (2-1) return 2; return 3; }'
assert 2 '{ int a=2;if (a-1) return 2; return 3; }'
assert 3 '{ int a=2;if (a-2) return 2; return 3; }'

assert 3 '{ int a=0;if (0) a=2; else a=3; return a; }'
assert 2 '{ int a=0;if (1) a=2; else a=3; return a; }'

assert 55 '{ int i=0; int j=0; for (i=0; i<= 10; i=i+1) j=i+j; return j; }'
assert 5 '{ for (;;) return 5; return 0; }'
assert 55 '{ int j=0; int i; for (i=0; i<= 10; i=i+1) j=i+j; return j; }'
assert 5 '{ int i; for (i=0; i<= 10; i=i+1) if (i>=5) return i; return 0; }'

assert 10 '{ int i=0; while(i<10) i=i+1; return i; }'

assert 3 '{ {1; {2;} return 3;} }'

assert 55 '{ int i=0; int j=0; for(;;) { j=j+1; i=i+j; if (j==10) return i; } return 0; }'

# zinccとはローカル変数の配置順が違うので注意
# toy   = ..r14->r15->x->y
# zincc = ..r14->r15->y->x
assert 3 '{int x=3; return *&x; }'
assert 3 '{int x=3; int *y=&x; int **z=&y; return **z; }'
assert 5 '{int x=3; int y=5; return *(&x-1); }'
assert 3 '{int x=3; int y=5; return *(&y+1); }'
assert 5 '{int x=3; int *y=&x; *y=5; return x; }'
assert 7 '{int x=3; int y=5; *(&x-1)=7; return y; }'
assert 7 '{int x=3; int y=5; *(&y+1)=7; return x; }'
assert 2 '{ int x=3; return (&x+2)-&x; }'

assert 8 '{ int x, y; x=3; y=5; return x+y; }'
assert 8 '{ int x=3, y=5; return x+y; }'

echo OK
