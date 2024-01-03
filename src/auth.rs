use std::collections::HashMap;

use crate::ChatResult;

pub struct Auth {
    users: HashMap<String, String>,
}
// TODO: prevent user authenticating on two chats at the same time
// TODO: Salt and hash the passwords
// TODO: Use a database

impl Auth {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn signup(&mut self, nick: String, password: String) -> ChatResult<String> {
        if self.users.contains_key(&nick) {
            Err("User with nick {nick} already exists")?;
        }
        self.users.insert(nick.clone(), password);
        Ok(nick)
    }
    pub fn login(&mut self, nick: String, password: String) -> ChatResult<String> {
        let user_password = self.users.get(&nick).ok_or("User not found")?;
        if user_password != &password {
            Err("Invalid password")?;
        }
        Ok(nick)
    }
    pub fn update_nick(&mut self, nick: String, new_nick: String) -> ChatResult<String> {
        if self.users.contains_key(&new_nick) {
            Err("Nick taken")?;
        }
        let password = self.users.remove(&nick).ok_or("User not found")?;
        let _ = self.users.insert(new_nick.clone(), password);
        Ok(new_nick)
    }
}

impl Default for Auth {
    fn default() -> Self {
        Self::new()
    }
}
