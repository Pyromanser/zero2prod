use actix_web::HttpResponse;

#[allow(clippy::unused_async)]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
