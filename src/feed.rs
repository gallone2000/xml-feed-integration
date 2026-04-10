use feed_rs::parser;
use regex::Regex;
use scraper::{Html, Selector};
use sqlx::PgPool;
use std::env;
use std::sync::{Arc, OnceLock};
use tokio::sync::Semaphore;
use tracing::error;

use crate::models::NewPost;
use crate::repositories::PostRepository;

const PAGE_SIZE: usize = 2;
const MAX_CONCURRENT: usize = 10;

pub(crate) struct PostDetails {
    pub title: String,
    pub img_url: Option<String>,
    pub body: String,
}

struct Regexes {
    script: Regex,
    style: Regex,
    object: Regex,
    embed: Regex,
    br: Regex,
    tags: Regex,
    spaces: Regex,
    newlines: Regex,
}

static REGEXES: OnceLock<Regexes> = OnceLock::new();

fn regexes() -> &'static Regexes {
    REGEXES.get_or_init(|| Regexes {
        script: Regex::new(r"(?si)<script[^>]*>.*?</script>").unwrap(),
        style: Regex::new(r"(?si)<style[^>]*>.*?</style>").unwrap(),
        object: Regex::new(r"(?si)<object[^>]*>.*?</object>").unwrap(),
        embed: Regex::new(r"(?si)<embed[^>]*>.*?</embed>").unwrap(),
        br: Regex::new(r"<br\s*/?>").unwrap(),
        tags: Regex::new(r"<[^>]+>").unwrap(),
        spaces: Regex::new(r"[ \t]{2,}").unwrap(),
        newlines: Regex::new(r"\n{3,}").unwrap(),
    })
}

fn html_to_text(html: &str) -> String {
    // Prende solo il testo dopo il primo </a> (salta l'immagine di copertina)
    let after_cover = html.find("</a>").map(|i| &html[i + 4..]).unwrap_or(html);

    // Tronca prima della prossima immagine Blogger (immagine voto/rating a fine recensione).
    // Funziona sia quando l'immagine è avvolta in un <a> (post Evidence) sia quando è un
    // <img> standalone (post EPMD): cerchiamo src="https://blogger..." e torniamo indietro
    // fino al < che apre il tag.
    let body_html =
        if let Some(src_pos) = after_cover.find(r#"src="https://blogger.googleusercontent.com"#) {
            let tag_start = after_cover[..src_pos].rfind('<').unwrap_or(src_pos);
            &after_cover[..tag_start]
        } else {
            after_cover
        };

    let re = regexes();
    let text = re.script.replace_all(body_html, "");
    let text = re.style.replace_all(&text, "");
    let text = re.object.replace_all(&text, "");
    let text = re.embed.replace_all(&text, "");

    // <br> → newline
    let text = re.br.replace_all(&text, "\n");

    // Strisce tutti i tag rimanenti
    let text = re.tags.replace_all(&text, "");

    // Decode entità HTML comuni
    let text = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
        .replace("&#171;", "«")
        .replace("&#187;", "»")
        .replace("&#8216;", "\u{2018}")
        .replace("&#8217;", "\u{2019}")
        .replace("&#8220;", "\u{201C}")
        .replace("&#8221;", "\u{201D}");

    // Normalizza spazi e newline
    let text = re.spaces.replace_all(&text, " ");
    let text = re.newlines.replace_all(&text, "\n\n");

    text.trim().to_string()
}

async fn fetch_post_details(url: String, client: reqwest::Client) -> PostDetails {
    let html = async { client.get(&url).send().await?.text().await }
        .await
        .unwrap_or_default();

    let document = Html::parse_document(&html);

    let title = Selector::parse(r#"meta[property="og:title"]"#)
        .ok()
        .and_then(|s| document.select(&s).next())
        .and_then(|el| el.value().attr("content"))
        .unwrap_or("(nessun titolo)")
        .to_string();

    let img_url = Selector::parse(r#"meta[property="og:image"]"#)
        .ok()
        .and_then(|s| document.select(&s).next())
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());

    let body = Selector::parse("div.post-body")
        .ok()
        .and_then(|s| document.select(&s).next())
        .map(|el| html_to_text(&el.inner_html()))
        .unwrap_or_else(|| "(contenuto non disponibile)".to_string());

    PostDetails {
        title,
        img_url,
        body,
    }
}

pub async fn fetch_and_print() {
    dotenvy::dotenv().ok();
    let base_url = env::var("FEED_URL").expect("FEED_URL non trovata nel file .env");
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL non trovata nel file .env");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Impossibile connettersi al database");

    let client = reqwest::Client::new();
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let repo = PostRepository::new(pool);
    let mut start_index = 1;
    let mut total = 0;
    let mut page = 1;

    loop {
        let url = format!(
            "{}&start-index={}&max-results={}",
            base_url, start_index, PAGE_SIZE
        );
        println!(
            "--- Pagina {page} (post {start_index}–{}) ---",
            start_index + PAGE_SIZE - 1
        );

        let response = client
            .get(&url)
            .send()
            .await
            .expect("Errore nella richiesta HTTP")
            .bytes()
            .await
            .expect("Errore nella lettura della risposta");

        let feed = match parser::parse(&response[..]) {
            Ok(f) => f,
            Err(_) => {
                println!("Fine del feed (nessuna pagina valida oltre l'indice {start_index}).");
                break;
            }
        };
        let count = feed.entries.len();

        if count == 0 {
            println!("Nessun post trovato. Fine.");
            break;
        }

        // Lancia tutti i fetch della pagina in parallelo (max MAX_CONCURRENT simultanei)
        let handles: Vec<_> = feed
            .entries
            .iter()
            .map(|entry| {
                let post_url = entry
                    .links
                    .first()
                    .map(|l| l.href.clone())
                    .unwrap_or_default();
                let client = client.clone();
                let sem = semaphore.clone();
                let repo = repo.clone();
                tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    let details = fetch_post_details(post_url.clone(), client).await;
                    (post_url, details, repo)
                })
            })
            .collect();

        // Stampa i risultati nell'ordine originale e persiste su DB
        for handle in handles {
            let (post_url, details, repo) = handle.await.expect("Task fallito");

            let new_post = NewPost {
                title: details.title.clone(),
                url: post_url.clone(),
                img_url: details.img_url.clone(),
                body: details.body.clone(),
            };

            match repo.insert(&new_post).await {
                Ok(Some(_)) => println!("  [DB] Salvato: {}", post_url),
                Ok(None) => println!("  [DB] Già presente (skip): {}", post_url),
                Err(e) => error!(url = %post_url, err = %e, "Errore inserimento post nel DB"),
            }

            println!("  Titolo : {}", details.title);
            println!(
                "  Img    : {}",
                details.img_url.as_deref().unwrap_or("(nessuna immagine)")
            );
            println!("  Post   :\n{}", details.body);
            println!("  URL    : {post_url}");
            println!();
            total += 1;
        }

        if count < PAGE_SIZE {
            break;
        }

        start_index += PAGE_SIZE;
        page += 1;
    }

    println!("Totale post processati: {total}");
}
