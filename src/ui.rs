use ratatui::{
    Frame, symbols,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph, Table, Tabs},
};

use crate::shop::App;

pub fn render_tabs(frame: &mut Frame, area: Rect, selected_tab: usize) {
    let tabs = Tabs::new(vec!["Scrolls", "Items", "Specials"])
        .style(Color::White)
        .highlight_style(Style::default().magenta().on_black().bold())
        .select(selected_tab)
        .divider(symbols::DOT)
        .padding(" ", " ");
    frame.render_widget(tabs, area);
}

pub fn render_content(app: &mut App, frame: &mut Frame, area: Rect, selected_tab: usize) {
    let text = match selected_tab {
        0 => "Press 'g' to generate a shop and see scroll inventory here.",
        1 => "Press 'g' to generate a shop and see item inventory here.",
        2 => "Press 'g' to generate a shop and see special stock inventory here.",
        _ => unreachable!(),
    };
    let block = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::bordered());
    frame.render_widget(block, area);
}