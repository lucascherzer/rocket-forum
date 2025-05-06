const API_BASE_URL = 'http://127.0.0.1:8000'; // Wichtig: Hier 127.0.0.1 statt localhost verwenden

export async function apiRequest(endpoint: string, options: RequestInit = {}) {
	const url = `${API_BASE_URL}${endpoint}`;

	// Standard-Optionen für alle Anfragen
	const defaultOptions: RequestInit = {
		credentials: 'include', // Wichtig für Cookies/Sessions
		headers: {
			'Content-Type': 'application/json',
			...options.headers
		},
		mode: 'cors' // Explizit CORS-Modus festlegen
	};

	try {
		const response = await fetch(url, { ...defaultOptions, ...options });

		// Erfolgreiche Antwort mit Status 302 (Redirect) behandeln
		if (response.status === 302) {
			console.log('Erfolgreich mit Redirect');
			return true;
		}

		if (!response.ok) {
			const errorText = await response.text();
			console.error(`API-Fehler: ${errorText}`);
			throw new Error(errorText || response.statusText);
		}

		// Überprüfe, ob die Antwort JSON ist
		const contentType = response.headers.get('content-type');
		if (contentType && contentType.includes('application/json')) {
			return response.json();
		}

		return response.text();
	} catch (error) {
		console.error('API-Anfrage fehlgeschlagen:', error);
		throw error;
	}
}
