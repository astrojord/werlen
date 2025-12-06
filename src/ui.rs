use std::vec;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Tabs},
};

use crate::shop::{App, Rarity};

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
    if selected_tab == 3 {
        render_settings(app, frame, area, selected_tab);
        return;
    }

    if app.scroll_stock.is_empty() {
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
    } else {
        render_table(app, frame, area, selected_tab);
    }
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

pub fn render_settings(app: &mut App, frame: &mut Frame, area: Rect, _selected_tab: usize) {
    let widths = [Constraint::Percentage(25), Constraint::Fill(1)];
    let column_names = ["Setting", "Value"];

    let rows = [
        Row::new(column_names).bold(),
        Row::new(vec![
            String::from("Max scrolls"),
            app.max_scrolls.to_string(),
        ]),
        Row::new(vec![
            String::from("Max scroll level"),
            app.max_scroll_level.to_string(),
        ]),
        Row::new(vec![String::from("Max items"), app.max_items.to_string()]),
        Row::new(vec![
            String::from("Max item rarity"),
            app.max_item_rarity.to_string(),
        ]),
        Row::new(vec![
            String::from("Max specials"),
            app.max_specials.to_string(),
        ]),
        Row::new(vec![
            String::from("Max special rarity"),
            app.max_special_rarity.to_string(),
        ]),
        Row::new(vec![
            String::from("Stock source path"),
            app.stock_source.display().to_string(),
        ]),
    ];

    let table = Table::new(rows, widths).block(Block::bordered());

    frame.render_widget(table, area);
}

pub fn render_table(app: &mut App, frame: &mut Frame, area: Rect, selected_tab: usize) {
    let (stock, column_names) = match selected_tab {
        0 => (
            &app.scroll_stock,
            vec!["Name", "Category", "Level", "Price (gp)"],
        ),
        1 => (
            &app.item_stock,
            vec!["Name", "Category", "Rarity", "Price (gp)"],
        ),
        2 => (
            &app.special_stock,
            vec!["Name", "Category", "Level/Rarity", "Price (gp)"],
        ),
        _ => unreachable!(),
    };

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
    ];
    let mut rows: Vec<Row<'_>> = vec![Row::new(column_names).bold()];

    for item in stock {
        let col3_content: String;
        let mut col3_style = Style::new();
        if let Some(x) = item.rarity {
            col3_style = match x {
                Rarity::Common => col3_style,
                Rarity::Uncommon => col3_style.green(),
                Rarity::Rare => col3_style.cyan(),
                Rarity::VeryRare => col3_style.magenta(),
                Rarity::Legendary => col3_style.yellow(),
            };
            col3_content = x.to_string();
        } else {
            col3_content = item.level.unwrap().to_string();
        }

        let row = Row::new(vec![
            Cell::from(item.name.clone()),
            Cell::from(item.category.clone()),
            Cell::from(col3_content).style(col3_style),
            Cell::from(item.price.to_string()),
        ]);
        rows.push(row);
    }

    let table = Table::new(rows, widths).block(Block::bordered());

    frame.render_widget(table, area);
}
