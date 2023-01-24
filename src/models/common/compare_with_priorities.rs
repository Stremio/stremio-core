use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

/// Compare `a` and `b` based on provided priority.
/// Priorities should be set based on the type `T` and `i32`.
///
/// # Examples
///
/// ```
/// use std::{cmp::Ordering, collections::HashMap};
/// use stremio_core::{constants::TYPE_PRIORITIES, models::common::compare_with_priorities};
///
/// # fn main() {
///
/// let priorities = vec![("movie", 4), ("series", 3)]
///     .into_iter()
///     .collect::<HashMap<&str, i32>>();
///
/// let actual = compare_with_priorities("movie", "series", &priorities);
/// assert_eq!(Ordering::Greater, actual);
/// # }
/// ```
pub fn compare_with_priorities<T: Ord + Hash + Eq + ?Sized>(
    a: &T,
    b: &T,
    priorities: &HashMap<&T, i32>,
) -> Ordering {
    match (priorities.get(a), priorities.get(b)) {
        (Some(a_priority), Some(b_priority)) => a_priority.cmp(b_priority),
        (Some(priority), None) => {
            if *priority == i32::MIN {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (None, Some(priority)) => {
            if *priority == i32::MIN {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (None, None) => b.cmp(a),
    }
}
