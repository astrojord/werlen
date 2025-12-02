use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Offset},
    style::Stylize,
    text::{Line, Span},
};

use crate::ui::{render_content, render_tabs};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct App {
    /// Is the application running?
    running: bool,

    /// What tab are we looking at?
    tab: usize,

    /// State variables
    max_scrolls: usize,
    max_scroll_level: usize,
    max_items: usize,
    max_item_rarity: Rarity,
    max_specials: usize,
    max_special_rarity: Rarity,

    // Stock
    scroll_stock: Vec<StockItem>,
    item_stock: Vec<StockItem>,
    special_stock: Vec<StockItem>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self {
            running: false,
            tab: 0,
            max_scrolls: 20,
            max_scroll_level: 5,
            max_items: 10,
            max_item_rarity: Rarity::Uncommon,
            max_specials: 5,
            max_special_rarity: Rarity::VeryRare,

            ..Default::default()
        }
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.running = true;

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
        let outer_layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1).split(frame.area());
        let top = outer_layout[0];
        let main = outer_layout[1];

        // let inner_layout = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).spacing(1).split(main);
        // let art = inner_layout[0];
        // let tabs = inner_layout[1];

        let title = Line::from_iter([
            Span::from("Werlen's Ware Generator").bold(),
            Span::from(" (press 'q' to quit, 'g' to generate, arrow keys to navigate wares)"),
        ]);
        frame.render_widget(title.centered(), top);

        render_content(self, frame, main, selected_tab);
        // render_werlen(frame, art);
        render_tabs(frame, main.offset(Offset { x:1, y:0 }), selected_tab);
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
            (_, KeyCode::Char('g')) => self.generate_shop(),
            (_, KeyCode::Char('l') | KeyCode::Right) => self.tab = (self.tab + 1) % 3,
            (_, KeyCode::Char('h') | KeyCode::Left) => self.tab = (self.tab + 2) % 3,
            _ => {}
        }
    }

    fn generate_shop(&mut self) {
        todo!()
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}


#[derive(Debug, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
    Artifact
}

#[derive(Debug)]
pub struct StockItem {
    level: Option<usize>,
    rarity: Option<Rarity>,
    name: &'static str,
    price: usize,
}