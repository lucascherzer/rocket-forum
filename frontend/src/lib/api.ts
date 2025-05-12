const API_BASE_URL = 'http://127.0.0.1:8000';

export async function apiRequest(endpoint: string, options: RequestInit = {}) {
    const url = `${API_BASE_URL}${endpoint}`;

    // Default options for all requests
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

        // Handle successful response with status 302 (Redirect)
        if (response.status === 302) {
            console.log('Success with redirect');
            return true;
        }

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`API error: ${errorText}`);
            throw new Error(errorText || response.statusText);
        }

        // Check if the response is JSON
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
