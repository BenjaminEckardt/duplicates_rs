use md5::{Md5, Digest};
use glob::glob;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use clap::Parser;
use rayon::prelude::*;
use indicatif::ParallelProgressIterator;

fn hash_content(reader: &mut dyn Read) -> String {
    let digest = {
        let mut hasher = Md5::new();
        let mut buffer = vec![0; 1024];
        loop {
            let count = reader.read(&mut buffer).unwrap();
            if count == 0 { break }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };
    base16ct::lower::encode_string(&digest)
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The glob pattern to look for duplicates
    #[arg(short = 'p', long = "pattern", default_value_t = String::from("./**/*"))]
    pattern: String,
}

fn main() {
    let args = Args::parse();

    let mut files: Vec<PathBuf> = glob(&args.pattern).unwrap()
        .map(|glob_result| glob_result.unwrap())
        .filter(|path| path.is_file()).collect();

    files.sort_by(|a, b| a.metadata().unwrap().len().partial_cmp(&b.metadata().unwrap().len()).unwrap());

    files.par_iter()
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
        .for_each(|(hash, paths)| {
            println!("Found {} occurrences:", paths.len());
            for p in paths {
                println!("Path: {}", p);
            }
            println!("Sha256 Hash: {}\n", hash);
        });
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