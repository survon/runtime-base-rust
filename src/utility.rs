use md5::{Md5, Digest};
use std::fs;

pub fn compute_md5(path: &str) -> String {
    let contents = fs::read(path).expect("Failed to read file");
    let mut hasher = Md5::new();
    hasher.update(contents);
    format!("{:x}", hasher.finalize())
}
