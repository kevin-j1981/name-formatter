use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

mod normalize;

use normalize::{dedup_preserve_order, is_plausible_name, normalize_entry, split_entries};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let raw = match read_input(&args) {
        Ok(raw) => raw,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let cleaned = clean(&raw);
    if cleaned.is_empty() {
        eprintln!("warning: no plausible names found in input");
    }
    for name in cleaned {
        println!("{name}");
    }
    ExitCode::SUCCESS
}

/// Read from the path given as the first argument, or from stdin if no
/// argument (or "-") was given.
fn read_input(args: &[String]) -> io::Result<String> {
    match args.first().map(String::as_str) {
        None | Some("-") => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
        Some(path) => fs::read_to_string(path),
    }
}

/// Run the full normalization pipeline over raw text: split into entries,
/// normalize each, drop anything that doesn't look like a name, and
/// deduplicate while preserving first-seen order.
fn clean(raw: &str) -> Vec<String> {
    let entries = split_entries(raw)
        .into_iter()
        .filter_map(|e| normalize_entry(&e))
        .filter(|e| is_plausible_name(e))
        .collect();
    dedup_preserve_order(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_a_messy_blob() {
        let raw = "  mary jane , MARY JANE\nBob O'brien\n007\n\n";
        let result = clean(raw);
        assert_eq!(
            result,
            vec!["Mary Jane".to_string(), "Bob O'Brien".to_string()]
        );
    }
}
