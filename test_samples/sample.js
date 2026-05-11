/**
 * Sample JavaScript module demonstrating JS parsing capabilities
 */

const EventEmitter = require('events');

/**
 * A simple data class for storing user information
 */
class User {
    constructor(id, name) {
        this.id = id;
        this.name = name;
        this.email = '';
    }

    /**
     * Sets the user's email address
     * @param {string} email - The email to set
     */
    setEmail(email) {
        this.email = email;
    }

    /**
     * Returns a formatted display string
     * @returns {string} Display string
     */
    display() {
        return `User ${this.id}: ${this.name}`;
    }
}

/**
 * A manager for storing and retrieving users
 */
class UserManager extends EventEmitter {
    constructor() {
        super();
        this.users = new Map();
    }

    /**
     * Adds a user to the manager
     * @param {User} user - The user to add
     */
    addUser(user) {
        this.users.set(user.id, user);
        this.emit('userAdded', user);
    }

    /**
     * Retrieves a user by ID
     * @param {number} id - The user ID
     * @returns {User|undefined} The user or undefined
     */
    getUser(id) {
        return this.users.get(id);
    }

    /**
     * Returns all users
     * @returns {Array<User>} Array of all users
     */
    listAll() {
        return Array.from(this.users.values());
    }

    /**
     * Removes a user by ID
     * @param {number} id - The user ID to remove
     * @returns {boolean} Whether user was removed
     */
    removeUser(id) {
        return this.users.delete(id);
    }
}

/**
 * Main function to demonstrate usage
 */
function main() {
    const manager = new UserManager();
    
    const user1 = new User(1, 'Alice');
    const user2 = new User(2, 'Bob');
    
    manager.addUser(user1);
    manager.addUser(user2);
    
    manager.listAll().forEach(user => {
        console.log(user.display());
    });
}

module.exports = { User, UserManager, main };
