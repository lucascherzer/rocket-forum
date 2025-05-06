<script lang="ts">
	import { isAuthenticated, checkAuthStatus } from '$lib/stores/auth';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let showSuccess = false;
	let isChecking = true;

	onMount(async () => {
		isChecking = true;

		// Aktiven Authentifizierungsstatus direkt von der API überprüfen
		const isAuth = await checkAuthStatus();

		if (!isAuth) {
			// Wenn nicht authentifiziert, zurück zur Login-Seite
			goto('/');
			return;
		}

		isChecking = false;
		showSuccess = true;
	});
</script>

<div class="success-container">
	{#if isChecking}
		<div class="loading">Authentifizierung wird überprüft...</div>
	{:else if showSuccess}
		<div class="success-message">
			<h1>Erfolgreich angemeldet!</h1>
			<div class="success-icon">✓</div>
			<p>Sie sind jetzt eingeloggt und können das Forum nutzen.</p>
			<div class="button-group">
				<a href="/forum" class="button primary">Zum Forum</a>
				<a href="/profile" class="button secondary">Mein Profil</a>
				<button class="button logout" on:click={() => goto('/api/auth/logout')}>Abmelden</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.success-container {
		display: flex;
		justify-content: center;
		align-items: center;
		min-height: 80vh;
		padding: 2rem;
	}

	.loading {
		font-size: 1.2rem;
		color: #666;
	}

	.success-message {
		background-color: white;
		border-radius: 8px;
		padding: 2rem;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
		text-align: center;
		max-width: 500px;
		width: 100%;
	}

	.success-icon {
		font-size: 5rem;
		color: #4caf50;
		margin: 1rem 0;
	}

	h1 {
		color: #4caf50;
		margin-bottom: 1rem;
	}

	p {
		font-size: 1.1rem;
		margin-bottom: 2rem;
		color: #666;
	}

	.button-group {
		display: flex;
		flex-direction: column;
		gap: 0.8rem;
	}

	.button {
		padding: 0.75rem 1.5rem;
		border-radius: 4px;
		text-decoration: none;
		font-weight: bold;
		cursor: pointer;
		transition: background-color 0.2s;
		border: none;
		font-size: 1rem;
	}

	.primary {
		background-color: #4caf50;
		color: white;
	}

	.primary:hover {
		background-color: #3e9142;
	}

	.secondary {
		background-color: #f1f1f1;
		color: #333;
	}

	.secondary:hover {
		background-color: #e1e1e1;
	}

	.logout {
		background-color: transparent;
		color: #ff5252;
		border: 1px solid #ff5252;
	}

	.logout:hover {
		background-color: #fff0f0;
	}
</style>
