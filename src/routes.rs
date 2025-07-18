use actix_web::{get, web, HttpResponse, Responder};
use log::info;

use crate::services::{calc_dv, scrape_guest};

#[get("/scrape/{term}")]
async fn scrape(path: web::Path<String>) -> impl Responder {
    let raw = path.into_inner();
    let re = regex::Regex::new(r"^(?P<num>\d{3,8})(?:-(?P<dv>\d))?$").unwrap();
    let caps = match re.captures(&raw) {
        Some(c) => c,
        None => return HttpResponse::BadRequest().body(" Invalid RUC Format"),
    };
    let base = caps.name("num").unwrap().as_str();
    let provided_dv = caps.name("dv").map(|m| m.as_str());

    if let Some(dv) = provided_dv {
        if let Some(calc) = calc_dv(base) {
            if dv != calc.to_string() {
                return HttpResponse::BadRequest().body(format!("Bad check digit: expected {}", calc));
            }
        }
    }

    info!("Scraping /guest to base RUC: {base}");
    match scrape_guest(base).await {
        Ok(list) if !list.is_empty() => HttpResponse::Ok().json(list),
        Ok(_) => HttpResponse::NotFound().body("No results found"),
        Err(e) => {
            info!("Scraper error: {e}");
            HttpResponse::InternalServerError().body("Scraper error")
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(scrape);
}
