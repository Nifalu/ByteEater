use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Favorite {
    pub product_name: String,
    pub category: String,
}

fn favorites_path() -> Result<PathBuf> {
    let dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("no home directory"))?
        .join(".byteeater");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("favorites.json"))
}

pub fn load() -> Vec<Favorite> {
    let path = match favorites_path() {
        Ok(p) => p,
        Err(_) => return vec![],
    };
    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return vec![],
    };
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save(favs: &[Favorite]) -> Result<()> {
    let path = favorites_path()?;
    let json = serde_json::to_string_pretty(favs)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn is_favorite(favs: &[Favorite], product_name: &str, category: &str) -> bool {
    favs.iter()
        .any(|f| f.product_name == product_name && f.category == category)
}

pub fn toggle(favs: &mut Vec<Favorite>, product_name: &str, category: &str) {
    if let Some(idx) = favs
        .iter()
        .position(|f| f.product_name == product_name && f.category == category)
    {
        favs.remove(idx);
    } else {
        favs.push(Favorite {
            product_name: product_name.to_string(),
            category: category.to_string(),
        });
    }
}
