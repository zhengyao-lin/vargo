use vstd::prelude::*;

#[allow(unused_imports)]
use crate_a::*;

verus! {

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
