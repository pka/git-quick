mod index;
#[allow(dead_code)]
mod path_util;
#[allow(dead_code)]
mod select;
#[allow(dead_code)]
mod theme;

fn main() {
    let mut index = index::Index::new();
    let repos = index.get_repos();

    let selection = select::Select::with_theme(&theme::ColorfulTheme::default())
        .with_prompt("Select repo")
        .default(0)
        .paged(true)
        .items(&repos[..])
        .interact()
        .unwrap();
    println!("Selected: {}", repos[selection]);
}
