extern crate git2;
extern crate ssh2;

use std::env;
use std::process::Command;

use git2::{Repository, Status, StatusOptions};
use ssh2::Session;

fn main() {
    // Check the current git repository and branch
    let repo = Repository::open(".").expect("failed to open repository");
    let head = repo.head().expect("failed to get HEAD");
    let branch_name = head.shorthand().expect("failed to get branch name");
    println!("Current repository: {}", repo.path().display().to_string());
    println!("Current branch: {}", branch_name);

    // Ensure that all committed changes are pushed
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    let statuses = repo.statuses(Some(&mut opts)).expect("failed to get statuses");
    let mut is_clean = true;
    for entry in statuses.iter() {
        if entry.status() != Status::CURRENT {
            is_clean = false;
            println!("Uncommitted changes: {}", entry.path().unwrap());
        }
    }
    if !is_clean {
        println!("There are uncommitted changes. Please commit and push them before continuing.");
        return;
    }

    // Get the host and command to run from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <host> <command>", args[0]);
        return;
    }
    let host = &args[1];
    let command = &args[2..];

    // SSH into the host
    // let tcp = std::net::TcpStream::connect(host).expect("failed to connect");
    // let mut sess = Session::new().expect("failed to create session");
    // sess.set_tcp_stream(tcp);
    // sess.handshake(&tcp).expect("failed to handshake");
    // sess.userauth_password("username", "password").expect("failed to authenticate");

    // Locate the repository on the host or clone it if it's missing
    // let mut sftp = sess.sftp().expect("failed to start sftp session");
    // if sftp.stat("/path/to/repo").is_err() {
    //     println!("Repository not found on host. Cloning it now...");
    //     let _ = Command::new("git")
    //         .arg("clone")
    //         .arg("/path/to/repo")
    //         .output()
    //         .expect("failed to execute git clone command");
    // } else {
    //     println!("Repository found on host.");
    // }

    // Run the specified command
    // let output = sess.exec(command).expect("failed to execute command");
    // let mut s = String::new();
    // output
    //     .read_to_string(&mut s)
    //     .expect("failed to read output");
    // println!("{}", s);
}
