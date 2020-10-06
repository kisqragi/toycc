cargo build
if [ $? != 0 ]; then
    exit 1
fi

cat <<EOF | gcc -xc -c -o ./target/tmp2.o -
int ret3() { return 3; }
int ret5() { return 5; }
int add(int x, int y) { return x + y; }
int sub(int x, int y) { return x - y; }

int add6(int a, int b, int c, int d, int e, int f) {
    return a+b+c+d+e+f;
}
EOF


#!/bin/bash
assert() {
  expected="$1"
  input="$2"

  ./target/debug/toycc "$input" > ./target/tmp.s || exit
  gcc -static -o ./target/tmp ./target/tmp.s ./target/tmp2.o
  ./target/tmp
  actual="$?"

  if [ "$actual" = "$expected" ]; then
    echo "$input => $actual"
  else
    echo "$input => $expected expected, but got $actual"
    exit 1
  fi
}

assert 0 'int main() { return 0; }'
assert 42 'int main() { return 42; }'
assert 5 'int main() { return 2+3; }'
assert 5 'int main() { return 8-3; }'
assert 9 'int main() { return 10+3-4; }'
assert 41 'int main() { return  12 + 34 - 5 ; }'
assert 14 'int main() { return 4 + 5 * 2; }'
assert 18 'int main() { return (4 + 5) * 2; }'
assert 4 'int main() { return (3+5)/2; }'
assert 5 'int main() { return 10/(5-3); }'
assert 10 'int main() { return -10 + 20; }'
assert 10 'int main() { return - -10; }'
assert 10 'int main() { return - - +10; }'

assert 0 'int main() { return 0 == 1; }'
assert 1 'int main() { return 1 == 1; }'
assert 1 'int main() { return 0 != 1; }'
assert 0 'int main() { return 1 != 1; }'

assert 1 'int main() { return 0 < 1; }'
assert 0 'int main() { return 1 < 0; }'
assert 0 'int main() { return 0 > 1; }'
assert 1 'int main() { return 1 > 0; }'

assert 1 'int main() { return 1 <= 1; }'
assert 1 'int main() { return 1 <= 2; }'
assert 0 'int main() { return 1 <= 0; }'
assert 0 'int main() { return 0 >= 1; }'
assert 1 'int main() { return 1 >= 1; }'
assert 1 'int main() { return 2 >= 1; }'

assert 1 'int main() { return 1; 2; 3; }'
assert 2 'int main() { 1; return 2; 3; }'
assert 3 'int main() { 1; 2; return 3; }'

assert 3 'int main() { int a=3; return a; }'
assert 5 'int main() { int a=3; a=5; return a; }'
assert 3 'int main() { int k = 3; return k; }'
assert 8 'int main() { int a=3; int z=5; return a+z; }'
assert 6 'int main() { int a; int b; a=b=3; return a+b; }'

assert 3 'int main() { int foo=3; return foo; }'
assert 7 'int main() { int foo=3; int hoge=4; return foo+hoge; }'
assert 1 'int main() { int foo=3; int hoge=4; return hoge-foo; }'
assert 2 'int main() { int foo=5; int hoge=3; return foo-hoge; }'

assert 123 'int main() { int foo123=123; return foo123; }'
assert 1 'int main() { int _foo=1; return _foo; }'

assert 3 'int main() { if (0) return 2; return 3; }'
assert 3 'int main() { if (1-1) return 2; return 3; }'
assert 2 'int main() { if (1) return 2; return 3; }'
assert 2 'int main() { if (2-1) return 2; return 3; }'
assert 2 'int main() { int a=2;if (a-1) return 2; return 3; }'
assert 3 'int main() { int a=2;if (a-2) return 2; return 3; }'

assert 3 'int main() { int a=0;if (0) a=2; else a=3; return a; }'
assert 2 'int main() { int a=0;if (1) a=2; else a=3; return a; }'

assert 55 'int main() { int i=0; int j=0; for (i=0; i<= 10; i=i+1) j=i+j; return j; }'
assert 5 'int main() { for (;;) return 5; return 0; }'
assert 55 'int main() { int j=0; int i; for (i=0; i<= 10; i=i+1) j=i+j; return j; }'
assert 5 'int main() { int i; for (i=0; i<= 10; i=i+1) if (i>=5) return i; return 0; }'

assert 10 'int main() { int i=0; while(i<10) i=i+1; return i; }'

assert 3 'int main() { {1; {2;} return 3;} }'

assert 55 'int main() { int i=0; int j=0; for(;;) { j=j+1; i=i+j; if (j==10) return i; } return 0; }'

# zinccとはローカル変数の配置順が違うので注意
# toycc = ..r14->r15->x->y
# zincc = ..r14->r15->y->x
assert 3 'int main() {int x=3; return *&x; }'
assert 3 'int main() {int x=3; int *y=&x; int **z=&y; return **z; }'
assert 5 'int main() {int x=3; int y=5; return *(&x-1); }'
assert 3 'int main() {int x=3; int y=5; return *(&y+1); }'
assert 5 'int main() {int x=3; int *y=&x; *y=5; return x; }'
assert 7 'int main() {int x=3; int y=5; *(&x-1)=7; return y; }'
assert 7 'int main() {int x=3; int y=5; *(&y+1)=7; return x; }'
assert 2 'int main() { int x=3; return (&x+2)-&x; }'

assert 8 'int main() { int x, y; x=3; y=5; return x+y; }'
assert 8 'int main() { int x=3, y=5; return x+y; }'

assert 3 'int main() { return ret3(); }'
assert 5 'int main() { return ret5(); }'
assert 5 'int main() { int a = ret3(); a = ret5(); return a; }'
assert 3 'int main() { int a = ret5(); a = ret3(); return a; }'

assert 5 'int main() { return add(2, 3); }'
assert 2 'int main() { return sub(5, 3); }'
assert 21 'int main() { return add6(1,2,3,4,5,6); }'
assert 5 'int main() { int a=2, b=3; return add(a, b); }'

assert 10 'int main() { return ret10(); } int ret10 { return 10; }'
assert 10 'int ret10 { return 10; } int main() { return ret10(); }'

assert 5 'int main() { return add2(2,3); } int add2(int x, int y) { return x + y; }'
assert 5 'int main() { return sub(10,5); } int sub2(int x, int y) { return x - y; }'
assert 55 'int main() { return fib(9); } int fib (int x) { if (x<=1) return 1; return fib(x-1) + fib(x-2); }'

assert 21 'int main() { return _add6(1,2,3,4,5,6); } int _add6(int a, int b, int c, int d, int e, int f) { return a+b+c+d+e+f; }'

echo OK
