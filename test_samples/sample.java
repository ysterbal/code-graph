package com.example;

import java.util.List;
import java.util.ArrayList;
import java.util.Optional;

/**
 * A sample service class demonstrating Java parsing capabilities.
 */
public class UserService {
    private List<String> users;
    private int userIdCounter;

    public UserService() {
        this.users = new ArrayList<>();
        this.userIdCounter = 0;
    }

    /**
     * Adds a user to the service.
     * 
     * @param name The name of the user
     * @return The assigned user ID
     */
    public int addUser(String name) {
        validateName(name);
        users.add(name);
        return userIdCounter++;
    }

    /**
     * Retrieves a user by their index.
     * 
     * @param index The user index
     * @return Optional containing the user name if found
     */
    public Optional<String> getUser(int index) {
        if (index >= 0 && index < users.size()) {
            return Optional.of(users.get(index));
        }
        return Optional.empty();
    }

    /**
     * Gets all users in the service.
     * 
     * @return Unmodifiable list of all users
     */
    public List<String> getAllUsers() {
        return List.copyOf(users);
    }

    /**
     * Validates that a name is not null or empty.
     * 
     * @param name The name to validate
     * @throws IllegalArgumentException if name is invalid
     */
    private void validateName(String name) {
        if (name == null || name.trim().isEmpty()) {
            throw new IllegalArgumentException("Name cannot be null or empty");
        }
    }

    /**
     * Removes a user from the service.
     * 
     * @param index The user index to remove
     * @return true if user was removed, false otherwise
     */
    public boolean removeUser(int index) {
        if (index >= 0 && index < users.size()) {
            users.remove(index);
            return true;
        }
        return false;
    }
}
