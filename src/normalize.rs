use std::collections::HashSet;

/// Split a raw blob of messy input into candidate name entries.
///
/// Input might come from a pasted spreadsheet column, a comma-separated
/// list, or one name per line with stray tabs. We treat newlines, commas,
/// semicolons, and tabs all as separators since real-world exports mix them
/// depending on where the list originally came from.
pub fn split_entries(raw: &str) -> Vec<String> {
    raw.split(|c: char| matches!(c, '\n' | '\r' | ',' | ';' | '\t'))
        .map(|s| s.to_string())
        .collect()
}

/// Normalize a single entry: trim surrounding whitespace and quotes,
/// collapse repeated internal whitespace, and title-case each word while
/// respecting hyphens and apostrophes as word boundaries.
pub fn normalize_entry(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_matches(|c| c == '"' || c == '\'').trim();
    if trimmed.is_empty() {
        return None;
    }

    let collapsed = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");

    let formatted = collapsed
        .split(' ')
        .map(title_case_word)
        .collect::<Vec<_>>()
        .join(" ");

    Some(formatted)
}

/// Title-case a single word, treating '-' and '\'' as sub-word boundaries
/// so "mary-jane" becomes "Mary-Jane" and "o'brien" becomes "O'Brien".
/// The apostrophe boundary already gets O'Brien-style names right on its
/// own. Mc/Mac surnames need a curated list instead: blindly capitalizing
/// after any "mc"/"mac" prefix would turn "Mack" or "Macy" into "MacK" and
/// "MacY", so we only apply the extra capital for names we actually know.
fn title_case_word(word: &str) -> String {
    let mut result = String::with_capacity(word.len());
    let mut segment = String::new();
    for ch in word.chars() {
        if ch == '-' || ch == '\'' {
            result.push_str(&case_segment(&segment));
            segment.clear();
            result.push(ch);
        } else {
            segment.push(ch);
        }
    }
    result.push_str(&case_segment(&segment));
    result
}

/// Known Mc/Mac surnames where the letter after the prefix should also be
/// capitalized. Not exhaustive; anything not on this list falls back to
/// plain title-casing (e.g. "mack" stays "Mack", not "MacK").
const MC_MAC_SURNAMES: &[&str] = &[
    "macarthur",
    "macdonald",
    "macdougall",
    "macfarlane",
    "macgregor",
    "mackay",
    "mackenzie",
    "maclean",
    "macleod",
    "macmillan",
    "macneil",
    "macpherson",
    "mctavish",
    "mcallister",
    "mccarthy",
    "mcconnell",
    "mccormick",
    "mcdaniel",
    "mcdonald",
    "mcdougall",
    "mcfadden",
    "mcgee",
    "mcgrath",
    "mcgregor",
    "mcguire",
    "mcintosh",
    "mcintyre",
    "mckay",
    "mckenzie",
    "mclaughlin",
    "mclean",
    "mcleod",
    "mcmahon",
    "mcmillan",
    "mcneil",
    "mcpherson",
];

/// Title-case one sub-word (a run of letters between boundaries), applying
/// the Mc/Mac exception when the whole sub-word matches a known surname.
fn case_segment(segment: &str) -> String {
    if segment.is_empty() {
        return String::new();
    }

    let lower = segment.to_lowercase();
    let prefix_len = if MC_MAC_SURNAMES.contains(&lower.as_str()) {
        if lower.starts_with("mac") {
            Some(3)
        } else if lower.starts_with("mc") {
            Some(2)
        } else {
            None
        }
    } else {
        None
    };

    let chars: Vec<char> = lower.chars().collect();
    match prefix_len {
        Some(prefix_len) if chars.len() > prefix_len => {
            let mut result = String::with_capacity(segment.len());
            result.extend(chars[0].to_uppercase());
            result.extend(&chars[1..prefix_len]);
            result.extend(chars[prefix_len].to_uppercase());
            result.extend(&chars[prefix_len + 1..]);
            result
        }
        _ => {
            let mut chars = lower.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

/// Reject entries that are clearly not names: empty strings, anything with
/// a digit, or anything that isn't letters/space/hyphen/apostrophe/period.
pub fn is_plausible_name(entry: &str) -> bool {
    if entry.is_empty() {
        return false;
    }
    entry
        .chars()
        .all(|c| c.is_alphabetic() || c.is_whitespace() || matches!(c, '-' | '\'' | '.'))
        && entry.chars().any(|c| c.is_alphabetic())
}

/// Remove duplicates (case-insensitive) while keeping the first-seen form
/// and original order, since callers likely want a stable, reviewable list
/// rather than one resorted alphabetically.
pub fn dedup_preserve_order(entries: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let key = entry.to_lowercase();
        if seen.insert(key) {
            out.push(entry);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_mixed_delimiters() {
        let raw = "Alice,Bob;  Carol\nDave\tEve";
        let parts = split_entries(raw);
        assert_eq!(parts.len(), 5);
    }

    #[test]
    fn normalizes_case_and_whitespace() {
        assert_eq!(
            normalize_entry("  mARY   jane  "),
            Some("Mary Jane".to_string())
        );
    }

    #[test]
    fn title_cases_hyphens_and_apostrophes() {
        assert_eq!(normalize_entry("o'brien"), Some("O'Brien".to_string()));
        assert_eq!(
            normalize_entry("anne-marie"),
            Some("Anne-Marie".to_string())
        );
    }

    #[test]
    fn title_cases_known_mc_and_mac_surnames() {
        assert_eq!(normalize_entry("mcdonald"), Some("McDonald".to_string()));
        assert_eq!(normalize_entry("MACKENZIE"), Some("MacKenzie".to_string()));
        assert_eq!(
            normalize_entry("mary mcguire"),
            Some("Mary McGuire".to_string())
        );
    }

    #[test]
    fn leaves_unknown_mac_names_plain() {
        assert_eq!(normalize_entry("mack"), Some("Mack".to_string()));
        assert_eq!(normalize_entry("macy"), Some("Macy".to_string()));
    }

    #[test]
    fn rejects_entries_with_digits() {
        assert!(!is_plausible_name("Agent007"));
        assert!(is_plausible_name("Jean-Paul"));
    }

    #[test]
    fn dedup_is_case_insensitive_and_stable() {
        let input = vec!["Alice".to_string(), "alice".to_string(), "Bob".to_string()];
        assert_eq!(
            dedup_preserve_order(input),
            vec!["Alice".to_string(), "Bob".to_string()]
        );
    }
}
