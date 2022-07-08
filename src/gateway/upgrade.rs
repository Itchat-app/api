use super::{client::Client, Payload};
use crate::database::redis::*;
use crate::{
    gateway::Empty,
    utils::{Permissions, Ref},
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    headers,
    response::IntoResponse,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn upgrade(
    ws: WebSocketUpgrade,
    _: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    ws.on_upgrade(handle)
}

async fn handle(ws: WebSocket) {
    let (sender, mut receiver) = ws.split();
    let sender = Arc::new(Mutex::new(sender));

    let client = Client::from(sender, pubsub().await);

    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(content) = msg {
            client.on_message(content).await;

            if client.user.lock().await.is_some() {
                break;
            } else {
                log::debug!("Socket did not authenticate with valid token");
                return;
            }
        }
    }

    let process = tokio::spawn(async move {
        while let Some((channel, payload)) = client.subscriptions.on_message().next().await {
            let target_id: i64 = channel.parse().unwrap();
            let user = client.user.lock().await;
            let user = &*user.as_ref().unwrap();

            let payload: Payload = serde_json::from_str(&payload.as_string().unwrap()).unwrap();
            let mut permissions = client.permissions.lock().await;
            let p = permissions
                .get(&target_id)
                .unwrap_or(&Permissions::ADMINISTRATOR);

            match &payload {
                Payload::MessageCreate(_)
                | Payload::MessageUpdate(_)
                | Payload::MessageDelete(_) => {
                    if p.has(Permissions::VIEW_CHANNEL).is_err() {
                        continue;
                    }
                }

                Payload::ChannelDelete(Empty::Default { id }) => {
                    client.subscriptions.unsubscribe(id.to_string()).await.ok();
                }

                Payload::ChannelUpdate(channel) => {
                    let server = if let Some(server_id) = channel.server_id {
                        Some(server_id.server(None).await.unwrap())
                    } else {
                        None
                    };

                    let p = Permissions::fetch_cached(user, server.as_ref(), channel.into())
                        .await
                        .unwrap();

                    permissions.insert(channel.id, p);
                }

                Payload::ServerMemberUpdate(member) => {
                    if member.id == user.id {
                        let p = Permissions::fetch(user, member.server_id.into(), None)
                            .await
                            .unwrap();
                        permissions.insert(member.server_id, p);
                    }
                }

                Payload::ServerMemberLeave(Empty::ServerObject { id, .. }) => {
                    if *id == user.id {
                        client
                            .subscriptions
                            .unsubscribe(target_id.to_string())
                            .await
                            .ok();
                    }
                }

                Payload::ServerDelete(_) => {
                    client
                        .subscriptions
                        .unsubscribe(target_id.to_string())
                        .await
                        .ok();
                }

                Payload::ServerCreate(server) => {
                    client
                        .subscriptions
                        .subscribe(server.id.to_string())
                        .await
                        .ok();
                }

                Payload::ChannelCreate(channel) => {
                    client
                        .subscriptions
                        .subscribe(channel.id.to_string())
                        .await
                        .ok();

                    if !channel.in_server() {
                        permissions.insert(
                            channel.id,
                            Permissions::fetch_cached(user, None, channel.into())
                                .await
                                .unwrap(),
                        );
                    }
                }
                _ => {}
            }

            if client.send(payload).await.is_err() {
                break;
            }
        }
    });

    if let Err(err) = process.await {
        log::error!("Socket disconnected with error: {:?}", err);
    } else {
        log::debug!("Socket connection closed");
    }
}
