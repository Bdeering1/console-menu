//! A simple yet powerful library for creating beautiful console menus in rust.
//!
//! Allows for easy creation of interactive console menus. A simple example:
//!
//! ```no_run
//! use console_menu::{Menu, MenuOption, MenuProps};
//! 
//! let menu_options = vec![
//!     MenuOption::new("option 1", || println!("option one!")),
//!     MenuOption::new("option 2", || println!("option two!")),
//!     MenuOption::new("option 3", || println!("option three!")),
//! ];
//! let mut menu = Menu::new(menu_options, MenuProps::default());
//! menu.show();
//! ```
//!
//! Menus can include a title, footer message, and any combination of [8-bit](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit)
//! colored backgrounds and text by configuring `MenuProps`. Menus that don't fit the console window are paginated.
//!
//! Menu controls are as follows:
//! 
//! | Key Bind | Action      |
//! | -------- | ----------- |
//! | ↓, ↑, ←, →, h, j, k, l | make selection        |
//! | enter    | confirm     |
//! | esc, q   | exit        |

use console::{Key, Term};

/// A collection of pre-selected color values to simplify menu theming.
pub mod color {
    pub const WHITE: u8 = 15;
    pub const LIGHT_GRAY: u8 = 7;
    pub const GRAY: u8 = 8;
    pub const BLUE: u8 = 32;
    pub const GREEN: u8 = 35;
    pub const PURPLE: u8 = 99;
    pub const RED: u8 = 160;
    pub const ORANGE: u8 = 208;
    pub const YELLOW: u8 = 220;
    pub const BLACK: u8 = 233;
    pub const DARK_GRAY:u8 = 236;
}

/// Stores configuration data passed to a `Menu` on creation.
///
/// Menus use [8-bit](https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit) colors to ensure
/// widespread terminal support. It should be noted that values from 0-15 will make colors vary
/// based on individual terminal settings.
///
/// Configure a subset of properties using the defaults and struct update syntax:
/// ```
/// # use console_menu::MenuProps;
/// let props = MenuProps {
///     title: "My Menu",
///     ..MenuProps::default()
/// };
/// ```
pub struct MenuProps<'a> {
    /// Displays above the list of menu options. Pass an empty string for no title.
    pub title: &'a str,
    /// Display below the list of menu options. Pass an empty string for no message.
    pub message: &'a str,
    /// If true, menu will exit immediately upon an option being selected.
    pub exit_on_action: bool,
    /// The background color for the menu.
    pub bg_color: u8,
    /// The foreground (text) color for the menu.
    pub fg_color: u8,
    /// Optional color for the title. If None, the foreground color will be used.
    pub title_color: Option<u8>,
    /// Optional color for the selected menu option. If None, the foreground color will be used.
    pub selected_color: Option<u8>,
    /// Optional color for the footer message. If None, the foreground color will be used.
    pub msg_color: Option<u8>,
}

/// ```
/// # use console_menu::MenuProps;
/// # fn default() -> MenuProps<'static> {
/// MenuProps {
///     title: "",
///     message: "",
///     exit_on_action: true,
///     bg_color: 8,
///     fg_color: 15,
///     title_color: None,
///     selected_color: None,
///     msg_color: Some(7),
/// }
/// # }
/// ```
impl Default for MenuProps<'_> {
    fn default() -> MenuProps<'static> {
        MenuProps {
            title: "",
            message: "",
            exit_on_action: true,
            bg_color: 8,
            fg_color: 15,
            title_color: None,
            selected_color: None,
            msg_color: Some(7),
        }
    }
}

/// An element in a `Menu`.
///
/// Consists of a label and a callback. Callbacks can be any function, including functions that
/// call nested menus:
///
/// ```
/// # use console_menu::{Menu, MenuOption, MenuProps};
/// let mut nested_menu = Menu::new(vec![], MenuProps::default());
/// let show_nested = MenuOption::new("show nested menu", move || nested_menu.show());
/// ```
pub struct MenuOption {
    pub label: String,
    pub action: Box<dyn FnMut()>,
}

impl MenuOption {
    pub fn new(label: &str, action: impl FnMut() + 'static) -> Self {
        Self {
            label: label.to_owned(),
            action: Box::new(action),
        }
    }
}

/// ```
/// # use console_menu::MenuOption;
/// # fn default() -> MenuOption {
/// MenuOption::new("exit", || {})
/// # }
/// ```
impl Default for MenuOption {
    fn default() -> MenuOption {
        MenuOption::new("exit", || {})
    }
}

