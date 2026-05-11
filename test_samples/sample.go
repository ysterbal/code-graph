package main

import (
	"fmt"
	"sync"
)

// User represents a user in the system
type User struct {
	ID    int
	Name  string
	Email string
}

// NewUser creates a new user with the given ID and name
func NewUser(id int, name string) *User {
	return &User{
		ID:    id,
		Name:  name,
		Email: "",
	}
}

// SetEmail sets the user's email address
func (u *User) SetEmail(email string) {
	u.Email = email
}

// Display returns a formatted display string
func (u *User) Display() string {
	return fmt.Sprintf("User %d: %s", u.ID, u.Name)
}

// UserManager manages a collection of users
type UserManager struct {
	mu    sync.Mutex
	users map[int]*User
}

// NewUserManager creates a new empty user manager
func NewUserManager() *UserManager {
	return &UserManager{
		users: make(map[int]*User),
	}
}

// AddUser adds a user to the manager
func (m *UserManager) AddUser(user *User) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.users[user.ID] = user
}

// GetUser retrieves a user by ID
func (m *UserManager) GetUser(id int) *User {
	m.mu.Lock()
	defer m.mu.Unlock()
	return m.users[id]
}

// ListAll returns all users
func (m *UserManager) ListAll() []*User {
	m.mu.Lock()
	defer m.mu.Unlock()
	
	users := make([]*User, 0, len(m.users))
	for _, user := range m.users {
		users = append(users, user)
	}
	return users
}

// RemoveUser removes a user by ID
func (m *UserManager) RemoveUser(id int) bool {
	m.mu.Lock()
	defer m.mu.Unlock()
	
	if _, exists := m.users[id]; exists {
		delete(m.users, id)
		return true
	}
	return false
}

// main is the entry point of the application
func main() {
	manager := NewUserManager()
	
	user1 := NewUser(1, "Alice")
	user2 := NewUser(2, "Bob")
	
	manager.AddUser(user1)
	manager.AddUser(user2)
	
	for _, user := range manager.ListAll() {
		fmt.Println(user.Display())
	}
}
