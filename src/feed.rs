use feed_rs::parser;
use std::env;

pub fn fetch_and_print() {
    dotenvy::dotenv().ok();
    let start_url = env::var("FEED_URL").expect("FEED_URL non trovata nel file .env");

    let mut current_url = start_url;
    let mut total = 0;
    let mut page = 1;

    loop {
        println!("--- Pagina {page} ({current_url}) ---");

        let response = reqwest::blocking::get(&current_url)
            .expect("Errore nella richiesta HTTP")
            .bytes()
            .expect("Errore nella lettura della risposta");

        let feed = parser::parse(&response[..]).expect("Errore nel parsing del feed XML");

        if feed.entries.is_empty() {
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

        // Cerca il link alla pagina successiva (standard Atom: <link rel="next">)
        let next = feed.links.iter().find(|l| l.rel.as_deref() == Some("next"));
        match next {
            Some(link) => {
                current_url = link.href.clone();
                page += 1;
            }
            None => {
                println!("\nNessuna pagina successiva trovata. Fine.");
                break;
            }
        }
    }

    println!("\nTotale post processati: {total}");
}
