use actix_web::{web, HttpResponse, Responder};
use actix_web::middleware::session::{Session, RequestSession};
use actix_web_httpauth::middleware::HttpAuthentication;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use serde::{Deserialize, Serialize};

use crate::Pool;

#[derive(Queryable)]
struct User {
    id: i32,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct UserInput {
    username: String,
    password: String,
}

async fn login(
    form: web::Form<UserInput>,
    pool: web::Data<Pool>,
    session: Session,
) -> impl Responder {
    use crate::schema::users::dsl::*;

    let conn = pool.get().expect("Failed to acquire a database connection");

    let user_result = web::block(move || users.filter(username.eq(&form.username)).first::<User>(&conn)).await;

    if let Ok(user) = user_result {
        let password_matched = verify(&form.password, &user.password)
            .expect("Failed to verify password");

        if password_matched {
            session.set("user_id", user.id.to_string()).expect("Failed to set session");

            return HttpResponse::Ok().body("Successfully logged in");
        }
    }

    HttpResponse::Unauthorized().body("Invalid username or password")
}

async fn logout(session: Session) -> impl Responder {
    session.remove("user_id");
    HttpResponse::Ok().body("Successfully logged out")
}

async fn restricted(session: Session) -> impl Responder {
    if let Some(user_id) = session.get::<String>("user_id").unwrap() {
        HttpResponse::Ok().body(format!("Welcome, User ID: {}", user_id))
    } else {
        HttpResponse::Unauthorized().body("Unauthorized access")
    }
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config
        .data(
            web::JsonConfig::default()
                .limit(4096)
                .error_handler(|err, _| {
                    actix_web::error::InternalError::from_response(
                        err,
                        HttpResponse::BadRequest().finish(),
                    )
                    .into()
                }),
        )
        .route("/login", web::post().to(login))
        .route("/logout", web::get().to(logout))
        .route("/restricted", web::get().to(restricted));
}
