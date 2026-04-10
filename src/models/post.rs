use chrono::{DateTime, Utc};

/// Riga letta dal DB (include id e timestamp gestiti da Postgres).
#[derive(Debug, sqlx::FromRow)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub url: String,
    pub img_url: Option<String>,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

/// Dati necessari per inserire un nuovo post (senza id/created_at).
#[derive(Debug)]
pub struct NewPost {
    pub title: String,
    pub url: String,
    pub img_url: Option<String>,
    pub body: String,
}

/// Campi aggiornabili di un post esistente (url e created_at sono immutabili).
#[derive(Debug)]
pub struct UpdatePost {
    pub title: String,
    pub img_url: Option<String>,
    pub body: String,
}
