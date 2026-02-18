use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub name: String,
    pub teaser: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub products: Vec<Product>,
}

#[derive(Debug, Clone)]
pub struct Day {
    pub weekday: u32,
    pub categories: Vec<Category>,
}

impl Day {
    pub fn weekday_name(&self) -> &'static str {
        match self.weekday {
            0 => "Monday",
            1 => "Tuesday",
            2 => "Wednesday",
            3 => "Thursday",
            4 => "Friday",
            5 => "Saturday",
            6 => "Sunday",
            _ => "Unknown",
        }
    }
}

/// Parse the raw API JSON (after unfire + slim) into typed structs.
pub fn parse_days(data: &Value) -> Vec<Day> {
    let days = match data.get("Days").and_then(|d| d.as_array()) {
        Some(d) => d,
        None => return vec![],
    };
    let mut out: Vec<Day> = days
        .iter()
        .filter_map(|day| {
            let weekday = day.get("WeekDay")?.as_f64()? as u32;
            let cats = day.get("Categories")?.as_array()?;
            let categories = cats
                .iter()
                .filter_map(|cat| {
                    let name = cat.get("Name")?.as_str()?.to_string();
                    let products = cat
                        .get("Products")?
                        .as_array()?
                        .iter()
                        .filter_map(|p| {
                            Some(Product {
                                name: p.get("Name")?.as_str()?.to_string(),
                                teaser: p
                                    .get("Teaser")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                description: p
                                    .get("Description")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                            })
                        })
                        .collect();
                    Some(Category { name, products })
                })
                .collect();
            Some(Day {
                weekday,
                categories,
            })
        })
        .collect();
    out.sort_by_key(|d| d.weekday);
    out
}
