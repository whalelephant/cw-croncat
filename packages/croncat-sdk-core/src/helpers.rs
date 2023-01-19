pub fn vect_difference<T: std::clone::Clone + std::cmp::PartialEq>(
    v1: &[T],
    v2: &[T],
) -> Vec<T> {
    v1.iter().filter(|&x| !v2.contains(x)).cloned().collect()
}