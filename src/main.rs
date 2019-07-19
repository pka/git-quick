#[allow(dead_code)]
mod select;
#[allow(dead_code)]
mod theme;

fn main() {
    let selections = &[
        "Ice Cream",
        "Vanilla Cupcake",
        "Chocolate Muffin",
        "A Pile of sweet, sweet mustard",
    ];

    let selection = select::Select::with_theme(&theme::ColorfulTheme::default())
        .with_prompt("Pick your flavor")
        .default(0)
        .items(&selections[..])
        .interact()
        .unwrap();
    println!("Enjoy your {}!", selections[selection]);
}
