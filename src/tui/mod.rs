pub mod favorites;
mod ui;

use crate::model::{Day, Product};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use favorites::Favorite;
use ratatui::prelude::*;
use std::io::stdout;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Menu,
    Detail,
    Favorites,
}

pub struct App {
    pub days: Vec<Day>,
    pub day_idx: usize,
    pub scroll: usize,
    pub view: View,
    pub favorites: Vec<Favorite>,
    pub fav_scroll: usize,
    pub detail_product: Option<(String, Product)>,
    pub week_label: String,
}

impl App {
    pub fn new(days: Vec<Day>, week_label: String) -> Self {
        Self {
            days,
            day_idx: 0,
            scroll: 0,
            view: View::Menu,
            favorites: favorites::load(),
            fav_scroll: 0,
            detail_product: None,
            week_label,
        }
    }

    /// Flat list of (category_name, product) for the current day.
    fn current_items(&self) -> Vec<(String, &Product)> {
        match self.days.get(self.day_idx) {
            Some(day) => day
                .categories
                .iter()
                .flat_map(|cat| cat.products.iter().map(move |p| (cat.name.clone(), p)))
                .collect(),
            None => vec![],
        }
    }

    fn item_count(&self) -> usize {
        self.current_items().len()
    }
}

pub fn run(days: Vec<Day>, week_label: String) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new(days, week_label);
    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    favorites::save(&app.favorites)?;
    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match app.view {
                View::Menu => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Left | KeyCode::Char('h') => {
                        if app.day_idx > 0 {
                            app.day_idx -= 1;
                            app.scroll = 0;
                        }
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        if app.day_idx + 1 < app.days.len() {
                            app.day_idx += 1;
                            app.scroll = 0;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.scroll = app.scroll.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let max = app.item_count().saturating_sub(1);
                        if app.scroll < max {
                            app.scroll += 1;
                        }
                    }
                    KeyCode::Enter => {
                        let items = app.current_items();
                        if let Some((cat, product)) = items.get(app.scroll) {
                            app.detail_product = Some((cat.clone(), (*product).clone()));
                            app.view = View::Detail;
                        }
                    }
                    KeyCode::Char('s') => {
                        let info: Option<(String, String)> = app
                            .current_items()
                            .get(app.scroll)
                            .map(|(cat, p)| (p.name.clone(), cat.clone()));
                        if let Some((name, cat)) = info {
                            favorites::toggle(&mut app.favorites, &name, &cat);
                        }
                    }
                    KeyCode::Char('f') => {
                        app.view = View::Favorites;
                        app.fav_scroll = 0;
                    }
                    _ => {}
                },
                View::Detail => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                        app.view = View::Menu;
                    }
                    KeyCode::Char('s') => {
                        if let Some((ref cat, ref product)) = app.detail_product {
                            favorites::toggle(&mut app.favorites, &product.name, cat);
                        }
                    }
                    _ => {}
                },
                View::Favorites => match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace | KeyCode::Char('f') => {
                        app.view = View::Menu;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.fav_scroll = app.fav_scroll.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        let max = app.favorites.len().saturating_sub(1);
                        if app.fav_scroll < max {
                            app.fav_scroll += 1;
                        }
                    }
                    KeyCode::Char('d') | KeyCode::Delete => {
                        if app.fav_scroll < app.favorites.len() {
                            app.favorites.remove(app.fav_scroll);
                            if app.fav_scroll > 0 && app.fav_scroll >= app.favorites.len() {
                                app.fav_scroll -= 1;
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}
