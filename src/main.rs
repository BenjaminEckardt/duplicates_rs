extern crate crypto;
extern crate glob;

use crypto::md5::Md5;
use crypto::digest::Digest;
use glob::glob;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let pattern = env::args().nth(1).unwrap_or("./**/*".to_string());

    let results: HashMap<String, Vec<String>> = glob(&pattern).unwrap().into_iter()
        .map(|path_result| path_result.unwrap())
        .filter(|path| !path.is_dir())
        .map(|file_path| {
            let mut file = File::open(file_path.clone()).unwrap();
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();

            let mut hasher = Md5::new();
            hasher.input(buffer.as_slice());
            let hash = hasher.result_str();

            (hash, file_path.to_str().unwrap().to_string())
        }).fold(HashMap::new(), |mut result_map, hash_to_path| {
            if !result_map.contains_key(&hash_to_path.0) {
                result_map.insert(hash_to_path.0, vec![hash_to_path.1]);
            } else {
                result_map.get_mut(&hash_to_path.0).unwrap().push(hash_to_path.1);
            }
            result_map
        });

    for (hash, paths) in results.iter() {
        if paths.len() > 1 {
            println!("Found {} ocurrences:", paths.len());
            for p in paths {
                println!("Path: {}", p);
            }
            println!("MD5-Hash: {}\n", hash);
        }
    }
}
