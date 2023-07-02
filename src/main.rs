use clap::Parser;
use ignore::{DirEntry, WalkBuilder};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::io::BufRead;
use std::path::Path;
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

fn check_file(entry: DirEntry, regexes: &Vec<Regex>, overrule: &str) -> bool {
    match entry.file_type() {
        Some(t) => {
            if t.is_file() {
                if let Ok(file) = File::open(entry.path()) {
                    let reader = BufReader::new(file);
                    let mut line_number = 1;
                    for line in reader.lines().flatten() {
                        for r in regexes {
                            if r.is_match(line.as_str()) && !line.contains(overrule) {
                                println!(
                                    "Prohibited value found: \"{}\" at {}:{}",
                                    r.as_str(),
                                    entry.path().to_str().unwrap_or("<unknown path>"),
                                    line_number
                                );
                                println!("{}\n", line);
                                return false;
                            }
                        }
                        line_number += 1;
                    }
                    return true;
                }
                return false;
            }
            false
        }
        None => false,
    }
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
                    success &= check_file(entry, &extra_patterns, overrule);
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

            // Evaluate every specific target.
            for specific in configuration.specific {
                check_specific(&specific, &common_patterns, configuration.overrule.as_str());
            }
        } else {
            eprintln!("Invalid configuration.");
        }
    } else {
        println!("Could not open configuration file {:?}", path);
    }
}
