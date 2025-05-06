<script lang="ts">
	import { goto } from '$app/navigation';
	import { isAuthenticated } from '$lib/stores/auth';
	import { onMount } from 'svelte';

	// Beim Laden der Seite den Auth-Status überprüfen
	import { checkAuthStatus } from '$lib/stores/auth';

	let isLoading = true;

	onMount(async () => {
		console.log('Startseite geladen, überprüfe Auth-Status...');
		isLoading = true;
		// Bei jedem Laden der Seite den Auth-Status neu überprüfen
		const authStatus = await checkAuthStatus();
		console.log('Auth-Status auf Startseite:', authStatus, 'isAuthenticated:', $isAuthenticated);
		isLoading = false;
	});

	function navigateToLogin() {
		goto('/login');
	}

	function navigateToForum() {
		goto('/forum');
	}
</script>

{#if isLoading}
	<div class="loading-indicator">Lade...</div>
{:else}
	{#if $isAuthenticated}
		<div class="logged-in-banner">
			<span class="logged-in-text">✓ Logged in</span>
		</div>
	{/if}

	<div class="welcome-container">
		<h1>Willkommen im Rocket-Forum</h1>

		<div class="welcome-text">
			<p>Deine neue Community-Plattform für Diskussionen und Austausch.</p>
			<p>Melde dich an, um an Gesprächen teilzunehmen oder eigene Themen zu erstellen.</p>
		</div>

		<div class="button-container">
			{#if !$isAuthenticated}
				<button class="login-button" on:click={navigateToLogin}> Zum Login </button>
			{:else}
				<button class="forum-button" on:click={navigateToForum}> Zum Forum </button>
			{/if}
		</div>
	</div>
{/if}

<style>
	.logged-in-banner {
		background-color: rgba(76, 175, 80, 0.1);
		padding: 0.5rem;
		text-align: center;
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		z-index: 100;
	}

	.logged-in-text {
		color: #4caf50;
		font-weight: bold;
	}

	.welcome-container {
		max-width: 800px;
		margin: 4rem auto;
		text-align: center;
		padding: 2rem;
		border-radius: 8px;
		background-color: white;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}

	h1 {
		color: #333;
		margin-bottom: 1.5rem;
		font-size: 2.2rem;
	}

	.welcome-text {
		margin-bottom: 2.5rem;
	}

	p {
		color: #666;
		line-height: 1.6;
		font-size: 1.1rem;
		margin-bottom: 1rem;
	}

	.button-container {
		margin-top: 2rem;
	}

	.login-button,
	.forum-button {
		background-color: #4caf50;
		color: white;
		border: none;
		padding: 0.8rem 2rem;
		font-size: 1.1rem;
		border-radius: 4px;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.login-button:hover,
	.forum-button:hover {
		background-color: #3e9142;
	}

	/* Extra Abstand, wenn das Banner angezeigt wird */
	:global(body) {
		margin-top: 2.5rem;
	}
</style>
