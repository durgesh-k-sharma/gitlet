use gitlet::commands;
use gitlet::error::{GitletError, Result};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please enter a command.");
        process::exit(0);
    }

    let command = &args[1];
    let result = run_command(command, &args[2..]);

    if let Err(err) = result {
        println!("{}", err);
        process::exit(0);
    }
}

fn run_command(command: &str, operands: &[String]) -> Result<()> {
    match command {
        "init" => {
            if !operands.is_empty() {
                return Err(GitletError::IncorrectOperands);
            }
            commands::init::execute()
        }
        "add" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::add::execute(&operands[0])
        }
        "commit" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::commit::execute(&operands[0]).map(|_| ())
        }
        "rm" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::rm::execute(&operands[0])
        }
        "log" => {
            if !operands.is_empty() {
                return Err(GitletError::IncorrectOperands);
            }
            commands::log::execute()
        }
        "global-log" => {
            if !operands.is_empty() {
                return Err(GitletError::IncorrectOperands);
            }
            commands::global_log::execute()
        }
        "find" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::find::execute(&operands[0])
        }
        "status" => {
            if !operands.is_empty() {
                return Err(GitletError::IncorrectOperands);
            }
            commands::status::execute()
        }
        "checkout" => {
            match operands.len() {
                1 => {
                    // gitlet checkout [branch name]
                    commands::checkout::checkout_branch(&operands[0])
                }
                2 => {
                    // gitlet checkout -- [file name]
                    if operands[0] != "--" {
                        return Err(GitletError::IncorrectOperands);
                    }
                    commands::checkout::checkout_file_head(&operands[1])
                }
                3 => {
                    // gitlet checkout [commit id] -- [file name]
                    if operands[1] != "--" {
                        return Err(GitletError::IncorrectOperands);
                    }
                    commands::checkout::checkout_file_commit(&operands[0], &operands[2])
                }
                _ => Err(GitletError::IncorrectOperands),
            }
        }
        "branch" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::branch::execute(&operands[0])
        }
        "rm-branch" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::rm_branch::execute(&operands[0])
        }
        "reset" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::reset::execute(&operands[0])
        }
        "merge" => {
            if operands.len() != 1 {
                return Err(GitletError::IncorrectOperands);
            }
            commands::merge::execute(&operands[0])
        }
        _ => Err(GitletError::Other(
            "No command with that name exists.".to_string(),
        )),
    }
}
