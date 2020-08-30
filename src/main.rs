use actix_web::{guard, web, App, HttpResponse, HttpServer, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_actix_web::{GQLRequest, GQLResponse};
use db::make_db_pool;
use dotenv;

mod db;

struct Query;

#[Object]
impl Query {
    #[field(desc = "Returns the sum of a and b")]
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    // async fn value_from_db(&self, ctx: &Context<'_>) -> FieldResult<String> {
    //     let conn = ctx.data::<PgPool>()?;
    //     Ok(String::new())
    // }
}

type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

async fn index(schema: web::Data<MySchema>, req: GQLRequest) -> GQLResponse {
    req.into_inner().execute(&schema).await.into()
}

async fn index_playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/").subscription_endpoint("/"),
        )))
}

#[async_std::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let db_pool = make_db_pool().await;

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(db_pool)
        .finish();

    println!("Playground: http://localhost:8000");

    HttpServer::new(move || {
        App::new()
            .data(schema.clone())
            .service(web::resource("/").guard(guard::Post()).to(index))
            .service(web::resource("/").guard(guard::Get()).to(index_playground))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
