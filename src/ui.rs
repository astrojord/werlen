use std::vec;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, Tabs},
};

use crate::shop::{App, Rarity};

const BORDER_COLOR: Color = Color::Rgb(90, 90, 130);
const TITLE_COLOR: Color = Color::Rgb(230, 220, 255);
const FOOTER_FG: Color = Color::Rgb(120, 120, 140);

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

/// Translate level/rarity to color
fn rarity_color(rarity: Rarity) -> Color {
    match rarity as usize {
        0 => Color::Gray,
        1 => Color::Green,
        2 => Color::Cyan,
        3 => Color::Magenta,
        4 => Color::Yellow,
        _ => Color::White,
    }
}

fn spell_color(level: usize) -> Color {
    match level {
        0 | 1 => Color::Gray,
        2 | 3 => Color::Green,
        4 | 5 => Color::Cyan,
        6 | 7 | 8 => Color::Magenta,
        9 => Color::Yellow,
        _ => Color::White,
    }
}

fn bordered_block(title: &str) -> Block<'_> {
    Block::default()
        .title(format!(" {title} "))
        .title_style(Style::new().fg(TITLE_COLOR).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(BORDER_COLOR))
}

pub fn render_header(frame: &mut Frame, area: Rect) {
    let block = bordered_block("Werlen's Ware Generator");
    let paragraph = Paragraph::new("* . ﹢ ˖ ✦ ¸ . ﹢ ° ¸. ° ˖ ･ ·̩ ｡ ☆ ﾟ ＊ ¸* . ﹢ ˖ ✦ ¸ . ﹢")
        .alignment(Alignment::Center)
        .style(Style::new().fg(TITLE_COLOR).italic())
        .block(block);
    frame.render_widget(paragraph, area);
}

pub fn render_footer(frame: &mut Frame, area: Rect) {
    let text =
        "q quit   •   g generate   •   r reload stock   •   ←/→ switch tabs   •   d dismiss error";
    let footer = Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::new().fg(FOOTER_FG));
    frame.render_widget(footer, area);
}

pub fn render_tabs(frame: &mut Frame, area: Rect, selected_tab: usize) {
    let tabs = Tabs::new(vec!["Scrolls", "Items", "Specials", "Settings"])
        .style(Style::new().fg(Color::White))
        .highlight_style(
            Style::new()
                .fg(Color::Rgb(255, 210, 120))
                .add_modifier(Modifier::BOLD),
        )
        .select(selected_tab)
        .divider(symbols::DOT)
        .padding(" ", " ")
        .block(bordered_block("Menu"));
    frame.render_widget(tabs, area);
}

pub fn render_content(app: &mut App, frame: &mut Frame, area: Rect, selected_tab: usize) {
    if selected_tab == 3 {
        render_settings(app, frame, area, selected_tab);
        return;
    }

    if app.scroll_stock.is_empty() {
        let (title, text) = match selected_tab {
            0 => (
                "Scrolls",
                "Press 'g' to generate a shop and see scroll inventory here.",
            ),
            1 => (
                "Items",
                "Press 'g' to generate a shop and see item inventory here.",
            ),
            2 => (
                "Specials",
                "Press 'g' to generate a shop and see special stock inventory here.",
            ),
            _ => unreachable!(),
        };
        let block = Paragraph::new(text)
            .alignment(Alignment::Center)
            .style(Style::new().fg(FOOTER_FG))
            .block(bordered_block(title));
        frame.render_widget(block, area);
    } else {
        render_table(app, frame, area, selected_tab);
    }
}

pub fn render_stock_error(frame: &mut Frame) {
    let popup_block = Block::default()
        .title(" Error generating shop ")
        .title_style(Style::new().fg(Color::Red).bold())
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::new().fg(Color::Red));

    let paragraph = Paragraph::new("Please ensure the shop stock pools have been\nloaded (press 'r') before generating inventory.\n\nPress 'd' to dismiss this message.")
        .style(Style::new().fg(Color::Rgb(255, 150, 150)))
        .alignment(Alignment::Center);

    let area = centered_rect(60, 25, frame.area());
    frame.render_widget(paragraph.clone().block(popup_block), area);
}

pub fn render_settings(app: &mut App, frame: &mut Frame, area: Rect, _selected_tab: usize) {
    let widths = [Constraint::Percentage(30), Constraint::Fill(1)];
    let column_names = ["Setting", "Value"];

    let data_rows: Vec<(String, String)> = vec![
        ("Max scrolls".into(), app.max_scrolls.to_string()),
        ("Max scroll level".into(), app.max_scroll_level.to_string()),
        ("Max items".into(), app.max_items.to_string()),
        ("Max item rarity".into(), app.max_item_rarity.to_string()),
        ("Max specials".into(), app.max_specials.to_string()),
        (
            "Max special rarity".into(),
            app.max_special_rarity.to_string(),
        ),
        (
            "Stock source path".into(),
            app.stock_source.display().to_string(),
        ),
    ];

    let mut rows: Vec<Row<'_>> =
        vec![Row::new(column_names).style(Style::new().fg(TITLE_COLOR).bold())];
    for (name, value) in data_rows.into_iter() {
        rows.push(Row::new(vec![Cell::from(name), Cell::from(value)]));
    }

    let table = Table::new(rows, widths).block(bordered_block("Settings"));
    frame.render_widget(table, area);
}

pub fn render_table(app: &mut App, frame: &mut Frame, area: Rect, selected_tab: usize) {
    let (stock, column_names, title) = match selected_tab {
        0 => (
            &app.scroll_stock,
            vec!["Name", "Category", "Level", "Price (gp)"],
            "Scrolls",
        ),
        1 => (
            &app.item_stock,
            vec!["Name", "Category", "Rarity", "Price (gp)"],
            "Items",
        ),
        2 => (
            &app.special_stock,
            vec!["Name", "Category", "Level/Rarity", "Price (gp)"],
            "Specials",
        ),
        _ => unreachable!(),
    };

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(10),
    ];
    let mut rows: Vec<Row<'_>> =
        vec![Row::new(column_names).style(Style::new().fg(TITLE_COLOR).bold())];

    for item in stock.iter() {
        let col3_content: String;
        let mut col3_style = Style::new();
        if let Some(rarity) = item.rarity {
            col3_style = col3_style.fg(rarity_color(rarity));
            col3_content = rarity.to_string();
        } else {
            let level = item.level.unwrap();
            col3_style = col3_style.fg(spell_color(level));
            col3_content = format!("{level}");
        }

        let row = Row::new(vec![
            Cell::from(item.name.clone()),
            Cell::from(item.category.clone()),
            Cell::from(col3_content).style(col3_style),
            Cell::from(format!("{} gp", item.price)),
        ]);

        rows.push(row);
    }

    let table = Table::new(rows, widths).block(bordered_block(title));
    frame.render_widget(table, area);
}
