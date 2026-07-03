use rand::Rng;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::seq::IndexedRandom;
use std::path::PathBuf;
use std::{error::Error, fmt};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};

use crate::ui::{render_content, render_footer, render_header, render_stock_error, render_tabs};

/// max spell level within 5e
const MAX_SCROLL_LEVEL: usize = 9;

#[derive(Debug, Default)]
pub struct App {
    /// Is the application running without error?
    running: bool,
    stock_error: bool,

    /// Whether the stock source CSV has been loaded, and what happened last time we tried.
    pub stock_status: StockStatus,

    /// Index of the currently highlighted row on the settings tab (0..=7).
    pub selected_setting: usize,

    /// What tab are we looking at? (mod 4)
    pub tab: usize,

    /// Settings
    pub max_scrolls: usize,
    pub max_scroll_level: usize,
    pub min_special_scroll_level: usize,
    pub max_items: usize,
    pub max_item_rarity: Rarity,
    pub max_specials: usize,
    pub max_special_rarity: Rarity,
    pub min_special_rarity: Rarity,
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
            max_scroll_level: 3,
            min_special_scroll_level: 4,
            max_items: 10,
            max_item_rarity: Rarity::Uncommon,
            max_specials: 5,
            max_special_rarity: Rarity::VeryRare,
            min_special_rarity: Rarity::Rare,
            stock_source: PathBuf::from(r"/home/jordan/code/werlen/stock_source.csv"), // currently readonly - todo

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

        // bucket the pools by tier so weighted picks are a tier draw
        // followed by a uniform draw within that tier
        let item_buckets = bucket_by_rarity(&self.item_stock_pool);
        let level_buckets = bucket_by_level(&self.scroll_stock_pool);

        let item_choices: Vec<(u32, &Vec<&StockItem>)> = (0..=self.max_item_rarity as usize)
            .map(|r| (rarity_weight(r), &item_buckets[r]))
            .collect();

        while self.item_stock.len() < self.max_items {
            match weighted_pick(&item_choices, &mut rng) {
                Some(chosen) => self.item_stock.push(chosen.clone()),
                None => break, // nothing in the pool qualifies; stop rather than spin forever
            }
        }

        let scroll_choices: Vec<(u32, &Vec<&StockItem>)> = (0..=self.max_scroll_level)
            .map(|l| (scroll_level_weight(l), &level_buckets[l]))
            .collect();

        while self.scroll_stock.len() < self.max_scrolls {
            match weighted_pick(&scroll_choices, &mut rng) {
                Some(chosen) => self.scroll_stock.push(chosen.clone()),
                None => break,
            }
        }

        // specials draw from item and scroll tiers combined into a single bucket
        let special_item_choices = (self.min_special_rarity as usize
            ..=self.max_special_rarity as usize)
            .map(|r| (rarity_weight(r), &item_buckets[r]));
        let special_scroll_choices = (self.min_special_scroll_level..=MAX_SCROLL_LEVEL)
            .map(|l| (scroll_level_weight(l), &level_buckets[l]));
        let special_choices: Vec<(u32, &Vec<&StockItem>)> =
            special_item_choices.chain(special_scroll_choices).collect();

        while self.special_stock.len() < self.max_specials {
            match weighted_pick(&special_choices, &mut rng) {
                Some(chosen) => self.special_stock.push(chosen.clone()),
                None => break,
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
        const NUM_SETTINGS: i32 = 8;
        let next = (self.selected_setting as i32 + delta).rem_euclid(NUM_SETTINGS);
        self.selected_setting = next as usize;
    }

    fn adjust_selected_setting(&mut self, delta: i32) {
        const POOL_MAX: usize = 128;
        match self.selected_setting {
            0 => self.max_scrolls = bump_usize(self.max_scrolls, delta, POOL_MAX),
            1 => self.max_scroll_level = bump_usize(self.max_scroll_level, delta, MAX_SCROLL_LEVEL),
            2 => {
                self.min_special_scroll_level =
                    bump_usize(self.min_special_scroll_level, delta, MAX_SCROLL_LEVEL)
            }
            3 => self.max_items = bump_usize(self.max_items, delta, POOL_MAX),
            4 => self.max_item_rarity = step_rarity(self.max_item_rarity, delta),
            5 => self.max_specials = bump_usize(self.max_specials, delta, POOL_MAX),
            6 => {
                let stepped = step_rarity(self.min_special_rarity, delta);
                self.min_special_rarity = stepped.min(self.max_special_rarity);
            }
            7 => {
                let stepped = step_rarity(self.max_special_rarity, delta);
                self.max_special_rarity = stepped.max(self.min_special_rarity);
            }
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

/// sort items into buckets by rarity (index 0 = Common .. 4 = Legendary)
fn bucket_by_rarity(pool: &[StockItem]) -> [Vec<&StockItem>; 5] {
    let mut buckets: [Vec<&StockItem>; 5] = std::array::from_fn(|_| Vec::new());
    for item in pool {
        if let Some(rarity) = item.rarity {
            buckets[rarity as usize].push(item);
        }
    }
    buckets
}

/// sort scrolls into buckets by level (index 0..=MAX_SCROLL_LEVEL).
fn bucket_by_level(pool: &[StockItem]) -> Vec<Vec<&StockItem>> {
    let mut buckets: Vec<Vec<&StockItem>> = (0..=MAX_SCROLL_LEVEL).map(|_| Vec::new()).collect();
    for item in pool {
        if let Some(level) = item.level {
            if let Some(bucket) = buckets.get_mut(level) {
                bucket.push(item);
            }
        }
    }
    buckets
}

/// assign weights to rarity levels out of 100
fn rarity_weight(rarity_index: usize) -> u32 {
    const WEIGHTS: [u32; 5] = [30, 25, 20, 15, 10];
    WEIGHTS.get(rarity_index).copied().unwrap_or(1)
}

/// assign weights to scroll levels (10 - level)
fn scroll_level_weight(level: usize) -> u32 {
    ((MAX_SCROLL_LEVEL + 1).saturating_sub(level) as u32).max(1)
}

/// pick an item from weighted tiers by choosing a tier first
/// then pick uniformly inside the tier
/// return none if every bucket is empty
fn weighted_pick<'a>(
    choices: &[(u32, &Vec<&'a StockItem>)],
    rng: &mut impl Rng,
) -> Option<&'a StockItem> {
    let nonempty: Vec<&(u32, &Vec<&StockItem>)> = choices
        .iter()
        .filter(|(_, bucket)| !bucket.is_empty())
        .collect();
    if nonempty.is_empty() {
        return None;
    }

    let weights: Vec<u32> = nonempty.iter().map(|(weight, _)| *weight).collect();
    let dist = WeightedIndex::new(&weights).ok()?;
    let (_, bucket) = nonempty[dist.sample(rng)];
    bucket.choose(rng).copied()
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
