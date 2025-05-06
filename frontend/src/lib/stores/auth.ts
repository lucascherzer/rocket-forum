// src/lib/stores/auth.ts
import { writable } from 'svelte/store';
import { apiRequest } from '$lib/api';

export const currentUser = writable(null);
export const isAuthenticated = writable(false);

// Initialisiere Auth-Status beim Laden
export async function checkAuthStatus(): Promise<boolean> {
	try {
		await apiRequest('/api/auth/check');
		isAuthenticated.set(true);
		return true;
	} catch (error) {
		console.error('Auth check failed:', error);
		isAuthenticated.set(false);
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
		// Nach dem Login explizit den Status erneut prüfen
		// const authStatus = await checkAuthStatus();

		// Nur wenn der Auth-Check erfolgreich war, als angemeldet betrachten
		// 	if (authStatus) {
		// 		console.log('Login erfolgreich und Auth-Status verifiziert');
		// 		isAuthenticated.set(true);
		// 		return true;
		// 	} else {
		// 		console.error('Login schien erfolgreich, aber Auth-Check fehlgeschlagen');
		// 		isAuthenticated.set(false);
		// 		return false;
		// 	}
	} catch (error) {
		console.error('Login failed:', error);
		isAuthenticated.set(false);
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
		isAuthenticated.set(false);
		currentUser.set(null);
		return true;
	} catch (error) {
		console.error('Logout failed:', error);
		return false;
	}
}
