/**
 * Sample TypeScript module demonstrating TS parsing capabilities
 */

import { EventEmitter } from 'events';

/**
 * Interface for user data
 */
export interface UserData {
    id: number;
    name: string;
    email?: string;
}

/**
 * A simple data class for storing user information
 */
export class User implements UserData {
    public id: number;
    public name: string;
    public email: string = '';

    constructor(id: number, name: string) {
        this.id = id;
        this.name = name;
    }

    /**
     * Sets the user's email address
     * @param email - The email to set
     */
    setEmail(email: string): void {
        this.email = email;
    }

    /**
     * Returns a formatted display string
     * @returns Formatted display string
     */
    display(): string {
        return `User ${this.id}: ${this.name}`;
    }
}

/**
 * A manager for storing and retrieving users
 */
export class UserManager extends EventEmitter {
    private users: Map<number, User>;

    constructor() {
        super();
        this.users = new Map();
    }

    /**
     * Adds a user to the manager
     * @param user - The user to add
     */
    addUser(user: User): void {
        this.users.set(user.id, user);
        this.emit('userAdded', user);
    }

    /**
     * Retrieves a user by ID
     * @param id - The user ID
     * @returns The user or undefined
     */
    getUser(id: number): User | undefined {
        return this.users.get(id);
    }

    /**
     * Returns all users
     * @returns Array of all users
     */
    listAll(): User[] {
        return Array.from(this.users.values());
    }

    /**
     * Removes a user by ID
     * @param id - The user ID to remove
     * @returns Whether user was removed
     */
    removeUser(id: number): boolean {
        return this.users.delete(id);
    }
}

/**
 * Main function to demonstrate usage
 */
export function main(): void {
    const manager = new UserManager();
    
    const user1 = new User(1, 'Alice');
    const user2 = new User(2, 'Bob');
    
    manager.addUser(user1);
    manager.addUser(user2);
    
    manager.listAll().forEach(user => {
        console.log(user.display());
    });
}
