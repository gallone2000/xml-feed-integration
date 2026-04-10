use sqlx::PgPool;

use crate::models::{NewPost, Post, UpdatePost};

/// Repository per le operazioni CRUD sulla tabella `posts`.
///
/// Internamente wrappa un `PgPool` (già reference-counted), quindi
/// il clone è economico e si presta bene all'uso in task async paralleli.
#[derive(Clone)]
pub struct PostRepository {
    pool: PgPool,
}

impl PostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Inserisce un nuovo post.
    /// Ritorna `Some(post)` se inserito, `None` se l'URL era già presente.
    pub async fn insert(&self, post: &NewPost) -> anyhow::Result<Option<Post>> {
        let result = sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (title, url, img_url, body)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (url) DO NOTHING
            RETURNING id, title, url, img_url, body, created_at
            "#,
            post.title,
            post.url,
            post.img_url,
            post.body,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    /// Recupera un post per ID. Ritorna `None` se non esiste.
    pub async fn find_by_id(&self, id: i64) -> anyhow::Result<Option<Post>> {
        let post = sqlx::query_as!(
            Post,
            "SELECT id, title, url, img_url, body, created_at FROM posts WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    /// Recupera un post per URL. Ritorna `None` se non esiste.
    pub async fn find_by_url(&self, url: &str) -> anyhow::Result<Option<Post>> {
        let post = sqlx::query_as!(
            Post,
            "SELECT id, title, url, img_url, body, created_at FROM posts WHERE url = $1",
            url
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    /// Ritorna tutti i post ordinati dal più recente.
    pub async fn find_all(&self) -> anyhow::Result<Vec<Post>> {
        let posts = sqlx::query_as!(
            Post,
            "SELECT id, title, url, img_url, body, created_at FROM posts ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(posts)
    }

    /// Aggiorna title, img_url e body di un post esistente.
    /// Ritorna `Some(post)` con i dati aggiornati, `None` se l'ID non esiste.
    pub async fn update(&self, id: i64, update: &UpdatePost) -> anyhow::Result<Option<Post>> {
        let post = sqlx::query_as!(
            Post,
            r#"
            UPDATE posts
            SET title = $1, img_url = $2, body = $3
            WHERE id = $4
            RETURNING id, title, url, img_url, body, created_at
            "#,
            update.title,
            update.img_url,
            update.body,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(post)
    }

    /// Elimina un post per ID.
    /// Ritorna `true` se il post esisteva ed è stato eliminato, `false` altrimenti.
    pub async fn delete(&self, id: i64) -> anyhow::Result<bool> {
        let result = sqlx::query!("DELETE FROM posts WHERE id = $1", id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
