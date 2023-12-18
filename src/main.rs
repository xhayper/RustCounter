mod manager;

#[macro_use]
extern crate rocket;

use crate::manager::{SvgGenerateOptions, ThemeManager};
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::http::Header;
use rocket::State;
use rocket::{fairing, Build, Responder, Rocket};
use rocket_db_pools::sqlx::{self};
use rocket_db_pools::{Connection, Database};
use sqlx::SqlitePool;

#[derive(Database)]
#[database("sqlite_counts")]
struct Counts(SqlitePool);

struct AppState {
    theme_manager: ThemeManager<'static>,
}

#[derive(Responder)]
enum CountResponse {
    #[response(status = 200, content_type = "image/svg+xml")]
    Success(String, Header<'static>),
    #[response(status = 400, content_type = "plain")]
    Failed(String),
}

fn respond_svg(app_state: &State<AppState>, options: SvgGenerateOptions) -> CountResponse {
    let svg = app_state.theme_manager.generate_svg(&options).unwrap();

    CountResponse::Success(svg, Header::new("Cache-Control", "no-cache"))
}

fn validate_svg_options(
    app_state: &State<AppState>,
    id: &str,
    theme: Option<&str>,
    length: Option<u8>,
) -> Option<CountResponse> {
    if id.len() > 256 {
        return Some(CountResponse::Failed("id too long".to_string()));
    };

    if id.len() <= 0 {
        return Some(CountResponse::Failed("id too short".to_string()));
    };

    if theme.is_some() && !app_state.theme_manager.themes.contains_key(theme.unwrap()) {
        return Some(CountResponse::Failed("theme not found".to_string()));
    };

    if length.is_some() && length.unwrap() > 12 {
        return Some(CountResponse::Failed("length too long".to_string()));
    };

    if length.is_some() && length.unwrap() <= 0 {
        return Some(CountResponse::Failed("length too short".to_string()));
    };

    None
}

#[get("/number/<number>?<theme>&<pixelated>&<length>")]
fn number(
    app_state: &State<AppState>,
    number: u64,
    theme: Option<&str>,
    pixelated: Option<bool>,
    length: Option<u8>,
) -> CountResponse {
    // too lazy lmao
    if let Some(response) = validate_svg_options(app_state, "placeholder", theme, length) {
        return response;
    }

    let options = SvgGenerateOptions {
        count: number,
        theme: theme.unwrap_or("moebooru"),
        pixelated: pixelated.unwrap_or(true),
        length: length.unwrap_or(7),
    };

    respond_svg(app_state, options)
}

#[get("/count/<id>?<theme>&<pixelated>&<length>")]
async fn count(
    mut db: Connection<Counts>,
    app_state: &State<AppState>,
    id: &str,
    theme: Option<&str>,
    pixelated: Option<bool>,
    length: Option<u8>,
) -> CountResponse {
    if let Some(response) = validate_svg_options(app_state, id, theme, length) {
        return response;
    }

    let count = match sqlx::query!("SELECT count FROM counts WHERE id = $1", id)
        .fetch_one(&mut **db)
        .await
    {
        Ok(record) => record.count.unwrap_or(0),
        Err(..) => 0,
    };

    let options = SvgGenerateOptions {
        count: count as u64,
        theme: theme.unwrap_or("moebooru"),
        pixelated: pixelated.unwrap_or(true),
        length: length.unwrap_or(7),
    };

    let new_count = (options.count + 1) as i64;

    sqlx::query!(
        "INSERT INTO counts (id, count) VALUES ($1, $2) ON CONFLICT(id) DO UPDATE SET count = $2",
        id,
        new_count
    )
    .execute(&mut **db)
    .await
    .ok();

    respond_svg(app_state, options)
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Counts::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

pub fn sqlx_stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(Counts::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
    })
}

#[launch]
async fn rocket() -> _ {
    let mut theme_manager = ThemeManager::new();
    theme_manager.load();

    rocket::build()
        .attach(sqlx_stage())
        .manage(AppState { theme_manager })
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![count, number])
}
