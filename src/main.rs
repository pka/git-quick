mod index;
#[allow(dead_code)]
mod path_util;
#[allow(dead_code)]
mod select;
#[allow(dead_code)]
mod theme;

use sublime_fuzzy::best_match;

fn main() {
    let mut index = index::Index::new();
    let repos = index.get_repos();

    let input = "RustTile";
    let mut matches = repos
        .iter()
        .map(|repo| (repo, best_match(input, repo)))
        .filter(|(_r, m)| m.is_some())
        .map(|(r, m)| (r, m.unwrap()))
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| b.1.score().cmp(&a.1.score()));
    for i in 0..3.min(matches.len()) {
        println!("{:?}", matches[i]);
    }

    let items = matches.iter().map(|(r, _m)| r).collect::<Vec<_>>();
    let selection = select::Select::with_theme(&theme::ColorfulTheme::default())
        .with_prompt("Select repo")
        .default(0)
        .paged(true)
        .items(&items[..])
        .interact()
        .unwrap();
    println!("Selected: {}", repos[selection]);
}
