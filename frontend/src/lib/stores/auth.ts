// src/lib/stores/auth.ts
import { writable } from 'svelte/store';
import { apiRequest } from '$lib/api';

export const currentUser = writable(null);

/**
 * Checks the current authentication status by making a request to the backend.
 * Updates the currentUser store if the user is not authenticated.
 * @returns A promise that resolves to true if the user is authenticated, false otherwise.
 */
export async function checkAuthStatus(): Promise<boolean> {
    try {
        await apiRequest('/api/auth/check');	
        return true;
    } catch (error) {
        console.error('Auth check failed:', error);
        currentUser.set(null);
        return false;
    }
}

/**
 * Attempts to log in a user with the provided username and password.
 * @param username - The user's username.
 * @param password - The user's password.
 * @returns A promise that resolves to true if login is successful, false otherwise.
 */
export async function login(username: string, password: string) {
    try {
        await apiRequest('/api/auth/login', {
            method: 'POST',
            body: JSON.stringify({ username, password })
        });
        return true;

    } catch (error) {
        console.error('Login failed:', error);
        return false;
    }
}

/**
 * Attempts to sign up a new user with the provided username and password.
 * @param username - The desired username for the new account.
 * @param password - The desired password for the new account.
 * @returns A promise that resolves to true if signup is successful, false otherwise.
 */
export async function signup(username: string, password: string) {
    try {
        await apiRequest('/api/auth/signup', {
            method: 'POST',
            body: JSON.stringify({ username, password })
        });
        return true;
    } catch (error) {
        console.error('Signup failed:', error);
        return false;
    }
}

/**
 * Logs out the current user.
 * Updates the currentUser store to null on successful logout.
 * @returns A promise that resolves to true if logout is successful, false otherwise.
 */
export async function logout() {
    try {
        await apiRequest('/api/auth/logout');
        currentUser.set(null);
        return true;
    } catch (error) {
        console.error('Logout failed:', error);
        return false;
    }
}
