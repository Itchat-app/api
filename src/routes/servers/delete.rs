use crate::extractors::*;
use crate::gateway::*;
use crate::structures::*;
use crate::utils::*;

pub async fn delete(Extension(user): Extension<User>, Path(server_id): Path<u64>) -> Result<()> {
    let server = server_id.server(user.id.into()).await?;

    if server.owner_id != user.id {
        return Err(Error::MissingAccess);
    }

    server.delete().await;

    publish(server.id, Payload::ServerDelete(server.id.into())).await;

    Ok(())
}