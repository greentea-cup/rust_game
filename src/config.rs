#![allow(dead_code, unused)]

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, self},
    path::Path,
};

pub fn load_config(path: &Path) -> io::Result<HashMap<String, String>> {
  Ok(HashMap::new())
}

pub fn save_config(path: &Path) -> io::Result<()> {
  Ok(())
}

fn open_settings(path: &Path) -> File {
    println!("Opening file {}", path.display());
    match OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
    {
        Err(why) => panic!("Cannot create or open file `{}`: {}", path.display(), why),
        Ok(file) => file,
    }
}

fn parse_settings(file: &File) -> HashMap<String, String> {
    BufReader::new(file)
        .lines()
        .filter_map(|s| match s {
            Ok(l) => match l.split_once('=') {
                Some((k, v)) => Some((k.to_owned(), v.to_owned())),
                None => None,
            },
            Err(why) => panic!("Error parsing settings: {}", why),
        })
        .collect::<HashMap<_, _>>()
}
