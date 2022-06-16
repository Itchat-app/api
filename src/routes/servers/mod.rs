pub mod channels;
pub mod invites;
pub mod members;
pub mod roles;
pub mod servers;

pub fn routes() -> axum::Router {
    use axum::{routing::*, Router};

    Router::new()
        .route("/", post(create_server).get(fetch_servers))
        .route("/:server_id", get(fetch_server).delete(delete_server))
}


pub fn routes() -> axum::Router {
    use crate::middlewares::*;
    use axum::{middleware, Router};

    Router::new()
        .nest("/", servers::routes())
        .nest("/:server_id/channels", channels::routes())
        .nest("/:server_id/invites", invites::routes())
        .nest("/:server_id/roles", roles::routes())
        .nest("/:server_id/members", members::routes())
        .layer(middleware::from_fn(ratelimit::handle!(10, 1000 * 10)))
}
