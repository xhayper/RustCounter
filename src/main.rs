mod manager;
mod utility;

#[macro_use]
extern crate rocket;

use crate::manager::{SvgGenerateOptions, ThemeManager};
use dotenv::dotenv;
use rocket::fairing::AdHoc;
use rocket::fs::{relative, FileServer};
use rocket::http::Header;
use rocket::State;
use rocket::{fairing, Build, Responder, Rocket};
use rocket_db_pools::sqlx::{self};
use rocket_db_pools::{Connection, Database};
use rocket_dyn_templates::context;
use rocket_dyn_templates::Template;
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
    SvgSuccess(String, Header<'static>),
    #[response(status = 200, content_type = "image/png")]
    PngSuccess(Vec<u8>, Header<'static>),
    #[response(status = 400, content_type = "plain")]
    Failed(String),
}

fn respond_svg(
    app_state: &State<AppState>,
    options: SvgGenerateOptions,
    cache: bool,
) -> CountResponse {
    let svg = app_state.theme_manager.generate_svg(&options);

    if svg.is_none() {
        return CountResponse::Failed("failed to generate png".to_string());
    }

    let svg = svg.unwrap();
    CountResponse::SvgSuccess(
        svg,
        Header::new(
            if !cache { "Cache-Control" } else { "a" },
            if !cache {
                "max-age=0, no-cache, no-store, must-revalidate"
            } else {
                "a"
            },
        ),
    )
}

fn respond_png(
    app_state: &State<AppState>,
    options: SvgGenerateOptions,
    cache: bool,
) -> CountResponse {
    let svg = app_state.theme_manager.generate_svg(&options);

    if svg.is_none() {
        return CountResponse::Failed("failed to generate png".to_string());
    }

    let svg = svg
        .unwrap()
        .replace(" style='image-rendering: pixelated;'", "");

    let png = utility::svg_to_png(svg.as_bytes(), options.pixelated);

    if png.is_none() {
        return CountResponse::Failed("failed to generate png".to_string());
    }

    let png = png.unwrap();
    CountResponse::PngSuccess(
        png,
        Header::new(
            "Cache-Control",
            if cache {
                "max-age=31536000, public"
            } else {
                "max-age=0, no-cache, no-store, must-revalidate"
            },
        ),
    )
}

fn validate_options(
    app_state: &State<AppState>,
    id: &str,
    theme: Option<&str>,
    length: Option<u8>,
    format: Option<&str>,
) -> Option<CountResponse> {
    if id.len() > 256 {
        return Some(CountResponse::Failed("id too long".to_string()));
    };

    if id.is_empty() {
        return Some(CountResponse::Failed("id too short".to_string()));
    };

    if theme.is_some() && !app_state.theme_manager.themes.contains_key(theme.unwrap()) {
        return Some(CountResponse::Failed("theme not found".to_string()));
    };

    if length.is_some() && length.unwrap() > 12 {
        return Some(CountResponse::Failed("length too long".to_string()));
    };

    if format.is_some() && format.unwrap() != "svg" && format.unwrap() != "png" {
        return Some(CountResponse::Failed("invalid format".to_string()));
    };

    None
}

fn respond_with_dynamic_format(
    app_state: &State<AppState>,
    format: &str,
    options: SvgGenerateOptions,
    cache: bool,
) -> CountResponse {
    if format == "png" {
        respond_png(app_state, options, cache)
    } else if format == "svg" {
        respond_svg(app_state, options, cache)
    } else {
        CountResponse::Failed("invalid format".to_string())
    }
}

#[get("/number/<number>?<theme>&<pixelated>&<length>&<format>")]
fn number(
    app_state: &State<AppState>,
    number: u64,
    theme: Option<&str>,
    pixelated: Option<bool>,
    length: Option<u8>,
    format: Option<&str>,
) -> CountResponse {
    if let Some(response) = validate_options(app_state, "placeholder", theme, length, format) {
        return response;
    }

    let options = SvgGenerateOptions {
        count: number,
        theme: theme.unwrap_or("moebooru"),
        pixelated: pixelated.unwrap_or(true),
        length: length.unwrap_or(7),
    };

    respond_with_dynamic_format(app_state, format.unwrap_or("svg"), options, true)
}

#[get("/count/<id>?<theme>&<pixelated>&<length>&<format>")]
async fn count(
    mut db: Connection<Counts>,
    app_state: &State<AppState>,
    id: &str,
    theme: Option<&str>,
    pixelated: Option<bool>,
    length: Option<u8>,
    format: Option<&str>,
) -> CountResponse {
    if let Some(response) = validate_options(app_state, id, theme, length, format) {
        return response;
    }

    let count = match sqlx::query!("SELECT count FROM counts WHERE id = $1", id)
        .fetch_one(&mut **db)
        .await
    {
        Ok(record) => record.count.unwrap_or(0) + 1,
        Err(..) => 1,
    };

    let options = SvgGenerateOptions {
        count: count as u64,
        theme: theme.unwrap_or("moebooru"),
        pixelated: pixelated.unwrap_or(true),
        length: length.unwrap_or(7),
    };

    let new_count = options.count as i64;

    sqlx::query!(
        "INSERT INTO counts (id, count) VALUES ($1, $2) ON CONFLICT(id) DO UPDATE SET count = $2",
        id,
        new_count
    )
    .execute(&mut **db)
    .await
    .ok();

    respond_with_dynamic_format(app_state, format.unwrap_or("svg"), options, false)
}

#[get("/")]
fn index() -> Template {
    Template::render(
        "index",
        context! {
            base_url: std::env::var("BASE_URL").unwrap_or("http://127.0.0.1:8000/".to_string())
        },
    )
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
    dotenv().ok();

    let mut theme_manager = ThemeManager::new();
    theme_manager.load();

    rocket::build()
        .attach(sqlx_stage())
        .manage(AppState { theme_manager })
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![index, count, number])
        .attach(Template::fairing())
}
