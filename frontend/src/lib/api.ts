// src/lib/api.ts
const API_BASE_URL = 'http://127.0.0.1:8000';

/**
 * Makes a request to the API.
 * @param endpoint - The API endpoint to call (e.g., '/users').
 * @param options - Optional request options (e.g., method, body, headers).
 * @returns A promise that resolves with the API response (parsed JSON if applicable, otherwise text, or true for a 302 redirect).
 * @throws Will throw an error if the API request fails or returns an error status.
 */
export async function apiRequest(endpoint: string, options: RequestInit = {}) {
    const url = `${API_BASE_URL}${endpoint}`;

    const defaultOptions: RequestInit = {
        credentials: 'include',
        headers: {
            'Content-Type': 'application/json',
            ...options.headers
        },
        mode: 'cors'
    };

    try {
        const response = await fetch(url, { ...defaultOptions, ...options });

        if (response.status === 302) {
            console.log('Success with redirect');
            return true;
        }

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`API error: ${response.status} ${response.statusText} - ${errorText}`);
            // Include the status code in the error message for better error handling
            throw new Error(`${response.status}: ${errorText || response.statusText}`);
        }

        const contentType = response.headers.get('content-type');
        if (contentType && contentType.includes('application/json')) {
            return response.json();
        }

        return response.text();
    } catch (error) {
        console.error('API request failed:', error);
        throw error;
    }
}
