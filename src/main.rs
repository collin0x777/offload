use std::env;
use std::time::Duration;
use futures::{FutureExt, StreamExt};

use git2::{Repository, Status, StatusOptions};
use openssh::{Session, KnownHosts, Stdio};
use futures_lite::future::race;
use tokio::{io::AsyncBufReadExt, io::BufReader, sync::watch, time::timeout};
use tokio_stream::wrappers::LinesStream;

fn get_repo_info(allow_uncommitted_changes: bool) -> Result<(String, String), String>{
    let repo = Repository::open(".").map_err(|err| format!("Failed to open repository: {}", err.message()))?;
    let remote = repo.find_remote("origin").map_err(|err| format!("Failed to find remote repository: {}", err.message()))?;
    let remote_url = remote.url().ok_or(String::from("Failed to get remote URL"))?;
    let head = repo.head().map_err(|err| format!("Failed to get HEAD: {}", err.message()))?;
    let branch_name = head.shorthand().ok_or(String::from("Failed to get branch name"))?;

    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|err| format!("Failed to get statuses: {}", err.message()))?;
    
    let mut is_clean = true;
    for entry in statuses.iter() {
        if entry.status() != Status::CURRENT {
            is_clean = false;
            println!("Uncommitted changes: {}", entry.path().unwrap());
        }
    }

    if !is_clean {
        if allow_uncommitted_changes {
            println!("There are uncommitted changes. Continuing anyways as specified...");
        } else {
            println!("There are uncommitted changes. Aborting...");
            return Err(String::from("Uncommitted changes"));
        }
    }

    Ok((branch_name.to_owned(), remote_url.to_owned()))
}


fn parse_args(args: Vec<String>) -> Result<(String, String), String> {
    if args.len() < 3 {
        println!("Usage: {} <hostname> <command>", args[0]);
        return Err(String::from("Not enough arguments"));
    }
    let host = &args[1];
    let command = &args[2..];

    return Ok((host.to_owned(), command.join(" ")));
}

async fn create_tmp_repository(session: &Session, branch_name: &str, remote_url: &str) -> Result<bool, openssh::Error> {
    let repo_location = format!("/tmp/{}", branch_name);
    let directory_exists = session.command("test").arg("-d").arg(repo_location.as_str()).status().await.unwrap().success();

    if directory_exists {
        println!("Directory already exists, skipping clone");
        return Ok(true);
    }

    let status = session.command("git").args(["clone", "--single-branch", "--branch", branch_name, remote_url, repo_location.as_str()]).status().await.unwrap();

    return Ok(status.success());
}

async fn start_shell(session: &Session, initial_command: &String, tmp_repository_location: &String) -> Result<Option<i32>, openssh::Error> {

    let mut child = session
        .shell(format!("cd {} && {}", tmp_repository_location, initial_command))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .await?;

    let child_stdout = child.stdout().take().expect("!stdout");
    let child_stderr = child.stderr().take().expect("!stderr");
    let _child_stdin = child.stdin().take().expect("!stdin");
    
    let stdout_lines = LinesStream::new(BufReader::new(child_stdout).lines());
    let stderr_lines = LinesStream::new(BufReader::new(child_stderr).lines());
    
    stdout_lines.for_each(|s| async move { 
        if let Ok(s) = s {
            println!("> {}", s);
        }
    }).await;
    stderr_lines.for_each(|s| async move { 
        if let Ok(s) = s {
            println!("! {}", s);
        }
    }).await;

    return child.wait().await.map(|exit_status| exit_status.code());
}

#[tokio::main]
async fn main() {
    // Check the current git repository and branch
    let (branch_name, remote_url) = get_repo_info(true).expect("Failed to get repository info");
    let (host_arg, command) = parse_args(env::args().collect()).expect("Failed to parse arguments");

    let session = 
        timeout(Duration::from_secs(5), Session::connect(format!("{}", host_arg), KnownHosts::Strict))
        .await
        .expect("Timed out connecting to host")
        .expect("Failed to connect to host");

    let exit_status = create_tmp_repository(&session, &branch_name, &remote_url).await.expect("Failed to create repository on host");
    let tmp_repository_location = format!("/tmp/{}", branch_name);
    assert!(exit_status, "Failed to create repository on host");

    let (tx, mut rx) = watch::channel(false);
    let send_shutdown = move || tx.send(true).expect("Error sending Ctrl-C signal");
    ctrlc::set_handler(send_shutdown).expect("Error setting Ctrl-C handler");
    let escape_future = rx.changed().map(|_| Ok(Some(130)));
    let exit_status = race(start_shell(&session, &command, &tmp_repository_location), escape_future).await.expect("Error in running shell");

    session.close().await.expect("Error closing session");
    
    std::process::exit(exit_status.unwrap_or(1));
}