use rand::seq::IndexedRandom;
use std::path::PathBuf;
use std::{error::Error, process};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::style::Style;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Offset},
    style::Stylize,
    text::{Line, Span},
};

use crate::ui::{render_content, render_stock_error, render_tabs};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct App {
    /// Is the application running without error?
    running: bool,
    stock_error: bool,

    /// What tab are we looking at? (mod 4)
    tab: usize,

    /// Settings
    max_scrolls: usize,
    max_scroll_level: usize,
    max_items: usize,
    max_item_rarity: Rarity,
    max_specials: usize,
    max_special_rarity: Rarity,
    stock_source: PathBuf,

    // Stock pools, populated from stock source
    scroll_stock_pool: Vec<StockItem>,
    item_stock_pool: Vec<StockItem>,

    /// Current shop stock, taken from pools
    scroll_stock: Vec<StockItem>,
    item_stock: Vec<StockItem>,
    special_stock: Vec<StockItem>,
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
            stock_source: PathBuf::from(r"C:\Users\Jordan\Desktop\code\werlen\stock_source.csv"),

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
        let outer_layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
            .spacing(1)
            .split(frame.area());
        let top = outer_layout[0];
        let main = outer_layout[1];

        // let inner_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).spacing(1).split(main);
        // let art = inner_layout[0];
        // let tabs = inner_layout[1];

        let title = Line::from_iter([
            Span::from("Werlen's Ware Generator").bold(),
            Span::from(
                " (q: quit, g: generate, r: reload stock source, arrow keys: navigate wares)",
            )
            .style(Style::new().cyan()),
        ]);
        frame.render_widget(title.centered(), top);

        render_content(self, frame, main, selected_tab);
        // render_werlen(frame, art);
        render_tabs(frame, main.offset(Offset { x: 1, y: 0 }), selected_tab);

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
            // Add other key handlers here.
            (_, KeyCode::Char('r')) => self.update_stock_pools(),
            (_, KeyCode::Char('g')) => self.generate_shop(),
            (_, KeyCode::Char('d')) => self.stock_error = false,
            (_, KeyCode::Char('l') | KeyCode::Right) => self.tab = (self.tab + 1) % 4,
            (_, KeyCode::Char('h') | KeyCode::Left) => self.tab = (self.tab + 3) % 4,
            _ => {}
        }
    }

    fn update_stock_pools(&mut self) {
        if !self.item_stock_pool.is_empty() {
            self.item_stock_pool.clear();
        }

        if !self.scroll_stock_pool.is_empty() {
            self.scroll_stock_pool.clear();
        }

        if let Err(err) = self.read_csv() {
            println!("{}", err);
            process::exit(1);
        }

        // println!("{:?}", self.scroll_stock_pool);
        // println!("{:?}", self.item_stock_pool);
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
            if chosen_item.rarity < Some(self.max_item_rarity) {
                self.item_stock.push(chosen_item.clone());
            }
        }

        while self.scroll_stock.len() < self.max_scrolls {
            // 9 1st, 5 2nd, 3 3rd, 2 4th, 1 5th for a max of 20 with max level 5?
            // would be better to just weight the individual levels
            let chosen_scroll = self.scroll_stock_pool.choose(&mut rng).unwrap();
            if chosen_scroll.level < Some(self.max_scroll_level) {
                self.scroll_stock.push(chosen_scroll.clone());
            }
        }

        //while self.special_stock.len() < self.max_specials {
        //    todo!()
        //};
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
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

#[derive(Debug, Clone)]
pub struct StockItem {
    level: Option<usize>,
    rarity: Option<Rarity>,
    name: String,
    category: String,
    price: usize,
}
