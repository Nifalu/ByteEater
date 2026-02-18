use anyhow::{anyhow, Result};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use serde_json::Value;

const KEEP: &[&str] = &["Name", "Teaser", "Description"];

enum Craving {
    Day(NaiveDate),
    Week(i32, u32),
}

/// Parse a date specifier into a single day or a whole week.
fn parse(s: &str) -> Result<Craving> {
    let today = Utc::now().date_naive();
    match s {
        "today" => Ok(Craving::Day(today)),
        "tomorrow" => Ok(Craving::Day(today + Duration::days(1))),
        "yesterday" => Ok(Craving::Day(today - Duration::days(1))),
        "thisweek" => Ok(week_of(today)),
        "lastweek" => Ok(week_of(today - Duration::weeks(1))),
        "nextweek" => Ok(week_of(today + Duration::weeks(1))),
        s if s.starts_with("t+") => s[2..].parse().map(|n| Craving::Day(today + Duration::days(n)))
            .map_err(|_| anyhow!("invalid offset '{s}', expected e.g. t+3")),
        s if s.starts_with("t-") => s[2..].parse().map(|n| Craving::Day(today - Duration::days(n)))
            .map_err(|_| anyhow!("invalid offset '{s}', expected e.g. t-2")),
        s if s.starts_with("w+") => s[2..].parse().map(|n: i64| {
            week_of(today + Duration::weeks(n))
        }).map_err(|_| anyhow!("invalid offset '{s}', expected e.g. w+1")),
        s if s.starts_with("w-") => s[2..].parse().map(|n: i64| {
            week_of(today - Duration::weeks(n))
        }).map_err(|_| anyhow!("invalid offset '{s}', expected e.g. w-1")),
        s if s.starts_with('w') => s[1..].parse().map(|w| Craving::Week(today.year(), w))
            .map_err(|_| anyhow!("invalid week '{s}', expected e.g. w7")),
        s => NaiveDate::parse_from_str(s, "%d-%m-%Y")
            .or_else(|_| NaiveDate::parse_from_str(&s.replace(':', "-"), "%d-%m-%Y"))
            .map(Craving::Day)
            .map_err(|_| anyhow!("unknown '{s}', try: today|tomorrow|yesterday|t+/-n|thisweek|lastweek|nextweek|w+/-n|w7|DD-MM-YYYY")),
    }
}

/// Extract the ISO year and week number from a date.
fn week_of(d: NaiveDate) -> Craving {
    Craving::Week(d.year(), d.iso_week().week())
}

/// Strip the API response down to just Name, Teaser and Description per product.
fn slim(data: &Value) -> Value {
    let days = match data.get("Days").and_then(|d| d.as_array()) {
        Some(days) => days,
        None => return Value::Array(vec![]),
    };
    Value::Array(
        days.iter()
            .filter_map(|day| {
                let cats = day.get("Categories")?.as_array()?;
                let slim_cats: Vec<Value> = cats
                    .iter()
                    .filter_map(|cat| {
                        let name = cat.get("Name")?.clone();
                        let products = cat.get("Products")?.as_array()?;
                        let slim_prods: Vec<Value> = products
                            .iter()
                            .map(|p| {
                                let mut out = serde_json::Map::new();
                                for &k in KEEP {
                                    if let Some(v) = p.get(k) {
                                        out.insert(k.into(), v.clone());
                                    }
                                }
                                Value::Object(out)
                            })
                            .collect();
                        Some(serde_json::json!({"Name": name, "Products": slim_prods}))
                    })
                    .collect();
                Some(serde_json::json!({
                    "WeekDay": day.get("WeekDay")?,
                    "Categories": slim_cats,
                }))
            })
            .collect(),
    )
}

/// Pretty-print a JSON value to stdout.
fn print(v: &Value) {
    println!("{}", serde_json::to_string_pretty(v).unwrap());
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.get(1).map(|s| s.as_str()) != Some("indulge") || args.get(2).is_none() {
        eprintln!("Usage: byteeater indulge <today|tomorrow|yesterday|t+/-n|thisweek|lastweek|nextweek|w+/-n|w7|DD-MM-YYYY>");
        std::process::exit(1);
    }

    match parse(&args[2])? {
        Craving::Day(date) => {
            let data = byteeater::fetch_week(date.year(), date.iso_week().week())?;
            let days = slim(&data);
            let weekday = date.weekday().num_days_from_monday();
            if let Some(day) = days.as_array().and_then(|ds| {
                ds.iter()
                    .find(|d| d.get("WeekDay").and_then(|w| w.as_f64()) == Some(weekday as f64))
            }) {
                print(day);
            } else {
                println!("null");
            }
        }
        Craving::Week(year, week) => {
            print(&slim(&byteeater::fetch_week(year, week)?));
        }
    }
    Ok(())
}
