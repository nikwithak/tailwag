pub fn gcd(
    mut a: i64,
    mut b: i64,
) -> i64 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

pub fn lcm(
    a: i64,
    b: i64,
) -> i64 {
    (a / gcd(a, b)) * b
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_gcd() {
        assert_eq!(5, gcd(15, 20));
    }
    #[test]
    fn test_lcm() {
        assert_eq!(20, lcm(4, 5));
    }
}
