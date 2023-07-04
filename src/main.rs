use clap::Parser;
use ignore::{DirEntry, WalkBuilder};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::io::BufRead;
use std::path::Path;
use std::process::exit;
use std::{fs::File, io::BufReader};

/// A specific target which requires additional checks.
#[derive(Serialize, Deserialize, Debug)]
struct Specific {
    targets: Vec<String>,
    patterns: Vec<String>,
}

/// Top level file configuration.
#[derive(Serialize, Deserialize, Debug)]
struct Configuration {
    /// Used to mark a line as being overruled..
    overrule: String,

    /// Patterns which apply everywhere.
    global: Vec<String>,

    /// Patterns which only apply in specific directories, or to specific files
    specific: Vec<Specific>,
}

fn make_patterns(v: &Vec<String>) -> Vec<Regex> {
    let mut regexes = Vec::new();
    for s in v {
        match Regex::new(s.as_str()) {
            Ok(r) => {
                regexes.push(r);
            }
            Err(error) => {
                eprintln!("Error creating regex: {:?}", error);
            }
        }
    }
    regexes
}

struct ProhibitedResult {
    pub line_number: u32,
    pub line: String,
    pub pattern: String,
}

fn check_file(entry: &DirEntry, regexes: &Vec<Regex>, overrule: &str) -> Vec<ProhibitedResult> {
    let mut results = Vec::new();

    if let Some(t) = entry.file_type() {
        if !t.is_file() {
            return results;
        }

        if let Ok(file) = File::open(entry.path()) {
            let reader = BufReader::new(file);
            let mut line_number = 1;
            for line in reader.lines().flatten() {
                for r in regexes {
                    if r.is_match(line.as_str()) && !line.contains(overrule) {
                        results.push(ProhibitedResult {
                            line_number,
                            line: line.clone(),
                            pattern: r.as_str().to_string(),
                        });
                    }
                }
                line_number += 1;
            }
        }
    }
    results
}

fn check_specific(specific: &Specific, common_patterns: &Vec<Regex>, overrule: &str) -> bool {
    let mut extra_patterns = make_patterns(&specific.patterns);

    for r in common_patterns {
        extra_patterns.push(r.clone());
    }

    let mut success = true;
    for dir in &specific.targets {
        let path = Path::new(dir.as_str());
        let walk = WalkBuilder::new(path).build();
        for d in walk {
            match d {
                Ok(entry) => {
                    let errors = check_file(&entry, &extra_patterns, overrule);
                    if !errors.is_empty() {
                        success = false;
                        eprintln!(
                            "File check failed on {}",
                            entry.path().to_str().unwrap_or("<unknown file>")
                        );
                        for prohibition in errors {
                            println!(
                                "Prohibited value found: \"{}\" at {}:{}",
                                prohibition.pattern,
                                entry.path().to_str().unwrap_or("<unknown path>"),
                                prohibition.line_number
                            );
                            println!("{}\n", prohibition.line);
                        }
                    }
                }
                Err(err) => {
                    println!("Error {:?}", err);
                    success = false;
                }
            }
        }
    }
    success
}

#[derive(Parser, Debug)]
#[command(name = "Prohibit")]
#[command(version = "0.1")]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(default_value = "./prohibited.json")]
    config: String,
}

fn main() {
    let args = Args::parse();

    let path = args.config;
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        let read_result: Result<Configuration> = serde_json::from_reader(reader);

        // Read the configuration.
        if let Ok(configuration) = read_result {
            let common_patterns = make_patterns(&configuration.global);

            let mut pass = true;

            // Evaluate every specific target.
            for specific in configuration.specific {
                pass &=
                    check_specific(&specific, &common_patterns, configuration.overrule.as_str());
            }

            if pass {
                exit(0)
            } else {
                eprintln!("Check failed.");
                exit(1)
            }
        } else {
            eprintln!("Invalid configuration.");
        }
    } else {
        println!("Could not open configuration file {:?}", path);
    }
}
