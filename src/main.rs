mod index;
#[allow(dead_code)]
mod path_util;
#[allow(dead_code)]
mod select;
#[allow(dead_code)]
mod theme;

use select::SelectCommand;
use std::path::Path;
use sublime_fuzzy::best_match;

fn search(input: &String, repos: &Vec<String>) -> Vec<String> {
    if input.is_empty() {
        return repos.clone();
    }
    let mut matches = repos
        .iter()
        .map(|repo| (repo, best_match(&input, repo)))
        .filter(|(_r, m)| m.is_some())
        .map(|(r, m)| (r, m.unwrap()))
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| b.1.score().cmp(&a.1.score()));
    for i in 0..3.min(matches.len()) {
        println!("{:?}", matches[i]);
    }

    let items = matches
        .iter()
        .map(|(r, _m)| r.clone().clone())
        .collect::<Vec<_>>();
    items
}

fn main() {
    let mut index = index::Index::new();
    let repos = index.get_repos();

    let mut input = String::new();
    loop {
        let items = search(&input, &repos);
        match select::Select::with_theme(&theme::ColorfulTheme::default())
            .default(0)
            .paged(true)
            .items(&items[..])
            .interact_opt()
        {
            Ok(SelectCommand::CharInput { ch }) => match ch {
                '\u{7f}' => {
                    input.pop();
                }
                _ => {
                    input.push(ch);
                }
            },
            Ok(SelectCommand::Select { row }) => {
                println!("{}", items[row]);
                // cwd is only kept if script is started in same shell (source)
                let _ = std::env::set_current_dir(&Path::new(&items[row]));
                break;
            }
            Ok(SelectCommand::Quit) => break,
            _ => {}
        }
    }
}
