/// Returns the length of the longest common prefix of two strings.
pub(crate) fn longest_prefix(key1: &str, key2: &str) -> usize {
    let max_len = key1.len().min(key2.len());

    let key1_bytes = key1.as_bytes();
    let key2_bytes = key2.as_bytes();
    let mut i = 0;
    while i < max_len {
        if key1_bytes[i] != key2_bytes[i] {
            break;
        }
        i += 1;
    }
    i
}

pub trait NodeValue = Default + std::fmt::Debug + Clone + std::hash::Hash + PartialEq + Eq;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_prefix() {
        assert_eq!(longest_prefix("hello", "helicopter"), 3);
        assert_eq!(longest_prefix("test", "testing"), 4);
        assert_eq!(longest_prefix("abc", "xyz"), 0);
        assert_eq!(longest_prefix("", "nonempty"), 0);
        assert_eq!(longest_prefix("same", "same"), 4);
    }
}
