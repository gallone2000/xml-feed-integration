use feed_rs::parser;
use regex::Regex;
use scraper::{Html, Selector};
use std::env;

const PAGE_SIZE: usize = 50;

struct PostDetails {
    title: String,
    img_url: Option<String>,
    body: String,
}

fn html_to_text(html: &str) -> String {
    // Prende solo il testo dopo il primo </a> (salta l'immagine di copertina)
    let after_cover = html
        .find("</a>")
        .map(|i| &html[i + 4..])
        .unwrap_or(html);

    // Tronca prima della prossima immagine Blogger (immagine voto/rating a fine recensione).
    // Funziona sia quando l'immagine è avvolta in un <a> (post Evidence) sia quando è un
    // <img> standalone (post EPMD): cerchiamo src="https://blogger..." e torniamo indietro
    // fino al < che apre il tag.
    let body_html = if let Some(src_pos) = after_cover.find(r#"src="https://blogger.googleusercontent.com"#) {
        let tag_start = after_cover[..src_pos].rfind('<').unwrap_or(src_pos);
        &after_cover[..tag_start]
    } else {
        after_cover
    };

    // Rimuove elementi non testuali con il loro contenuto (no backreference in Rust regex)
    let re_script  = Regex::new(r"(?si)<script[^>]*>.*?</script>").unwrap();
    let re_style   = Regex::new(r"(?si)<style[^>]*>.*?</style>").unwrap();
    let re_object  = Regex::new(r"(?si)<object[^>]*>.*?</object>").unwrap();
    let re_embed   = Regex::new(r"(?si)<embed[^>]*>.*?</embed>").unwrap();
    let text = re_script.replace_all(body_html, "");
    let text = re_style.replace_all(&text, "");
    let text = re_object.replace_all(&text, "");
    let text = re_embed.replace_all(&text, "");

    // <br> → newline
    let re_br = Regex::new(r"<br\s*/?>").unwrap();
    let text = re_br.replace_all(&text, "\n");

    // Strisce tutti i tag rimanenti
    let re_tags = Regex::new(r"<[^>]+>").unwrap();
    let text = re_tags.replace_all(&text, "");

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
    let re_spaces = Regex::new(r"[ \t]{2,}").unwrap();
    let text = re_spaces.replace_all(&text, " ");
    let re_newlines = Regex::new(r"\n{3,}").unwrap();
    let text = re_newlines.replace_all(&text, "\n\n");

    text.trim().to_string()
}

fn fetch_post_details(url: &str, client: &reqwest::blocking::Client) -> PostDetails {
    let html = client
        .get(url)
        .send()
        .and_then(|r| r.text())
        .unwrap_or_default();

    let document = Html::parse_document(&html);

    let title = Selector::parse(r#"meta[property="og:title"]"#).ok()
        .and_then(|s| document.select(&s).next())
        .and_then(|el| el.value().attr("content"))
        .unwrap_or("(nessun titolo)")
        .to_string();

    let img_url = Selector::parse(r#"meta[property="og:image"]"#).ok()
        .and_then(|s| document.select(&s).next())
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());

    let body = Selector::parse("div.post-body").ok()
        .and_then(|s| document.select(&s).next())
        .map(|el| html_to_text(&el.inner_html()))
        .unwrap_or_else(|| "(contenuto non disponibile)".to_string());

    PostDetails { title, img_url, body }
}

pub fn fetch_and_print() {
    dotenvy::dotenv().ok();
    let base_url = env::var("FEED_URL").expect("FEED_URL non trovata nel file .env");

    let client = reqwest::blocking::Client::new();
    let mut start_index = 1;
    let mut total = 0;
    let mut page = 1;

    loop {
        let url = format!("{}&start-index={}&max-results={}", base_url, start_index, PAGE_SIZE);
        println!("--- Pagina {page} (post {start_index}–{}) ---", start_index + PAGE_SIZE - 1);

        let response = client.get(&url)
            .send()
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
            let post_url = entry.links.first().map(|l| l.href.as_str()).unwrap_or("");
            let details = fetch_post_details(post_url, &client);

            println!("  Titolo : {}", details.title);
            println!("  Img    : {}", details.img_url.as_deref().unwrap_or("(nessuna immagine)"));
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
