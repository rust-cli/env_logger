use std::collections::BTreeSet;

pub fn all<T>(input: &[T]) -> BTreeSet<BTreeSet<T>>
where
    T: Ord + Eq + Clone,
{
    let mut permutations = BTreeSet::new();

    if input.is_empty() {
        return permutations;
    }

    permutations.insert(input.iter().cloned().collect());

    if input.len() > 1 {
        for t in input {
            let p = input
                .iter()
                .filter(|pt| *pt != t)
                .cloned()
                .collect::<Vec<_>>();

            for pt in all(&p) {
                permutations.insert(pt);
            }
        }
    }

    permutations
}
