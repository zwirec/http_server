use std::fs::File;
use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use std::borrow::Borrow;
use std::io::Result;

pub struct Config {
    pub document_root: String,
    pub thread_num: usize,
    pub port: u16,
}

const DEFAULT_DOC_ROOT: &str = "./";
const DEFAULT_NUM_THREADS: usize = 16;
const DEFAULT_PORT: u16 = 8080;

impl Config {
    pub fn parse<S>(filepath: S) -> Result<Config> where S: Into<String> {
        let file = File::open(filepath.into())?;
        let buf_file = BufReader::new(&file);
        let mut params: HashMap<String, String> = HashMap::new();
        for line in buf_file.lines() {
            let line = line;
            let mut l = line.as_ref().unwrap().split_whitespace();
            params.insert(l.next().unwrap_or_default().to_owned(), l.next().clone().unwrap_or_default().to_owned());
        }
        Ok(
            Config {
                document_root: params.get("document_root").
                    unwrap_or(DEFAULT_DOC_ROOT.to_owned().borrow()).clone(),
                thread_num: params.get("thread_limit").
                    unwrap_or(DEFAULT_NUM_THREADS.to_string().borrow()).parse().unwrap(),
                port: params.get("listen").
                    unwrap_or(DEFAULT_PORT.to_string().borrow()).parse().unwrap(),
            })
    }

    pub fn default() -> Config {
        Config {
            document_root:DEFAULT_DOC_ROOT.to_owned(),
            thread_num: DEFAULT_NUM_THREADS,
            port: DEFAULT_PORT,
        }
    }
}