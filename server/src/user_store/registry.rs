use dashmap::DashMap;
use crate::user_store::User;
use std::path::Path;

pub struct UserRegistry {
    users: DashMap<String, User>,
}

impl UserRegistry {
    pub fn new(file_path: &str) -> Self {
        let users = DashMap::new();
        let path = Path::new(file_path);
        
        if path.exists() {
            let mut rdr = csv::Reader::from_path(path).expect("Failed to read CSV");
            for result in rdr.deserialize() {
                let user: User = result.expect("Failed to deserialize user");
                users.insert(user.username.clone(), user);
            }
        }

        Self { users }
    }

    pub fn insert(&self, user: User) {
        self.users.insert(user.username.clone(), user);
    }

    pub fn get_by_username(&self, username: &str) -> Option<User> {
        self.users.get(username).map(|u| u.clone())
    }

    pub fn count(&self) -> usize {
        self.users.len()
    }

    pub fn exists(&self, username: &str) -> bool {
        self.users.contains_key(username)
    }
}
