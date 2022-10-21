use std::collections::VecDeque;
use std::env;
use std::io;
use std::fmt;
use std::io::Write;
use std::str;
// use std::fs;
use std::collections::HashMap;
// use ssh::*;
use std::process::Command;

// offload hostname cargo run

fn main() {
    //parse clargs
    let mut args: VecDeque<String> = env::args().collect();
    args.pop_front();

    if args.len() < 2 {
        panic!("Invalid arguments: command must be of the form 'offload <hostname> <command>'")
    }

    let hostname = args.pop_front().unwrap();
    let command = args.iter().fold("".to_string(), |cmd, arg| cmd + " " + arg);

    //check git repo/branch - issue warning if there are uncommitted/unpushed changes
    let git_branch = execute_cmd("git branch --show-current").trim().to_string();

    //fetch remote url
    let remote_url = execute_cmd("git config --get remote.origin.url").trim().to_string();
    let repo_name = remote_url.trim_end_matches(".git").split("/").last().unwrap().to_string();

    println!("Hostname: {}\nCommand: {}\nBranch: {}\nRemote URL: {}\nRepository: {}", hostname, command, git_branch, remote_url, repo_name);

    //connect to other computer over ssh

    //find repo by remote url
    let repo_search = execute_cmd(&format!("rg -.l --no-messages {} ~/", remote_url));
    let mut repo_paths = repo_search.lines();

    println!("{}", repo_search);

    let repo_path = match repo_paths.next() {
        Some(path) => path.to_string(),
        None => {
            print!("No repositories were found which track the remote, please enter a directory for the repository to be cloned:");
        
            let mut directory = String::new();
            io::stdin().read_line(&mut directory)
                .expect("Failed to read line");

            execute_cmd(&format!("cd {}", directory));

            execute_cmd(&format!("git clone {}", remote_url));

            directory + &repo_name
        }
    };

    //navigate to repo
    execute_cmd(&format!("cd {}", repo_path));

    //change branch
    execute_cmd(&format!("git checkout {}", git_branch));

    //pull changes
    execute_cmd("git pull");

    //run command
    let command_output = 
        Command::new(command)
                .output()
                .unwrap();

    //pipe outputs
    io::stdout().write_all(&command_output.stdout).unwrap();
    io::stderr().write_all(&command_output.stderr).unwrap();

    //pipe output files?
}

fn execute_cmd(cmd: &str) -> String {
    let mut cmd_args = cmd.split_whitespace();

    let mut command = Command::new(cmd_args.next().unwrap());

    command.args(cmd_args);

    let output = command.output().unwrap();

    str::from_utf8(&output.stdout).unwrap().to_string()
}