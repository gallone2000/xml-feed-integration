use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use xml_feed_fetcher::models::{NewPost, UpdatePost};
use xml_feed_fetcher::repositories::PostRepository;

async fn setup() -> (PostRepository, impl Drop) {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    let pool = PgPool::connect(&db_url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    (PostRepository::new(pool), container)
}

fn sample_post(suffix: &str) -> NewPost {
    NewPost {
        title: format!("Titolo {suffix}"),
        url: format!("https://example.com/post-{suffix}"),
        img_url: Some(format!("https://example.com/img-{suffix}.jpg")),
        body: format!("Corpo del post {suffix}."),
    }
}

// ── INSERT ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_insert_returns_post() {
    let (repo, _c) = setup().await;
    let new = sample_post("insert");

    let result = repo.insert(&new).await.unwrap();

    assert!(result.is_some(), "dovrebbe restituire il post inserito");
    let post = result.unwrap();
    assert_eq!(post.title, new.title);
    assert_eq!(post.url, new.url);
    assert_eq!(post.img_url.as_deref(), new.img_url.as_deref());
    assert_eq!(post.body, new.body);
    assert!(post.id > 0);
}

#[tokio::test]
async fn test_insert_duplicate_returns_none() {
    let (repo, _c) = setup().await;
    let new = sample_post("dup");

    repo.insert(&new).await.unwrap();
    let second = repo.insert(&new).await.unwrap();

    assert!(second.is_none(), "il duplicato deve restituire None");
}

#[tokio::test]
async fn test_insert_without_image() {
    let (repo, _c) = setup().await;
    let new = NewPost {
        title: "Senza immagine".to_string(),
        url: "https://example.com/no-img".to_string(),
        img_url: None,
        body: "Nessuna copertina.".to_string(),
    };

    let post = repo.insert(&new).await.unwrap().unwrap();

    assert!(post.img_url.is_none());
}

// ── READ ──────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_find_by_id() {
    let (repo, _c) = setup().await;
    let inserted = repo.insert(&sample_post("find-id")).await.unwrap().unwrap();

    let found = repo.find_by_id(inserted.id).await.unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().id, inserted.id);
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let (repo, _c) = setup().await;

    let result = repo.find_by_id(999_999).await.unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_find_by_url() {
    let (repo, _c) = setup().await;
    let new = sample_post("find-url");
    repo.insert(&new).await.unwrap();

    let found = repo.find_by_url(&new.url).await.unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().url, new.url);
}

#[tokio::test]
async fn test_find_all() {
    let (repo, _c) = setup().await;
    repo.insert(&sample_post("all-1")).await.unwrap();
    repo.insert(&sample_post("all-2")).await.unwrap();
    repo.insert(&sample_post("all-3")).await.unwrap();

    let posts = repo.find_all().await.unwrap();

    assert_eq!(posts.len(), 3);
}

// ── UPDATE ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_update() {
    let (repo, _c) = setup().await;
    let inserted = repo.insert(&sample_post("upd")).await.unwrap().unwrap();

    let updated = repo
        .update(
            inserted.id,
            &UpdatePost {
                title: "Titolo aggiornato".to_string(),
                img_url: None,
                body: "Corpo aggiornato.".to_string(),
            },
        )
        .await
        .unwrap();

    assert!(updated.is_some());
    let post = updated.unwrap();
    assert_eq!(post.title, "Titolo aggiornato");
    assert!(post.img_url.is_none());
    assert_eq!(post.body, "Corpo aggiornato.");
    assert_eq!(post.url, inserted.url, "url non deve cambiare");
}

#[tokio::test]
async fn test_update_nonexistent_returns_none() {
    let (repo, _c) = setup().await;

    let result = repo
        .update(
            999_999,
            &UpdatePost {
                title: "X".to_string(),
                img_url: None,
                body: "X".to_string(),
            },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

// ── DELETE ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete() {
    let (repo, _c) = setup().await;
    let inserted = repo.insert(&sample_post("del")).await.unwrap().unwrap();

    let deleted = repo.delete(inserted.id).await.unwrap();
    assert!(deleted, "deve restituire true se il post esisteva");

    let after = repo.find_by_id(inserted.id).await.unwrap();
    assert!(after.is_none(), "il post non deve più esistere");
}

#[tokio::test]
async fn test_delete_nonexistent_returns_false() {
    let (repo, _c) = setup().await;

    let result = repo.delete(999_999).await.unwrap();

    assert!(!result);
}
