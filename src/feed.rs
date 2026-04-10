use feed_rs::parser;
use std::env;

const PAGE_SIZE: usize = 50;

pub fn fetch_and_print() {
    dotenvy::dotenv().ok();
    let base_url = env::var("FEED_URL").expect("FEED_URL non trovata nel file .env");

    let mut start_index = 1;
    let mut total = 0;
    let mut page = 1;

    loop {
        let url = format!("{}&start-index={}&max-results={}", base_url, start_index, PAGE_SIZE);
        println!("--- Pagina {page} (post {start_index}–{}) ---", start_index + PAGE_SIZE - 1);

        let response = reqwest::blocking::get(&url)
            .expect("Errore nella richiesta HTTP")
            .bytes()
            .expect("Errore nella lettura della risposta");

        let feed = parser::parse(&response[..]).expect("Errore nel parsing del feed XML");

        let count = feed.entries.len();

        if count == 0 {
            println!("Nessun post trovato. Fine.");
            break;
        }

        for entry in &feed.entries {
            let title = entry
                .title
                .as_ref()
                .map(|t| t.content.as_str())
                .unwrap_or("(nessun titolo)");
            println!("  - {title}");
            total += 1;
        }

        // Se i risultati sono meno di PAGE_SIZE, siamo all'ultima pagina
        if count < PAGE_SIZE {
            break;
        }

        start_index += PAGE_SIZE;
        page += 1;
    }

    println!("\nTotale post processati: {total}");
}
