use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::Hash;

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
