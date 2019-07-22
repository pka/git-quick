use std::io;

use std::ops::Rem;

use crate::theme::{get_default_theme, SelectionStyle, TermThemeRenderer, Theme};

use console::{Key, Term};

use sublime_fuzzy::Match;

pub struct Item {
    pub text: String,
    pub item_key: usize,
    pub match_info: Match,
}

pub enum SelectCommand {
    CharInput { ch: char },
    Select { item_key: usize },
    Command { ch: char, item_key: usize },
    Quit,
}

/// Renders a selection menu.
pub struct Select<'a> {
    default: usize,
    items: Vec<Item>,
    prompt: Option<String>,
    clear: bool,
    theme: &'a Theme,
    paged: bool,
    page_size: usize,
}

impl<'a> Select<'a> {
    /// Creates the prompt with a specific text.
    pub fn new() -> Select<'static> {
        Select::with_theme(get_default_theme())
    }

    /// Same as `new` but with a specific theme.
    pub fn with_theme(theme: &'a Theme) -> Select<'a> {
        Select {
            default: !0,
            items: vec![],
            prompt: None,
            clear: true,
            theme: theme,
            paged: false,
            page_size: 0,
        }
    }
    /// Enables or disables paging
    pub fn paged(&mut self, val: bool) -> &mut Select<'a> {
        self.paged = val;
        if self.paged {
            self.page_size = 10;
        }
        self
    }
    /// Sets the clear behavior of the menu.
    ///
    /// The default is to clear the menu.
    pub fn clear(&mut self, val: bool) -> &mut Select<'a> {
        self.clear = val;
        self
    }

    /// Sets a default for the menu
    pub fn default(&mut self, val: usize) -> &mut Select<'a> {
        self.default = val;
        self
    }

    /// Add a single item to the selector.
    pub fn item(&mut self, item: Item) -> &mut Select<'a> {
        self.items.push(item);
        self
    }

    /// Adds multiple items to the selector.
    pub fn items(&mut self, items: Vec<Item>) -> &mut Select<'a> {
        for item in items {
            self.items.push(item);
        }
        self
    }

    /// Prefaces the menu with a prompt.
    ///
    /// When a prompt is set the system also prints out a confirmation after
    /// the selection.
    pub fn with_prompt(&mut self, prompt: &str) -> &mut Select<'a> {
        self.prompt = Some(prompt.to_string());
        self
    }

    /// Enables user interaction and returns the result.
    ///
    /// The index of the selected item.
    /// The dialog is rendered on stderr.
    pub fn interact(&self) -> io::Result<SelectCommand> {
        self.interact_on(&Term::stderr())
    }

    /// Enables user interaction and returns the result.
    ///
    /// The index of the selected item. None if the user
    /// cancelled with Esc or 'q'.
    /// The dialog is rendered on stderr.
    pub fn interact_opt(&self) -> io::Result<SelectCommand> {
        self._interact_on(&Term::stderr(), true)
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on(&self, term: &Term) -> io::Result<SelectCommand> {
        self._interact_on(term, false)
        // io::Error::new(
        //    io::ErrorKind::Other,
        //    "Quit not allowed in this case",
        //))
    }

    /// Like `interact` but allows a specific terminal to be set.
    pub fn interact_on_opt(&self, term: &Term) -> io::Result<SelectCommand> {
        self._interact_on(term, true)
    }

    /// Like `interact` but allows a specific terminal to be set.
    fn _interact_on(&self, term: &Term, allow_quit: bool) -> io::Result<SelectCommand> {
        let mut page = 0;
        let mut capacity = self.items.len();
        if self.paged {
            capacity = self.page_size.min(term.size().0 as usize - 1);
        }
        let pages = (self.items.len() / capacity) + 1;
        let mut render = TermThemeRenderer::new(term, self.theme);
        let mut sel = self.default;
        if let Some(ref prompt) = self.prompt {
            render.prompt(prompt)?;
        }
        let mut size_vec = Vec::new();
        for items in self.items.iter().as_slice() {
            let size = &items.text.len();
            size_vec.push(size.clone());
        }
        loop {
            for (idx, item) in self
                .items
                .iter()
                .enumerate()
                .skip(page * capacity)
                .take(capacity)
            {
                render.selection(
                    &item.text,
                    if sel == idx {
                        SelectionStyle::MenuSelected
                    } else {
                        SelectionStyle::MenuUnselected
                    },
                )?;
            }
            match term.read_key()? {
                Key::ArrowDown => {
                    if sel == !0 {
                        sel = 0;
                    } else {
                        sel = (sel as u64 + 1).rem(self.items.len() as u64) as usize;
                    }
                }
                Key::Escape => {
                    if allow_quit {
                        if self.clear {
                            term.clear_last_lines(self.items.len())?;
                        }
                        return Ok(SelectCommand::Quit);
                    }
                }
                Key::ArrowUp => {
                    if sel == !0 {
                        sel = self.items.len() - 1;
                    } else {
                        sel = ((sel as i64 - 1 + self.items.len() as i64)
                            % (self.items.len() as i64)) as usize;
                    }
                }
                Key::ArrowLeft => {
                    if self.paged {
                        if page == 0 {
                            page = pages - 1;
                        } else {
                            page = page - 1;
                        }
                        sel = page * capacity;
                    }
                }
                Key::ArrowRight => {
                    if self.paged {
                        if page == pages - 1 {
                            page = 0;
                        } else {
                            page = page + 1;
                        }
                        sel = page * capacity;
                    }
                }

                Key::Enter if sel != !0 => {
                    if self.clear {
                        render.clear()?;
                    }
                    if let Some(ref prompt) = self.prompt {
                        render.single_prompt_selection(prompt, &self.items[sel].text)?;
                    }
                    return Ok(SelectCommand::Select {
                        item_key: self.items[sel].item_key,
                    });
                }

                Key::Char(ch) if ch.is_alphanumeric() => {
                    if self.clear {
                        render.clear()?;
                    }
                    return Ok(SelectCommand::CharInput { ch });
                }

                Key::Char(ch) if !ch.is_alphanumeric() => {
                    return Ok(SelectCommand::Command {
                        ch,
                        item_key: self.items[sel].item_key,
                    });
                }
                _ => {}
            }
            if sel < page * capacity || sel >= (page + 1) * capacity {
                page = sel / capacity;
            }
            render.clear_preserve_prompt(&size_vec)?;
        }
    }
}
