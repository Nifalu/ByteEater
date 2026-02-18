use crate::tui::{favorites, App, View};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};

const STAR: &str = " *";

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // header / tabs
        Constraint::Min(0),   // body
        Constraint::Length(1), // help bar
    ])
    .split(f.area());

    draw_header(f, app, chunks[0]);

    match app.view {
        View::Menu => draw_menu(f, app, chunks[1]),
        View::Detail => draw_detail(f, app, chunks[1]),
        View::Favorites => draw_favorites(f, app, chunks[1]),
    }

    draw_help(f, app.view, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let day_names: Vec<String> = app
        .days
        .iter()
        .map(|d| d.weekday_name().to_string())
        .collect();

    match app.view {
        View::Menu => {
            let tabs = Tabs::new(day_names)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" ByteEater - {} ", app.week_label)),
                )
                .select(app.day_idx)
                .highlight_style(Style::default().fg(Color::Yellow).bold());
            f.render_widget(tabs, area);
        }
        View::Detail => {
            let title = match &app.detail_product {
                Some((cat, _)) => format!(" {} ", cat),
                None => " Detail ".to_string(),
            };
            let block = Block::default().borders(Borders::ALL).title(title);
            f.render_widget(block, area);
        }
        View::Favorites => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Favorites ");
            f.render_widget(block, area);
        }
    }
}

fn draw_menu(f: &mut Frame, app: &App, area: Rect) {
    let day = match app.days.get(app.day_idx) {
        Some(d) => d,
        None => {
            let msg = Paragraph::new("No menu available for this day.")
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(msg, area);
            return;
        }
    };

    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_idx = 0usize;

    for cat in &day.categories {
        for product in &cat.products {
            let starred = if favorites::is_favorite(&app.favorites, &product.name, &cat.name) {
                STAR
            } else {
                ""
            };

            let line = if product.teaser.is_empty() {
                format!("  {}{}", product.name, starred)
            } else {
                format!("  {} - {}{}", product.name, product.teaser, starred)
            };

            let style = if flat_idx == app.scroll {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };

            items.push(ListItem::new(line).style(style));
            flat_idx += 1;
        }
    }

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    // Manually manage scroll offset for visible window
    let visible_height = area.height.saturating_sub(2) as usize; // borders
    let offset = if app.scroll >= visible_height {
        app.scroll - visible_height + 1
    } else {
        0
    };

    let mut state = ListState::default().with_offset(offset);
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let (cat, product) = match &app.detail_product {
        Some(p) => p,
        None => return,
    };

    let starred = if favorites::is_favorite(&app.favorites, &product.name, cat) {
        " *"
    } else {
        ""
    };

    let text = format!(
        "{}{}\n\n{}\n\n{}",
        product.name, starred, product.teaser, product.description
    );

    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn draw_favorites(f: &mut Frame, app: &App, area: Rect) {
    if app.favorites.is_empty() {
        let msg = Paragraph::new("No favorites yet. Press [s] on a menu item to add one.")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .favorites
        .iter()
        .enumerate()
        .map(|(i, fav)| {
            let line = format!("  {} ({})", fav.product_name, fav.category);
            let style = if i == app.fav_scroll {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    let visible_height = area.height.saturating_sub(2) as usize;
    let offset = if app.fav_scroll >= visible_height {
        app.fav_scroll - visible_height + 1
    } else {
        0
    };

    let mut state = ListState::default().with_offset(offset);
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_help(f: &mut Frame, view: View, area: Rect) {
    let help = match view {
        View::Menu => " [</>] day  [j/k] scroll  [Enter] detail  [s] star  [f] favorites  [q] quit",
        View::Detail => " [s] star  [Esc] back",
        View::Favorites => " [j/k] scroll  [d] remove  [Esc] back",
    };
    let bar = Paragraph::new(help).style(Style::default().fg(Color::DarkGray));
    f.render_widget(bar, area);
}
