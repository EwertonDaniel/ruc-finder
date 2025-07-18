use anyhow::{anyhow, Result};
use log::info;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

use crate::models::RucInfo;

fn get_field(item: &Value, keys: &[&str]) -> String {
    for &k in keys {
        if let Some(v) = item.get(k) {
            return match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => String::new(),
            };
        }
    }
    String::new()
}

pub fn calc_dv(ruc: &str) -> Option<u8> {
    if ruc.is_empty() || ruc.len() > 8 || !ruc.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut peso = 2u32;
    let mut soma = 0u32;
    for ch in ruc.chars().rev() {
        let dig = ch.to_digit(10)?;
        soma += dig * peso;
        peso = if peso == 9 { 2 } else { peso + 1 };
    }
    let resto = soma % 11;
    let dv = if resto == 0 { 0 } else { (11 - resto) % 10 };
    Some(dv as u8)
}

pub async fn scrape_guest(query: &str) -> Result<Vec<RucInfo>> {
    let client = Client::builder().cookie_store(true).build()?;

    let html = client
        .get("https://ruc.sgi.com.py/")
        .send()
        .await?
        .text()
        .await?;

    let re_token = Regex::new(r#"['_]token'\s*:\s*'([0-9A-Za-z]+)'"#)?;
    let token = re_token
        .captures(&html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| anyhow!("_token not found"))?;

    let url = format!("https://ruc.sgi.com.py/guest/{}", query);
    let resp = client.post(&url).form(&[("_token", token)]).send().await?;
    if !resp.status().is_success() {
        return Ok(vec![]);
    }

    let arr: Value = resp.json().await?;
    let list = arr.as_array().cloned().unwrap_or_default();

    let mut results = Vec::new();
    for item in list {
        let rucn = get_field(&item, &["RUCN", "rucn", "ruc"]);
        let dvn = get_field(&item, &["DVN", "dvn", "dv"]);
        let nombre = get_field(&item, &["NOMBRE", "nombre"]);
        let ape = get_field(&item, &["APELLIDO", "apellido"]);
        let estado = get_field(&item, &["ESTADO", "estado"]);

        let full_name = if nombre.trim().is_empty() {
            ape.clone()
        } else {
            format!("{ape} {nombre}")
        };

        info!("Result => {rucn}-{dvn} | {full_name} [{estado}]");
        results.push(RucInfo {
            ruc: rucn,
            dv: dvn,
            name: full_name,
            status: estado,
        });
    }
    Ok(results)
}
