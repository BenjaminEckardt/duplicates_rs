use md5::{Md5, Digest};
use glob::glob;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;

fn hash_content(reader: &mut dyn Read) -> String {
    let digest = {
        let mut hasher = Md5::new();
        let mut buffer = vec![0; 1024 * 1024];
        loop {
            let count = reader.read(&mut buffer).unwrap();
            if count == 0 { break }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };
    base16ct::lower::encode_string(&digest)
}

fn main() {
    let pattern = env::args().nth(1).unwrap_or("./**/*".to_string());

    glob(&pattern).unwrap().into_iter()
        .map(|path_result| path_result.unwrap())
        .filter(|path| !path.is_dir())
        .map(|file_path| {
            let mut file = File::open(file_path.clone()).unwrap();
            let hex_hash = hash_content(&mut file);
            (hex_hash, file_path.to_str().unwrap().to_string())
        })
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
