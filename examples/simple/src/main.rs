use vstd::prelude::*;

verus! {

closed spec fn spec_fib(n: int) -> int
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

proof fn spec_fib_non_negative(n: int)
    ensures spec_fib(n) >= 0
    decreases n
{
    if n > 1 {
        spec_fib_non_negative(n - 1);
        spec_fib_non_negative(n - 2);
    }
}

proof fn spec_fib_monotone(a: int, b: int)
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

fn fib(n: u64) -> (res: u64)
    requires spec_fib(n + 1) <= u64::MAX,
    ensures res == spec_fib(n as int)
{
    let mut a = 0;
    let mut b = 1;

    #[allow(unused_variables)]
    for i in 0..n
        invariant
            spec_fib(n + 1) <= u64::MAX,
            a == spec_fib(i as int),
            b == spec_fib(i + 1),
    {
        proof { spec_fib_monotone(i + 2, n + 1); }
        let tmp = a;
        a = b;
        b = tmp + b;
    }

    return a;
}

}

fn main() {
    println!("fib(10) = {}", fib(10));
}
