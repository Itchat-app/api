use fred::interfaces::PubsubInterface;

use crate::database::pool;
use crate::gateway::{
    client::Client,
    payload::{ClientPayload, Payload},
};
use crate::structures::*;
use crate::utils::Permissions;

pub async fn run(client: &Client, payload: ClientPayload) {
    if client.user.lock().await.is_some() {
        return;
    }

    let user = if let ClientPayload::Authenticate { token } = payload {
        User::fetch_by_token(token.as_str()).await
    } else {
        None
    };

    if user.is_none() {
        return;
    }

    client.send(Payload::Authenticated).await.ok();

    let user = user.unwrap();

    *client.user.lock().await = Some(user.clone());

    let mut subscriptions: Vec<i64> = vec![user.id];
    let mut permissions = client.permissions.lock().await;
    let mut channels = user.fetch_channels().await.unwrap();
    let servers = user.fetch_servers().await.unwrap();
    let users = user.fetch_relations().await.unwrap();

    if !servers.is_empty() {
        let mut server_ids: String = servers.iter().map(|s| s.id.to_string() + ",").collect();
        server_ids.remove(server_ids.len() - 1);

        let mut other_channels = Channel::query(&format!(
            "SELECT * FROM {} WHERE server_id = ({})",
            Channel::table_name(),
            server_ids
        ))
        .fetch_all(pool())
        .await
        .unwrap();

        channels.append(&mut other_channels);
    }

    for server in &servers {
        subscriptions.push(server.id);
        permissions.insert(
            server.id,
            Permissions::fetch_cached(&user, server.into(), None)
                .await
                .unwrap(),
        );
    }

    for channel in &channels {
        let server = if let Some(server_id) = channel.server_id {
            servers.iter().find(|s| s.id == server_id)
        } else {
            None
        };

        subscriptions.push(channel.id);
        permissions.insert(
            channel.id,
            Permissions::fetch_cached(&user, server, channel.into())
                .await
                .unwrap(),
        );
    }

    for id in subscriptions {
        client.subscriptions.subscribe(id.to_string()).await.ok();
    }

    client
        .send(Payload::Ready {
            user,
            users,
            servers,
            channels,
        })
        .await
        .ok();
}
