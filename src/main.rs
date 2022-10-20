use std::env;
use std::fs;
use std::collections::HashMap;
use ssh::*;

// offload hostname cargo run

fn main() {
    let directory = env::current_dir().expect("Error: could not find current directory").to_str().unwrap().to_owned();
    let config_path = directory + "/offload.toml";

    println!("Loading config from {}\n", config_path);

    let config_contents = fs::read_to_string(config_path).expect("Error: could not open config file");

    let config_map = parse_config(config_contents);

    println!("Config read:\n{:#?}", config_map)

    //parse clargs

    //check git repo/branch

    //connect to other computer over ssh

    //navigate to repo

    //change branch

    //pull changes

    //run command

    //pipe outputs

    //pipe output files?
}

fn parse_config(config: String) -> HashMap<String, String> {
    let mut config_map: HashMap<String, String> = HashMap::new();

    let binding = config
        .chars()
        .filter(|c| !c.is_whitespace() | (*c == '\n'))
        .collect::<String>();

    let pairs = binding
        .split("\n")
        .map(|line| line.split("=").into_iter())
        .map(|mut elems| (elems.nth(0), elems.nth(0)))
        .map(|(key, value)| (key.expect("Invalid config file").to_owned(), value.expect("Invalid config file").to_owned()));
        
    for pair in pairs {
        config_map.insert(pair.0, pair.1);
    }

    config_map
}
