use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::{self, Command as ProcessCommand};


#[derive(Serialize, Deserialize)]
struct TestResult {
    name: String,
    status: String,
    message: Option<String>,
    line_no: Option<u32>,
    execution_time: String,
    score: i32,
}

#[derive(Serialize, Deserialize)]
struct Results {
    version: u32,
    status: String,
    tests: Vec<TestResult>,
    max_score: i32,
}

fn main() {
    let matches = Command::new("Run Tests")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("Run the tests of a Rust exercise.")
        .arg(
            Arg::new("input")
                .help("Directory where the [EXERCISE.rs] file is located")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .help("Directory where the results.json will be written")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::new("max_score")
                .help("Max amount of points the test suite is worth")
                .required(false)
                .index(3)
                .value_parser(clap::value_parser!(i32)),
        )
        .get_matches();

    let input_dir = matches.get_one::<String>("input").unwrap();
    let output_dir = matches.get_one::<String>("output").unwrap();
    let max_score: i32 = *matches.get_one::<i32>("max_score").unwrap_or_else(|| {
        eprintln!("Error parsing max_score");
        process::exit(1);
    });

    if let Err(e) = validate_directory(input_dir) {
        eprintln!("Error with input directory: {}", e);
        process::exit(1);
    }

    if let Err(e) = validate_directory(output_dir) {
        eprintln!("Error with output directory: {}", e);
        process::exit(1);
    }

    let cargo_args: Vec<&str> = matches
        .get_many::<String>("cargo_args")
        .unwrap_or_default()
        .map(|s| s.as_str())
        .collect();

    run_tests(input_dir, output_dir, max_score, &cargo_args);
}

fn validate_directory(dir: &str) -> Result<(), String> {
    let path = PathBuf::from(dir);
    if !path.exists() {
        return Err(format!("Directory '{}' does not exist", dir));
    }
    if !path.is_dir() {
        return Err(format!("'{}' is not a directory", dir));
    }
    Ok(())
}

fn run_tests(input: &str, output: &str, max_score: i32, cargo_args: &[&str]) {
    println!("Running tests in directory: {}", input);
    println!("Output will be written to: {}", output);
    println!("Max score: {}", max_score);
    println!("Additional cargo arguments: {:?}", cargo_args);

    // Run the tests and capture the output
    let test_output = ProcessCommand::new("cargo")
        .arg("+nightly")
        .arg("test")
        .arg("--")
        .arg("-Z")
        .arg("unstable-options")
        .arg("--format")
        .arg("json")
        .arg("--report-time")
        .args(cargo_args)
        .current_dir(input)
        .output()
        .expect("Failed to execute cargo test");

    // Print the raw JSON output for debugging
    let raw_output = String::from_utf8_lossy(&test_output.stdout);
    println!("Raw JSON output from cargo test:\n{}", raw_output);

    // Transform the test results into the desired format
    let mut tests = Vec::new();
    let mut total_tests = 0;
    let mut passed_tests = 0;

    for line in raw_output.lines() {
        if let Ok(event) = serde_json::from_str::<Value>(line) {
            match event.get("type").and_then(|t| t.as_str()) {
                Some("test") => {
                    if event.get("event").and_then(|e| e.as_str()) == Some("started") {
                        continue;
                    }
                    let name = event.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                    let event_status = event.get("event").and_then(|s| s.as_str()).unwrap_or("fail").to_string();
                    let execution_time = event.get("exec_time").and_then(|t| t.as_str()).unwrap_or("0ms").to_string();
                    let score = if event_status == "ok" { 1 } else { 0 };

                    let status = if event_status == "ok" { "pass".to_string() } else { "fail".to_string() };

                    if event_status == "ok" {
                        passed_tests += 1;
                    }

                    total_tests += 1;

                    tests.push(TestResult {
                        name,
                        status,
                        message: None,
                        line_no: None,
                        execution_time,
                        score
                    });
                }
                _ => {}
            }
        }
    }

    let status = if passed_tests == total_tests { "pass" } else { "fail" };

    let results = Results {
        version: 1,
        status: status.to_string(),
        tests,
        max_score
    };

    // Write the transformed results to the output directory
    let results_path = PathBuf::from(output).join("results.json");
    let results_json = serde_json::to_string_pretty(&results).expect("Failed to serialize results");
    fs::write(&results_path, results_json).expect("Unable to write results.json");

    // Print the results
    println!("Test results written to: {}", results_path.display());
}