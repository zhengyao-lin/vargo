use vstd::prelude::*;

verus! {

pub open spec fn spec_fib(n: int) -> int
    decreases n
{
    if n <= 0 {
        0
    } else if n == 1 {
        1
    } else {
        spec_fib(n - 1) + spec_fib(n - 2)
    }
}

pub proof fn spec_fib_non_negative(n: int)
    ensures spec_fib(n) >= 0
    decreases n
{
    if n > 1 {
        spec_fib_non_negative(n - 1);
        spec_fib_non_negative(n - 2);
    }
}

pub proof fn spec_fib_monotone(a: int, b: int)
    requires a <= b
    ensures spec_fib(a) <= spec_fib(b)
    decreases b - a
{
    if a < b {
        spec_fib_non_negative(a);
        spec_fib_non_negative(a - 1);
        spec_fib_monotone(a + 1, b);
    }
}

}