/// Interactive console menu.
///
/// Create a menu by passing it a list of `MenuOption` and a `MenuProps`. Display using`.show()`.
///
/// ```no_run
/// # use console_menu::{Menu, MenuOption, MenuProps};
/// let menu_options = vec![
///     MenuOption::new("option 1", || println!("option one!")),
///     MenuOption::new("option 2", || println!("option two!")),
///     MenuOption::new("option 3", || println!("option three!")),
/// ];
/// let mut menu = Menu::new(menu_options, MenuProps::default());
/// menu.show();
/// ```
pub struct Menu {
    options: Vec<MenuOption>,
    title: Option<String>,
    message: Option<String>,
    exit_on_action: bool,
    bg_color: u8,
    fg_color: u8,
    title_color: u8,
    selected_color: u8,
    msg_color: u8,
    selected_option: usize,
    selected_page: usize,
    options_per_page: usize,
    num_pages: usize,
    page_start: usize,
    page_end: usize,
    max_width: usize,
}

impl Menu {
    pub fn new(options: Vec<MenuOption>, props: MenuProps) -> Self {
        assert!(!options.is_empty(), "Menu options cannot be empty!");

        let options_per_page: usize = (Term::stdout().size().0 - 6) as usize;
        let options_per_page = clamp(options_per_page, 1, options.len());
        let num_pages = ((options.len() - 1) / options_per_page) + 1;

        let mut max_width = options.iter().fold(0, |max, option| {
            let label_len = option.label.len();
            if label_len > max { label_len } else { max }
        });
        if props.title.len() > max_width {
            max_width = props.title.len()
        }
        if props.message.len() > max_width {
            max_width = props.message.len()
        }

        let mut menu = Self {
            options,
            title: (!props.title.is_empty()).then(|| props.title.to_owned()),
            message: (!props.message.is_empty()).then(|| props.title.to_owned()),
            exit_on_action: props.exit_on_action,
            bg_color: props.bg_color,
            fg_color: props.fg_color,
            title_color: props.title_color.unwrap_or(props.fg_color),
            selected_color: props.selected_color.unwrap_or(props.fg_color),
            msg_color: props.msg_color.unwrap_or(props.fg_color),
            selected_option: 0,
            selected_page: 0,
            options_per_page,
            num_pages,
            page_start: 0,
            page_end: 0,
            max_width,
        };
        menu.set_page(0);
        menu
    }

    pub fn show(&mut self) {
        let stdout = Term::buffered_stdout();
        stdout.hide_cursor().unwrap();

        let term_height = Term::stdout().size().0 as usize;
        stdout.write_str(&"\n".repeat(term_height - 1)).unwrap();

        self.draw(&stdout);
        self.run_navigation(&stdout);
    }

    fn run_navigation(&mut self, stdout: &Term) {
        loop {
            let key = stdout.read_key().unwrap();

            match key {
                Key::ArrowUp | Key::Char('k') => {
                    if self.selected_option != self.page_start {
                        self.selected_option -= 1;
                    } else if self.selected_page != 0 {
                        self.set_page(self.selected_page - 1);
                        self.selected_option = self.page_end;
                    }
                }
                Key::ArrowDown | Key::Char('j') => {
                    if self.selected_option < self.page_end {
                        self.selected_option += 1
                    } else if self.selected_page < self.num_pages - 1 {
                        self.set_page(self.selected_page + 1);
                    }
                }
                Key::ArrowLeft | Key::Char('h') | Key::Char('b') => {
                    if self.selected_page != 0 {
                        self.set_page(self.selected_page - 1);
                    }
                }
                Key::ArrowRight | Key::Char('l') | Key::Char('w') => {
                    if self.selected_page < self.num_pages - 1 {
                        self.set_page(self.selected_page + 1);
                    }
                }
                Key::Escape | Key::Char('q') | Key::Backspace => {
                    self.exit(stdout);
                    break;
                }
                Key::Enter => {
                    if self.exit_on_action {
                        self.exit(stdout);
                        (self.options[self.selected_option].action)();
                        break;
                    }
                    (self.options[self.selected_option].action)();
                }
                _ => {}
            }

            self.draw(stdout);
        }
    }

