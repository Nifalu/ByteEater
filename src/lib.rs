use anyhow::{anyhow, Result};
use serde_json::{json, Map, Value};
use std::process::Command;

const IDENTITY_URL: &str = "https://apps-live-eu.qnips.com/cons/api/NewIdentity";
const AUTH_URL: &str = "https://identitytoolkit.googleapis.com/v1/accounts:signInWithCustomToken?key=AIzaSyDR_h6fX9hBl2I7oy7xjzmYgDxJY2igdls";
const FIRESTORE_URL: &str = "https://firestore.googleapis.com/v1/projects/qnips-sv-group-ch/databases/(default)/documents/Release/en-US/Menus/18306/Years/{}/Weeks/{}";

pub fn fetch_week(year: i32, week: u32) -> Result<Value> {
    let token = authenticate()?;
    let url = FIRESTORE_URL
        .replacen("{}", &year.to_string(), 1)
        .replacen("{}", &week.to_string(), 1);
    let raw = http_get(&url, &[("Authorization", &format!("Bearer {token}"))])?;
    Ok(unfire(&raw))
}

fn authenticate() -> Result<String> {
    let id = http_get(IDENTITY_URL, &[("App-Brand", "svgroupch")])?;
    let custom_token = id["Content"]["FirebaseCustomToken"]
        .as_str()
        .ok_or(anyhow!("missing custom token"))?;

    let auth = http_post(AUTH_URL, &json!({"token": custom_token, "returnSecureToken": true}))?;
    auth["idToken"]
        .as_str()
        .map(String::from)
        .ok_or(anyhow!("missing id token"))
}

fn http_get(url: &str, headers: &[(&str, &str)]) -> Result<Value> {
    let mut cmd = Command::new("curl");
    cmd.args(["-s", url]);
    for (k, v) in headers { cmd.args(["-H", &format!("{k}: {v}")]); }
    Ok(serde_json::from_slice(&cmd.output()?.stdout)?)
}

fn http_post(url: &str, body: &Value) -> Result<Value> {
    Ok(serde_json::from_slice(
        &Command::new("curl")
            .args(["-s", "-X", "POST", "-H", "Content-Type: application/json", "-d", &body.to_string(), url])
            .output()?.stdout
    )?)
}

fn unfire(v: &Value) -> Value {
    let m = match v.as_object() {
        Some(m) => m,
        None => return v.clone(),
    };
    for k in ["stringValue", "doubleValue", "booleanValue"] {
        if let Some(val) = m.get(k) {
            return val.clone();
        }
    }
    if let Some(n) = m.get("integerValue").and_then(|v| v.as_str()) {
        return Value::from(n.parse::<f64>().unwrap_or(0.0));
    }
    if let Some(av) = m.get("arrayValue").and_then(|v| v.as_object()) {
        return match av.get("values").and_then(|v| v.as_array()) {
            Some(vals) => Value::Array(vals.iter().map(unfire).collect()),
            None => Value::Array(vec![]),
        };
    }
    if let Some(mv) = m.get("mapValue").and_then(|v| v.as_object()) {
        return match mv.get("fields").and_then(|v| v.as_object()) {
            Some(fields) => Value::Object(fields.iter().map(|(k, v)| (k.clone(), unfire(v))).collect()),
            None => Value::Object(Map::new()),
        };
    }
    if let Some(fields) = m.get("fields").and_then(|v| v.as_object()) {
        return Value::Object(fields.iter().map(|(k, v)| (k.clone(), unfire(v))).collect());
    }
    v.clone()
}
