use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

mod normalize;
mod sample;

use normalize::{count_entries, is_plausible_name, normalize_entry, sort_entries, split_entries};
use sample::{weighted_sample, Rng};

struct Options {
    path: Option<String>,
    sort: bool,
    sample: Option<usize>,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let opts = match parse_args(&args) {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let raw = match read_input(opts.path.as_deref()) {
        Ok(raw) => raw,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let weighted = clean(&raw);
    if weighted.is_empty() {
        eprintln!("warning: no plausible names found in input");
    }

    if let Some(count) = opts.sample {
        let mut rng = Rng::from_entropy();
        for name in weighted_sample(&weighted, count, &mut rng) {
            println!("{name}");
        }
    } else {
        let mut cleaned: Vec<String> = weighted.into_iter().map(|(name, _)| name).collect();
        if opts.sort {
            sort_entries(&mut cleaned);
        }
        for name in cleaned {
            println!("{name}");
        }
    }
    ExitCode::SUCCESS
}

/// Parse CLI arguments into options. Accepts an optional `--sort` flag and
/// an optional `--sample N` flag (in any position) plus at most one
/// positional argument: an input path, or "-"/omitted for stdin.
fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut path = None;
    let mut sort = false;
    let mut sample = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--sort" => sort = true,
            "--sample" => {
                let value = args
                    .get(i + 1)
                    .ok_or("--sample requires a number of names to draw")?;
                let count: usize = value
                    .parse()
                    .map_err(|_| format!("--sample requires a number, got: {value}"))?;
                sample = Some(count);
                i += 1;
            }
            other if path.is_none() => path = Some(other.to_string()),
            other => return Err(format!("unexpected argument: {other}")),
        }
        i += 1;
    }
    Ok(Options { path, sort, sample })
}

/// Read from the given path, or from stdin if no path (or "-") was given.
fn read_input(path: Option<&str>) -> io::Result<String> {
    match path {
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
/// collapse duplicates into (name, occurrence count) pairs, preserving
/// first-seen order. The count doubles as a sampling weight for `--sample`.
fn clean(raw: &str) -> Vec<(String, usize)> {
    let entries = split_entries(raw)
        .into_iter()
        .filter_map(|e| normalize_entry(&e))
        .filter(|e| is_plausible_name(e))
        .collect();
    count_entries(entries)
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
            vec![("Mary Jane".to_string(), 2), ("Bob O'Brien".to_string(), 1)]
        );
    }

    #[test]
    fn parses_sort_flag_in_any_position() {
        let opts = parse_args(&["--sort".to_string(), "names.txt".to_string()]).unwrap();
        assert!(opts.sort);
        assert_eq!(opts.path.as_deref(), Some("names.txt"));

        let opts = parse_args(&["names.txt".to_string(), "--sort".to_string()]).unwrap();
        assert!(opts.sort);
        assert_eq!(opts.path.as_deref(), Some("names.txt"));
    }

    #[test]
    fn parses_no_args_as_stdin_without_sort() {
        let opts = parse_args(&[]).unwrap();
        assert!(!opts.sort);
        assert_eq!(opts.path, None);
        assert_eq!(opts.sample, None);
    }

    #[test]
    fn rejects_a_second_positional_argument() {
        assert!(parse_args(&["a.txt".to_string(), "b.txt".to_string()]).is_err());
    }

    #[test]
    fn parses_sample_flag_with_count() {
        let opts = parse_args(&["--sample".to_string(), "5".to_string()]).unwrap();
        assert_eq!(opts.sample, Some(5));

        let opts = parse_args(&[
            "names.txt".to_string(),
            "--sample".to_string(),
            "3".to_string(),
        ])
        .unwrap();
        assert_eq!(opts.sample, Some(3));
        assert_eq!(opts.path.as_deref(), Some("names.txt"));
    }

    #[test]
    fn rejects_sample_without_a_number() {
        assert!(parse_args(&["--sample".to_string()]).is_err());
        assert!(parse_args(&["--sample".to_string(), "abc".to_string()]).is_err());
    }
}