    fn set_page(&mut self, page: usize) {
        self.selected_page = page;
        self.page_start = self.selected_page * self.options_per_page;
        self.selected_option = self.page_start;
        if self.options.len() > self.page_start + self.options_per_page {
            self.page_end = self.page_start + self.options_per_page - 1
        } else {
            self.page_end = self.options.len() - 1
        }
    }

    fn draw(&self, stdout: &Term) {
        clear_screen(stdout);

        let menu_width = self.max_width;
        let mut extra_lines = 2;
        if let Some(_) = self.title {
           extra_lines += 2; 
        }
        if let Some(_) = self.message {
            extra_lines += 1;
        }

        let indent: usize = (stdout.size().1 / 2) as usize - ((menu_width + 4) / 2);
        let indent_str = pad_left("".to_string(), indent);

        let vertical_pad: usize = (stdout.size().0 / 2) as usize  - ((self.options_per_page + extra_lines) / 2);
        stdout.write_str(&format!("{:\n<width$}", "", width=vertical_pad)).unwrap();

        stdout.write_str(&format!("\x1b[38;5;{}m", self.fg_color)).unwrap(); // set foreground color
        stdout.write_line(&format!("{}{}", indent_str, self.apply_bg("", menu_width))).unwrap();

        let mut ansi_width = 34 + num_digs(self.fg_color) + num_digs(self.title_color);
        if let Some(title) = &self.title {
            let title_str = format!("\x1b[4m{}\x1b[24m", self.apply_bold(title)); // apply bold + underline
            stdout.write_line(&format!("{}{}", indent_str, self.apply_bg(&self.switch_fg(&title_str, self.title_color), menu_width + ansi_width))).unwrap();
            stdout.write_line(&format!("{}{}", indent_str, self.apply_bg("", menu_width))).unwrap();
        } 

        for (i, option) in self.options[self.page_start..=self.page_end].iter().enumerate() {
            let option_str = if self.page_start + i == self.selected_option {
                ansi_width = 25 + num_digs(self.fg_color) + num_digs(self.selected_color);
                format!("{}", self.switch_fg(&self.apply_bold(&option.label), self.selected_color))
            } else {
                ansi_width = 0;
                format!("{}", option.label)
            };
            stdout.write_line(&format!("{}{}", indent_str, self.apply_bg(&option_str, menu_width + ansi_width))).unwrap();
        }

        if self.num_pages > 1 {
            stdout.write_line(&format!("{}{}", indent_str, self.apply_bg(&format!("Page {} of {}", self.selected_page + 1, self.num_pages), menu_width))).unwrap();
        }
        if let Some(message) = &self.message {
            stdout.write_line(&format!("{}{}", indent_str, self.apply_bg("", menu_width))).unwrap();
            stdout.write_line(&format!("{}{}", indent_str, self.switch_fg(&self.apply_bg(message, menu_width), self.msg_color))).unwrap();
        }

        stdout.write_line(&format!("{}{}", indent_str, self.apply_bg("", menu_width))).unwrap();
        stdout.write_str("\x1b[39m").unwrap(); // reset foreground color

        stdout.flush().unwrap();
    }


    fn apply_bold(&self, s: &str) -> String { // 9 ansi chars
        format!("\x1b[1m{}\x1b[22m", s)
    }

    fn switch_fg(&self, s: &str, color: u8) -> String { // 16 + (fg digs + switch digs) ansi chars
        format!("\x1b[38;5;{}m{}\x1b[38;5;{}m", color, s, self.fg_color)
    }

    fn apply_bg(&self, s: &str, width: usize) -> String {
        format!("\x1b[48;5;{}m{}\x1b[49m", self.bg_color, pad_right(format!("  {}", s), width + 4)) 
    }


    fn exit(&self, stdout: &Term) {
        clear_screen(stdout);
        stdout.show_cursor().unwrap();
        stdout.flush().unwrap();
    }
}


fn clear_screen(stdout: &Term) {
    stdout.write_str("\x1b[H\x1b[J\x1b[H").unwrap();
}

fn pad_left(s: String, width: usize) -> String {
    format!("{: >width$}", s, width=width)
}

fn pad_right(s: String, width: usize) -> String {
    format!("{: <width$}", s, width=width)
}

fn clamp(num: usize, min: usize, max: usize) -> usize {
    let out = if num < min { min } else { num };
    if out > max { max } else { out }
}

fn num_digs(num: u8) -> usize {
    (num.checked_ilog10().unwrap_or(0) + 1) as usize
}
