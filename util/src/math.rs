pub const fn ncr(n: usize, r: usize) -> usize {
    if n > r {
        let mut i = 0;
        let mut num = 1;
        let mut den = 1;
        while i < r {
            num *= n - i;
            i += 1;
            den *= i;
        }
        num / den
    } else if n == r {
        1
    } else {
        0
    }
}
