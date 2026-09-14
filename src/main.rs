use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

mod normalize;
mod sample;

use normalize::{count_entries, is_plausible_name, normalize_entry, sort_entries, split_entries};
use sample::{weighted_sample, Rng};

struct Options {
    paths: Vec<String>,
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

    let raw = match read_input(&opts.paths) {
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
/// an optional `--sample N` flag (in any position) plus any number of
/// positional arguments: input paths, or "-" for stdin. With no positional
/// arguments at all, input is read from stdin.
fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut paths = Vec::new();
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
            other => paths.push(other.to_string()),
        }
        i += 1;
    }
    Ok(Options { paths, sort, sample })
}

/// Read and concatenate input from each of the given paths, in order,
/// joining pieces with a newline so entries from the end of one file don't
/// fuse with entries at the start of the next. "-" reads stdin at that
/// position. With no paths at all, read stdin once.
fn read_input(paths: &[String]) -> Result<String, String> {
    if paths.is_empty() {
        return read_stdin().map_err(|err| err.to_string());
    }

    let mut combined = String::new();
    for path in paths {
        let content = if path == "-" {
            read_stdin().map_err(|err| err.to_string())?
        } else {
            fs::read_to_string(path).map_err(|err| format!("{path}: {err}"))?
        };
        if !combined.is_empty() && !combined.ends_with('\n') {
            combined.push('\n');
        }
        combined.push_str(&content);
    }
    Ok(combined)
}

fn read_stdin() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
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
        assert_eq!(opts.paths, vec!["names.txt".to_string()]);

        let opts = parse_args(&["names.txt".to_string(), "--sort".to_string()]).unwrap();
        assert!(opts.sort);
        assert_eq!(opts.paths, vec!["names.txt".to_string()]);
    }

    #[test]
    fn parses_no_args_as_stdin_without_sort() {
        let opts = parse_args(&[]).unwrap();
        assert!(!opts.sort);
        assert!(opts.paths.is_empty());
        assert_eq!(opts.sample, None);
    }

    #[test]
    fn parses_multiple_positional_arguments_as_paths() {
        let opts = parse_args(&["a.txt".to_string(), "b.txt".to_string()]).unwrap();
        assert_eq!(opts.paths, vec!["a.txt".to_string(), "b.txt".to_string()]);
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
        assert_eq!(opts.paths, vec!["names.txt".to_string()]);
    }

    #[test]
    fn rejects_sample_without_a_number() {
        assert!(parse_args(&["--sample".to_string()]).is_err());
        assert!(parse_args(&["--sample".to_string(), "abc".to_string()]).is_err());
    }

    #[test]
    fn reads_and_concatenates_multiple_files() {
        let a = std::env::temp_dir().join(format!("name-formatter-test-a-{}", std::process::id()));
        let b = std::env::temp_dir().join(format!("name-formatter-test-b-{}", std::process::id()));
        fs::write(&a, "Alice\nBob").unwrap();
        fs::write(&b, "Carol\n").unwrap();

        let paths = vec![
            a.to_str().unwrap().to_string(),
            b.to_str().unwrap().to_string(),
        ];
        let raw = read_input(&paths).unwrap();

        fs::remove_file(&a).unwrap();
        fs::remove_file(&b).unwrap();

        assert_eq!(raw, "Alice\nBob\nCarol\n");
    }

    #[test]
    fn reports_which_file_is_missing() {
        let missing = std::env::temp_dir().join(format!(
            "name-formatter-test-missing-{}",
            std::process::id()
        ));
        let path = missing.to_str().unwrap().to_string();
        let err = read_input(&[path.clone()]).unwrap_err();
        assert!(err.contains(&path));
    }
}
