use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use glob::glob;
use indicatif::ParallelProgressIterator;
use md5::{Digest, Md5};
use rayon::prelude::*;
use serde::Serialize;
use serde_json::to_string as to_json;

#[derive(Debug)]
enum OutputFormat {
    Human,
    Json,
}

impl FromStr for OutputFormat {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "human" => Ok(OutputFormat::Human),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("Cannot map {s} to any output format").into()),
        }
    }
}

#[derive(Serialize)]
struct FileHashEntry {
    hash: String,
    paths: Vec<String>,
}

impl FileHashEntry {
    fn new(hash: String, paths: Vec<String>) -> FileHashEntry { FileHashEntry { hash, paths } }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The glob pattern to look for duplicates
    #[arg(short = 'p', long = "pattern", default_value_t = String::from("./**/*"))]
    pattern: String,

    /// The desire output format
    #[arg(short = 'o', long = "output", default_value_t = String::from("human"))]
    output: String,
}

fn main() {
    let args = Args::parse();
    let output_format: OutputFormat = OutputFormat::from_str(&args.output).unwrap();

    let mut files: Vec<PathBuf> = glob(&args.pattern).unwrap()
        .map(|glob_result| glob_result.unwrap())
        .filter(|path| path.is_file()).collect();

    sort_by_file_size_desc(&mut files);

    let file_hash_entries: Vec<_> = files.par_iter()
        .progress()
        .map(|file_path| {
            let mut file = File::open(file_path.clone()).unwrap();
            let hex_hash = hash_content(&mut file);
            (hex_hash, file_path.to_str().unwrap().to_string())
        })
        .collect::<Vec<(String, String)>>()
        .into_iter()
        .fold(HashMap::new(), |mut result_map, (hash, path)| {
            let paths_for_hash = result_map.entry(hash).or_insert(vec![]);
            paths_for_hash.push(path);
            result_map
        })
        .into_iter()
        .filter(|(_hash, paths)| paths.len() > 1)
        .map(|(hash, paths)| FileHashEntry::new(hash, paths))
        .collect();

    match output_format {
        OutputFormat::Json => println!("{}", to_json(&file_hash_entries).unwrap()),
        OutputFormat::Human => print_human_output(file_hash_entries)
    }
}

fn sort_by_file_size_desc(files: &mut Vec<PathBuf>) {
    files.sort_by(|a, b| a.metadata().unwrap().len().partial_cmp(&b.metadata().unwrap().len()).unwrap());
}

fn print_human_output(file_hash_entries: Vec<FileHashEntry>) {
    for file_hash_entry in file_hash_entries {
        println!("Found {} occurrences:", file_hash_entry.paths.len());
        for p in file_hash_entry.paths {
            println!("Path: {}", p);
        }
        println!("MD5 Hash: {}\n", file_hash_entry.hash);
    }
}

fn hash_content(reader: &mut dyn Read) -> String {
    let digest = {
        let mut hasher = Md5::new();
        let mut buffer = vec![0; 1024];
        loop {
            let count = reader.read(&mut buffer).unwrap();
            if count == 0 { break; }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };
    base16ct::lower::encode_string(&digest)
}


#[cfg(test)]
mod tests {
    use super::*;

    // hash generated with https://www.md5hashgenerator.com/
    #[test]
    fn hash_content_should_hash_simple_reader_content() {
        let mut reader = "asdf1234341!@#$asdf🥺".as_bytes();
        let hash = hash_content(&mut reader);
        assert_eq!(String::from("8526695f5313baed2e42434180636d66"), hash);
    }
}