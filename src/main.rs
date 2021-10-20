use actix_http::{body::Body, Response};
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{error, get, middleware, web, App, Error, HttpResponse, HttpServer, Result};
use chrono::offset::Utc;
use chrono::DateTime;
use markx::html::mark2html;
use serde::Serialize;
use std::fs;
use tera::Tera;

#[derive(Debug, Serialize)]
struct Blog {
    name: String,
    modified: DateTime<Utc>,
}

// store tera template in application state
async fn index(tmpl: web::Data<tera::Tera>) -> Result<HttpResponse, Error> {
    let files = fs::read_dir("blogs")?;

    let blogs: Vec<_> = files
        .map(|file| {
            let path = file.unwrap().path();
            let modified: DateTime<Utc> = fs::metadata(&path).unwrap().modified().unwrap().into();
            let name = String::from(path.file_stem().unwrap().to_str().unwrap());
            Blog {
                name: name,
                modified: modified,
            }
        })
        .collect();

    let mut ctx = tera::Context::new();
    ctx.insert("blogs", &blogs);

    let res = tmpl
        .render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

#[get("/blogs/{blog_slot}")] // <- define path parameters
async fn frend(
    tmpl: web::Data<tera::Tera>,
    web::Path(blog_slot): web::Path<String>,
) -> Result<HttpResponse, Error> {
    let blog_file_name = format!("blogs/{}.md", blog_slot.replace("-", " "));
    let rendered_blog = mark2html(&fs::read_to_string(&blog_file_name)?);

    let mut ctx = tera::Context::new();
    ctx.insert("title", &blog_slot.replace("-", " "));
    ctx.insert("rendered_blog", &rendered_blog);

    let res = tmpl
        .render("blog.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!("Listening on: 127.0.0.1:8080, open browser and visit have a try!");
    HttpServer::new(|| {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

        App::new()
            .data(tera)
            .wrap(middleware::Logger::default()) // enable logger
            .service(web::resource("/").route(web::get().to(index)))
            .service(frend)
            .service(web::scope("").wrap(error_handlers()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> Response<Body> {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        Response::build(res.status())
            .content_type("text/plain")
            .body(e.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error.html", &context);

            match body {
                Ok(body) => Response::build(res.status())
                    .content_type("text/html")
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}
