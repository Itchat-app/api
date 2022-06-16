use crate::extractors::*;
use crate::structures::*;
use crate::utils::*;
use serde::Deserialize;
use validator::Validate;


#[utoipa::path(
    get,
    path = "/messages/{id}",
    responses((status = 200, body = Message), (status = 400, body = Error)),
    params(("id" = u64, path))
)]
pub async fn fetch_one(
    Extension(user): Extension<User>,
    Path((channel_id, id)): Path<(u64, u64)>,
) -> Result<Json<Message>> {
    let msg = id.message().await?;

    if msg.channel_id != channel_id {
        return Err(Error::MissingAccess);
    }

    let permissions = Permissions::fetch(&user, None, channel_id.into()).await?;

    permissions.has(Permissions::VIEW_CHANNEL)?;
    permissions.has(Permissions::READ_MESSAGE_HISTORY)?;

    Ok(Json(msg))
}