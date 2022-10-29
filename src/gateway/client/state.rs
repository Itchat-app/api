use crate::structures::User;
use crate::utils::Permissions;
use dashmap::DashMap;
use tokio::sync::Mutex;

pub struct SocketClientState {
    pub permissions: DashMap<i64, Permissions>,
    pub user: Mutex<User>,
    pub user_id: i64,
}

impl SocketClientState {
    pub fn new(user: User) -> Self {
        Self {
            permissions: DashMap::new(),
            user_id: user.id,
            user: Mutex::new(user),
        }
    }
}
