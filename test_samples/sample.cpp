/**
 * Sample C++ module demonstrating C++ parsing capabilities
 */

#include <iostream>
#include <vector>
#include <memory>
#include <string>

/**
 * User class representing a user in the system
 */
class User {
private:
    int id_;
    std::string name_;
    std::string email_;

public:
    /**
     * Creates a new user with the given ID and name
     * @param id The user ID
     * @param name The user name
     */
    User(int id, const std::string& name) 
        : id_(id), name_(name), email_("") {}

    /**
     * Sets the user's email address
     * @param email The email to set
     */
    void setEmail(const std::string& email) {
        email_ = email;
    }

    /**
     * Returns a formatted display string
     * @return Formatted display string
     */
    std::string display() const {
        return "User " + std::to_string(id_) + ": " + name_;
    }

    /**
     * Gets the user ID
     * @return The user ID
     */
    int getId() const {
        return id_;
    }

    /**
     * Gets the user name
     * @return The user name
     */
    std::string getName() const {
        return name_;
    }
};

/**
 * UserManager class for managing a collection of users
 */
class UserManager {
private:
    std::vector<std::unique_ptr<User>> users_;

public:
    /**
     * Default constructor
     */
    UserManager() = default;

    /**
     * Adds a user to the manager
     * @param user The user to add (will be moved)
     */
    void addUser(std::unique_ptr<User> user) {
        users_.push_back(std::move(user));
    }

    /**
     * Retrieves a user by ID
     * @param id The user ID
     * @return Pointer to user or nullptr if not found
     */
    User* getUser(int id) {
        for (auto& user : users_) {
            if (user->getId() == id) {
                return user.get();
            }
        }
        return nullptr;
    }

    /**
     * Returns all users
     * @return Const reference to vector of users
     */
    const std::vector<std::unique_ptr<User>>& listAll() const {
        return users_;
    }

    /**
     * Removes a user by ID
     * @param id The user ID to remove
     * @return true if user was removed, false otherwise
     */
    bool removeUser(int id) {
        auto it = std::remove_if(users_.begin(), users_.end(),
            [id](const std::unique_ptr<User>& user) {
                return user->getId() == id;
            });
        
        bool removed = (it != users_.end());
        users_.erase(it, users_.end());
        return removed;
    }
};

/**
 * Main function demonstrating usage
 */
int main() {
    UserManager manager;
    
    auto user1 = std::make_unique<User>(1, "Alice");
    auto user2 = std::make_unique<User>(2, "Bob");
    
    manager.addUser(std::move(user1));
    manager.addUser(std::move(user2));
    
    for (const auto& user : manager.listAll()) {
        std::cout << user->display() << std::endl;
    }
    
    return 0;
}
