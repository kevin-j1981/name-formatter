use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

mod normalize;

use normalize::{
    dedup_preserve_order, is_plausible_name, normalize_entry, sort_entries, split_entries,
};

struct Options {
    path: Option<String>,
    sort: bool,
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

    let mut cleaned = clean(&raw);
    if cleaned.is_empty() {
        eprintln!("warning: no plausible names found in input");
    }
    if opts.sort {
        sort_entries(&mut cleaned);
    }
    for name in cleaned {
        println!("{name}");
    }
    ExitCode::SUCCESS
}

/// Parse CLI arguments into options. Accepts an optional `--sort` flag
/// (in any position) plus at most one positional argument: an input path,
/// or "-"/omitted for stdin.
fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut path = None;
    let mut sort = false;
    for arg in args {
        match arg.as_str() {
            "--sort" => sort = true,
            _ if path.is_none() => path = Some(arg.clone()),
            _ => return Err(format!("unexpected argument: {arg}")),
        }
    }
    Ok(Options { path, sort })
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
    }

    #[test]
    fn rejects_a_second_positional_argument() {
        assert!(parse_args(&["a.txt".to_string(), "b.txt".to_string()]).is_err());
    }
}
