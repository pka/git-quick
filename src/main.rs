mod index;
#[allow(dead_code)]
mod path_util;
mod repo;
#[allow(dead_code)]
mod select;
#[allow(dead_code)]
mod theme;

use console::{Style, Term};
use select::{Item, SelectCommand};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::Path;
use std::process::{Command, Stdio};
use sublime_fuzzy::{best_match, Match};

fn search(input: &String, repos: &Vec<String>) -> Vec<Item> {
    if input.is_empty() {
        return repos
            .iter()
            .enumerate()
            .map(|(i, repo)| Item {
                text: repo.clone(),
                item_key: i,
                match_info: Match::new(),
            })
            .collect();
    }
    let mut matches = repos
        .iter()
        .enumerate()
        .map(|(idx, repo)| (idx, best_match(&input, repo)))
        .filter(|(_i, m)| m.is_some())
        .map(|(i, m)| (i, m.unwrap()))
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| b.1.score().cmp(&a.1.score()));

    let highlight = Style::new().on_blue();
    let items: Vec<Item> = matches
        .iter()
        .map(|(i, m)| {
            let r = &repos[*i];
            let mut outstr = String::new();
            let mut idx = 0;
            for (match_idx, len) in m.continuous_matches() {
                outstr.push_str(&r[idx..match_idx]);
                idx = match_idx + len;
                outstr.push_str(&format!("{}", highlight.apply_to(&r[match_idx..idx])));
            }
            if idx < r.len() {
                outstr.push_str(&r[idx..r.len()]);
            }
            Item {
                text: outstr,
                item_key: *i,
                match_info: m.clone(),
            }
        })
        .collect();
    items
}

fn exec_command(dir: &str, cmd: &str, args: &[&str]) -> Result<(), Error> {
    std::env::set_current_dir(&Path::new(dir))?;
    let term = Term::stdout();
    let cmd_style = Style::new().cyan();
    println!(
        "{}",
        cmd_style.apply_to(&format!("{} {}", cmd, args.join(" ")))
    );
    let mut numlines = 1;
    let stdout = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let reader = BufReader::new(stdout);

    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            numlines += 1;
            println!("{}", line);
        });

    term.move_cursor_up(numlines)?;
    Ok(())
}

fn main() {
    let mut index = index::Index::new();
    let repos = index.get_repos_by_last_commit();

    // User input for search
    let mut input = String::new();
    let term = Term::stderr();
    loop {
        let items = search(&input, &repos);
        let theme = theme::ColorfulTheme::default();
        let mut select = select::Select::with_theme(&theme);
        match select
            .default(0)
            .paged(true)
            .items(items)
            .interact_on_opt(&term)
        {
            Ok(SelectCommand::CharInput { ch }) => match ch {
                _ => {
                    input.push(ch);
                }
            },
            Ok(SelectCommand::Command { ch, item_key }) => match ch {
                '\u{7f}' => {
                    // backspace
                    input.pop();
                }
                '\u{10}' => {
                    // ctrl-p
                    exec_command(&repos[item_key], "git", &["pull"]).unwrap();
                    select.reset_cursor(&term);
                }
                '\u{13}' => {
                    // ctrl-s
                    exec_command(&repos[item_key], "git", &["status"]).unwrap();
                    select.reset_cursor(&term);
                }
                '\u{1b}' => {
                    // home
                    input.clear(); // reset search
                }
                _ => {
                    println!("Unknown command key {:?}", ch);
                }
            },
            Ok(SelectCommand::Select { item_key }) => {
                // cwd is only kept if script is started in same shell (source)
                let _ = std::env::set_current_dir(&Path::new(&repos[item_key]));
                break;
            }
            Ok(SelectCommand::Quit) => break,
            _ => {}
        }
    }
}
