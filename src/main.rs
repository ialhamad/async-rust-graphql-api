use anyhow::Result;
use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Context, EmptyMutation, EmptySubscription, FieldResult, Object, Schema,
};
use sqlx::{query, PgPool, Pool};
use tide::{self, http::mime, Body, Request, Response, StatusCode};

#[derive(Debug)]
struct Query;

#[Object]
impl Query {
    #[field(desc = "Returns the sum of a and b")]
    async fn add(&self, ctx: &Context<'_>) -> FieldResult<i32> {
        let conn = ctx.data::<PgPool>()?;
        let row = query!("select 1 as one").fetch_one(conn).await?;
        Ok(row.one.unwrap())
    }
}

#[derive(Clone)]
struct AppState {
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
}

async fn graphql(req: Request<AppState>) -> tide::Result<Response> {
    let schema = req.state().schema.clone();
    async_graphql_tide::graphql(req, schema, |query_builder| query_builder).await
}

#[async_std::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db_pool: PgPool = Pool::new(&db_url).await?;

    let row = query!("select 1 as one").fetch_all(&db_pool).await?;

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(db_pool)
        .finish();

    let app_state = AppState { schema };
    let mut app = tide::with_state(app_state);
    app.at("/graphql").post(graphql).get(graphql);
    app.at("/").get(|_| async move {
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(Body::from_string(playground_source(
            GraphQLPlaygroundConfig::new("/graphql"),
        )));
        resp.set_content_type(mime::HTML);
        Ok(resp)
    });
    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
