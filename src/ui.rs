use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    widgets::{Block, BorderType, Borders, Paragraph, Table, Tabs},
};

use crate::shop::App;

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn render_tabs(frame: &mut Frame, area: Rect, selected_tab: usize) {
    let tabs = Tabs::new(vec!["Scrolls", "Items", "Specials", "Settings"])
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
        3 => "insert settings here",
        _ => unreachable!(),
    };
    let block = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::bordered());
    frame.render_widget(block, area);
}

pub fn render_stock_error(frame: &mut Frame) {
    let popup_block = Block::default()
        .title("Error generating shop")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick);

    let paragraph = Paragraph::new("Please ensure the shop stock pools have been\nloaded (press 'r') before generating inventory.\n\nPress 'd' to dismiss this message.")
        .red()
        .alignment(Alignment::Center);

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(paragraph.clone().block(popup_block), area);
}
