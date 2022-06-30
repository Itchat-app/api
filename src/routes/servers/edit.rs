use crate::extractors::*;
use crate::gateway::*;
use crate::structures::*;
use crate::utils::*;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, OpgModel)]
pub struct EditServerOptions {
    #[validate(length(min = 1, max = 50))]
    name: Option<String>,
}

pub async fn edit(
    Extension(user): Extension<User>,
    Path(server_id): Path<u64>,
    ValidatedJson(data): ValidatedJson<EditServerOptions>,
) -> Result<Json<Server>> {
    let mut server = server_id.server(user.id.into()).await?;

    Permissions::fetch_cached(&user, Some(&server), None)
        .await?
        .has(Permissions::MANAGE_SERVER)?;

    if let Some(name) = data.name {
        server.name = name;
    }

    server.update().await;

    publish(server.id, Payload::ServerUpdate(server.clone())).await;

    Ok(Json(server))
}