// src/lib/stores/auth.ts
import { writable } from 'svelte/store';
import { apiRequest } from '$lib/api';

export const currentUser = writable(null);

// Initialize auth status on load
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
