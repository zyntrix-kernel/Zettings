use unicode_normalization::UnicodeNormalization;

const COMBINING_MARKS: std::ops::RangeInclusive<char> = '\u{0300}'..='\u{036F}';

#[must_use]
pub fn fold(input: &str) -> String {
    input
        .nfd()
        .filter(|c| !COMBINING_MARKS.contains(c))
        .flat_map(char::to_lowercase)
        .collect()
}

#[must_use]
pub fn tokenize(folded: &str) -> Vec<String> {
    folded
        .split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit()))
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect()
}

#[must_use]
pub fn edit_distance_within(a: &str, b: &str, max: usize) -> bool {
    if a == b {
        return true;
    }
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return false;
    }

    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut current = vec![i + 1];
        let mut row_min = current[0];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            let value = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + cost);
            current.push(value);
            row_min = row_min.min(value);
        }
        if row_min > max {
            return false;
        }
        previous = current;
    }
    previous[b.len()] <= max
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_case_and_diacritics() {
        assert_eq!(fold("Café WIFI"), "cafe wifi");
        assert_eq!(fold("Ångström"), "angstrom");
    }

    #[test]
    fn tokenizes_on_non_alphanumerics() {
        assert_eq!(
            tokenize(&fold("Night-light: HDR!")),
            vec!["night", "light", "hdr"]
        );
    }

    #[test]
    fn edit_distance_bounds_hold() {
        assert!(edit_distance_within("wifi", "wiffi", 2));
        assert!(edit_distance_within("wifi", "wify", 1));
        assert!(edit_distance_within("abc", "abc", 0));
        assert!(!edit_distance_within("kitten", "sitting", 2));
        assert!(!edit_distance_within("wifi", "wiffii", 1));
    }
}
