use rand::seq::IndexedRandom;
use std::path::PathBuf;
use std::{error::Error, fmt};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};

use crate::ui::{render_content, render_footer, render_header, render_stock_error, render_tabs};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct App {
    /// Is the application running without error?
    running: bool,
    stock_error: bool,

    /// Whether the stock source CSV has been loaded, and what happened last time we tried.
    pub stock_status: StockStatus,

    /// Index of the currently highlighted row on the Settings tab (0..=5).
    pub selected_setting: usize,

    /// What tab are we looking at? (mod 4)
    pub tab: usize,

    /// Settings
    pub max_scrolls: usize,
    pub max_scroll_level: usize,
    pub max_items: usize,
    pub max_item_rarity: Rarity,
    pub max_specials: usize,
    pub max_special_rarity: Rarity,
    pub stock_source: PathBuf,

    // Stock pools, populated from stock source
    scroll_stock_pool: Vec<StockItem>,
    item_stock_pool: Vec<StockItem>,

    /// Current shop stock, taken from pools
    pub scroll_stock: Vec<StockItem>,
    pub item_stock: Vec<StockItem>,
    pub special_stock: Vec<StockItem>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: false,
            stock_error: false,
            tab: 0,
            max_scrolls: 20,
            max_scroll_level: 5,
            max_items: 10,
            max_item_rarity: Rarity::Uncommon,
            max_specials: 5,
            max_special_rarity: Rarity::VeryRare,
            stock_source: PathBuf::from(r"/home/jordan/code/werlen/stock_source.csv"),

            ..Default::default()
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;
        self.stock_error = false;

        while self.running {
            terminal.draw(|frame| self.render(frame, self.tab))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame, selected_tab: usize) {
        let outer_layout = Layout::vertical([
            Constraint::Length(3), // header (title + status line)
            Constraint::Length(3), // tab bar
            Constraint::Fill(1),   // content
            Constraint::Length(1), // footer / keybinds
        ])
        .split(frame.area());
        let header = outer_layout[0];
        let tabs = outer_layout[1];
        let main = outer_layout[2];
        let footer = outer_layout[3];

        render_header(self, frame, header);
        render_tabs(frame, tabs, selected_tab);
        render_content(self, frame, main, selected_tab);
        render_footer(frame, footer, selected_tab);

        if self.stock_error {
            render_stock_error(frame);
        }
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> color_eyre::Result<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            (_, KeyCode::Char('r')) => self.update_stock_pools(),
            (_, KeyCode::Char('g')) => self.generate_shop(),
            (_, KeyCode::Char('d')) => self.stock_error = false,
            (_, KeyCode::Char('l')) => self.tab = (self.tab + 1) % 4,
            (_, KeyCode::Char('h')) => self.tab = (self.tab + 3) % 4,
            (_, KeyCode::Right) => {
                if self.tab == 3 {
                    self.adjust_selected_setting(1);
                }
            }
            (_, KeyCode::Left) => {
                if self.tab == 3 {
                    self.adjust_selected_setting(-1);
                }
            }
            (_, KeyCode::Up) if self.tab == 3 => self.move_setting_selection(-1),
            (_, KeyCode::Down) if self.tab == 3 => self.move_setting_selection(1),
            _ => {}
        }
    }

    fn update_stock_pools(&mut self) {
        self.item_stock_pool.clear();
        self.scroll_stock_pool.clear();

        match self.read_csv() {
            Ok(()) => {
                self.stock_status = StockStatus::Loaded {
                    scrolls: self.scroll_stock_pool.len(),
                    items: self.item_stock_pool.len(),
                };
            }
            Err(err) => {
                self.stock_status = StockStatus::Error(err.to_string());
            }
        }
    }

    fn read_csv(&mut self) -> Result<(), Box<dyn Error>> {
        // read csv and populate the stock pools
        let mut reader = csv::Reader::from_path(self.stock_source.as_path())?;
        for result in reader.records() {
            let record = result?;

            let name = record.get(0).unwrap_or_default().trim().to_string();
            let price: usize =
                str::parse(record.get(1).unwrap_or_default().trim()).unwrap_or_default();
            let category = record.get(2).unwrap_or_default().trim().to_string();
            let rarity_str = record.get(3).unwrap_or_default().trim().to_lowercase();
            let level_str = record.get(4).unwrap_or_default().trim();

            let rarity: Option<Rarity>;
            if rarity_str.is_empty() {
                rarity = None;
            } else {
                rarity = match rarity_str.as_str() {
                    "common" => Some(Rarity::Common),
                    "uncommon" => Some(Rarity::Uncommon),
                    "rare" => Some(Rarity::Rare),
                    "very rare" => Some(Rarity::VeryRare),
                    "legendary" => Some(Rarity::Legendary),
                    _ => Some(Rarity::Common),
                };
            }

            let level: Option<usize>;
            if level_str.is_empty() {
                level = None;
            } else {
                let level_usize: usize = str::parse(level_str).unwrap_or_default();
                level = Some(level_usize);
            };

            let new_stock_item = StockItem {
                name: name,
                price: price,
                category: category,
                rarity: rarity,
                level: level,
            };

            if let Some(_) = level {
                self.scroll_stock_pool.push(new_stock_item);
            } else {
                self.item_stock_pool.push(new_stock_item);
            }

            drop(rarity_str);
        }
        Ok(())
    }

    fn generate_shop(&mut self) {
        // check that we actually have shop stock pool to look at
        if self.item_stock_pool.is_empty() || self.scroll_stock_pool.is_empty() {
            self.stock_error = true;
            return;
        }

        // new RNG whenever we re-generate
        let mut rng = rand::rng();
        // clear current stocks
        self.item_stock.clear();
        self.scroll_stock.clear();
        self.special_stock.clear();

        // select new stocks
        while self.item_stock.len() < self.max_items {
            let chosen_item = self.item_stock_pool.choose(&mut rng).unwrap();
            if chosen_item.rarity <= Some(self.max_item_rarity) {
                self.item_stock.push(chosen_item.clone());
            }
        }

        while self.scroll_stock.len() < self.max_scrolls {
            // 9 1st, 5 2nd, 3 3rd, 2 4th, 1 5th for a max of 20 with max level 5?
            // would be better to just weight the individual levels
            let chosen_scroll = self.scroll_stock_pool.choose(&mut rng).unwrap();
            if chosen_scroll.level <= Some(self.max_scroll_level) {
                self.scroll_stock.push(chosen_scroll.clone());
            }
        }

        // specials draw from both pools - item-type entries are capped by
        // max_special_rarity, scroll-type entries by max_scroll_level
        let combined_pool: Vec<&StockItem> = self
            .item_stock_pool
            .iter()
            .chain(self.scroll_stock_pool.iter())
            .collect();

        while self.special_stock.len() < self.max_specials {
            let chosen_special = combined_pool.choose(&mut rng).unwrap();
            let passes = match (chosen_special.rarity, chosen_special.level) {
                (Some(rarity), _) => rarity <= self.max_special_rarity,
                (_, Some(level)) => level <= self.max_scroll_level,
                (None, None) => false,
            };
            if passes {
                self.special_stock.push((*chosen_special).clone());
            }
        }

        // show scrolls sorted by level, then items sorted by rarity
        self.special_stock
            .sort_by_key(|item| match (item.level, item.rarity) {
                (Some(level), _) => (0u8, level, Rarity::default()),
                (_, Some(rarity)) => (1u8, 0usize, rarity),
                (None, None) => (2u8, 0usize, Rarity::default()),
            });

        self.scroll_stock.sort_by_key(|item| item.level.unwrap());
        self.item_stock.sort_by_key(|item| item.rarity.unwrap());
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn move_setting_selection(&mut self, delta: i32) {
        const NUM_SETTINGS: i32 = 6;
        let next = (self.selected_setting as i32 + delta).rem_euclid(NUM_SETTINGS);
        self.selected_setting = next as usize;
    }

    fn adjust_selected_setting(&mut self, delta: i32) {
        const POOL_MAX: usize = 128;
        match self.selected_setting {
            0 => self.max_scrolls = bump_usize(self.max_scrolls, delta, POOL_MAX),
            1 => self.max_scroll_level = bump_usize(self.max_scroll_level, delta, 9usize),
            2 => self.max_items = bump_usize(self.max_items, delta, POOL_MAX),
            3 => self.max_item_rarity = step_rarity(self.max_item_rarity, delta),
            4 => self.max_specials = bump_usize(self.max_specials, delta, POOL_MAX),
            5 => self.max_special_rarity = step_rarity(self.max_special_rarity, delta),
            _ => unreachable!(),
        }
    }
}

/// change a usize by a signed delta, clamped to [0, max]
/// used in settings tab
fn bump_usize(current: usize, delta: i32, max: usize) -> usize {
    let bumped = if delta < 0 {
        current.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        current.saturating_add(delta as usize)
    };
    bumped.min(max)
}

/// allow stepping through rarity levels in settings tab, clamped at both ends
fn step_rarity(current: Rarity, delta: i32) -> Rarity {
    const RARITIES: [Rarity; 5] = [
        Rarity::Common,
        Rarity::Uncommon,
        Rarity::Rare,
        Rarity::VeryRare,
        Rarity::Legendary,
    ];
    let idx = (current as i32 + delta).clamp(0, RARITIES.len() as i32 - 1);
    RARITIES[idx as usize]
}

#[derive(Debug, Default, Clone)]
pub enum StockStatus {
    #[default]
    NotLoaded,
    Loaded {
        scrolls: usize,
        items: usize,
    },
    Error(String),
}

#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
}

impl fmt::Display for Rarity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone)]
pub struct StockItem {
    pub level: Option<usize>,
    pub rarity: Option<Rarity>,
    pub name: String,
    pub category: String,
    pub price: usize,
}
