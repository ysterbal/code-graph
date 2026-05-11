/**
 * Sample C module demonstrating C parsing capabilities
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/**
 * User structure
 */
typedef struct {
    int id;
    char name[50];
    char email[100];
} User;

/**
 * Creates a new user with the given ID and name
 * @param id The user ID
 * @param name The user name
 * @return Pointer to new user or NULL on failure
 */
User* user_new(int id, const char* name) {
    User* user = (User*)malloc(sizeof(User));
    if (!user) return NULL;
    
    user->id = id;
    strncpy(user->name, name, sizeof(user->name) - 1);
    user->name[sizeof(user->name) - 1] = '\0';
    user->email[0] = '\0';
    
    return user;
}

/**
 * Sets the user's email address
 * @param user Pointer to user
 * @param email The email to set
 */
void user_set_email(User* user, const char* email) {
    if (user && email) {
        strncpy(user->email, email, sizeof(user->email) - 1);
        user->email[sizeof(user->email) - 1] = '\0';
    }
}

/**
 * Returns a formatted display string
 * @param user Pointer to user
 * @return Formatted display string (caller must free)
 */
char* user_display(const User* user) {
    if (!user) return NULL;
    
    char* buffer = (char*)malloc(128);
    if (!buffer) return NULL;
    
    snprintf(buffer, 128, "User %d: %s", user->id, user->name);
    return buffer;
}

/**
 * Frees a user structure
 * @param user Pointer to user to free
 */
void user_free(User* user) {
    if (user) {
        free(user);
    }
}

/**
 * User manager structure
 */
typedef struct {
    User** users;
    int count;
    int capacity;
} UserManager;

/**
 * Creates a new empty user manager
 * @return Pointer to new manager or NULL on failure
 */
UserManager* user_manager_new() {
    UserManager* manager = (UserManager*)malloc(sizeof(UserManager));
    if (!manager) return NULL;
    
    manager->users = NULL;
    manager->count = 0;
    manager->capacity = 0;
    
    return manager;
}

/**
 * Adds a user to the manager
 * @param manager Pointer to manager
 * @param user Pointer to user to add
 */
void user_manager_add(UserManager* manager, User* user) {
    if (!manager || !user) return;
    
    if (manager->count >= manager->capacity) {
        int new_capacity = manager->capacity == 0 ? 4 : manager->capacity * 2;
        User** new_users = (User**)realloc(manager->users, new_capacity * sizeof(User*));
        if (!new_users) return;
        
        manager->users = new_users;
        manager->capacity = new_capacity;
    }
    
    manager->users[manager->count++] = user;
}

/**
 * Retrieves a user by ID
 * @param manager Pointer to manager
 * @param id The user ID
 * @return Pointer to user or NULL if not found
 */
User* user_manager_get(UserManager* manager, int id) {
    if (!manager) return NULL;
    
    for (int i = 0; i < manager->count; i++) {
        if (manager->users[i]->id == id) {
            return manager->users[i];
        }
    }
    
    return NULL;
}

/**
 * Returns all users
 * @param manager Pointer to manager
 * @return Array of users (caller must not free)
 */
User** user_manager_list_all(UserManager* manager) {
    if (!manager) return NULL;
    return manager->users;
}

/**
 * Frees a user manager
 * @param manager Pointer to manager to free
 */
void user_manager_free(UserManager* manager) {
    if (manager) {
        if (manager->users) {
            free(manager->users);
        }
        free(manager);
    }
}

/**
 * Main entry point
 */
int main() {
    UserManager* manager = user_manager_new();
    
    User* user1 = user_new(1, "Alice");
    User* user2 = user_new(2, "Bob");
    
    user_manager_add(manager, user1);
    user_manager_add(manager, user2);
    
    User** users = user_manager_list_all(manager);
    for (int i = 0; i < manager->count; i++) {
        char* display = user_display(users[i]);
        if (display) {
            printf("%s\n", display);
            free(display);
        }
    }
    
    user_manager_free(manager);
    return 0;
}
