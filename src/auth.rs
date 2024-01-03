use std::collections::HashMap;

use crate::ChatResult;

pub struct UserData {
    nick: String,
    password: String,
}

impl UserData {
    fn new(nick: String, password: String) -> Self {
        Self { nick, password }
    }
}

pub struct Login {
    users: HashMap<String, UserData>,
}

impl Login {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    pub fn signup(&mut self, email: String, password: String) -> ChatResult<String> {
        if self.users.contains_key(&email) {
            Err("User with email {email} already exists")?;
        }
        self.users
            .insert(email.clone(), UserData::new(email.clone(), password));
        Ok(email)
    }
    pub fn login(&mut self, email: String, password: String) -> ChatResult<String> {
        let user = self.users.get(&email).ok_or("User not found")?;
        if user.password != password {
            Err("Invalid password")?;
        }
        Ok(user.nick.clone())
    }
    pub fn update_nick(&mut self, email: String, nick: String) -> ChatResult<String> {
        let user = self.users.get_mut(&email).ok_or("User not found")?;
        user.nick = nick.clone();
        Ok(nick)
    }
}

impl Default for Login {
    fn default() -> Self {
        Self::new()
    }
}
