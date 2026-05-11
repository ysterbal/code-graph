//! Sample Rust module demonstrating Rust parsing capabilities

use std::collections::HashMap;

/// A simple data structure for storing user information
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

impl User {
    /// Creates a new user with the given ID and name
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            email: String::new(),
        }
    }

    /// Sets the user's email address
    pub fn set_email(&mut self, email: &str) {
        self.email = email.to_string();
    }

    /// Returns a formatted display string
    pub fn display(&self) -> String {
        format!("User {}: {}", self.id, self.name)
    }
}

/// A manager for storing and retrieving users
pub struct UserManager {
    users: HashMap<u32, User>,
}

impl UserManager {
    /// Creates a new empty user manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }

    /// Adds a user to the manager
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    /// Retrieves a user by ID
    pub fn get_user(&self, id: u32) -> Option<&User> {
        self.users.get(&id)
    }

    /// Returns all users
    pub fn list_all(&self) -> Vec<&User> {
        self.users.values().collect()
    }
}

/// Main entry point for the application
fn main() {
    let mut manager = UserManager::new();
    
    let user1 = User::new(1, "Alice".to_string());
    let user2 = User::new(2, "Bob".to_string());
    
    manager.add_user(user1);
    manager.add_user(user2);
    
    for user in manager.list_all() {
        println!("{}", user.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(1, "Test".to_string());
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Test");
    }

    #[test]
    fn test_user_manager() {
        let mut manager = UserManager::new();
        let user = User::new(1, "Test".to_string());
        
        manager.add_user(user.clone());
        assert!(manager.get_user(1).is_some());
        assert_eq!(manager.list_all().len(), 1);
    }
}
