use crate::extractors::*;
use crate::gateway::*;
use crate::structures::*;
use crate::utils::*;
use inter_struct::prelude::*;
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate, OpgModel, StructMerge)]
#[struct_merge("crate::structures::channel::Channel")]
pub struct EditGroupOptions {
    #[validate(length(min = 3, max = 32))]
    name: Option<String>,
}

pub async fn edit(
    Extension(user): Extension<User>,
    ValidatedJson(data): ValidatedJson<EditGroupOptions>,
    Path(id): Path<i64>,
) -> Result<Json<Channel>> {
    let mut group = id.channel(user.id.into()).await?;

    Permissions::fetch_cached(&user, None, Some(&group))
        .await?
        .has(bits![MANAGE_CHANNELS])?;

    group.merge(data);

    let group = group.update_all_fields(pool()).await?;

    Payload::ChannelUpdate(group.clone()).to(group.id).await;

    Ok(Json(group))
}
