<script lang="ts">
	import { signup } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	let username = '';
	let password = '';
	let confirmPassword = '';
	let loading = false;
	let error = '';

	async function handleSubmit() {
		loading = true;
		error = '';

		// Überprüfe, ob Passwörter übereinstimmen
		if (password !== confirmPassword) {
			error = 'Die Passwörter stimmen nicht überein';
			loading = false;
			return;
		}

		try {
			const success = await signup(username, password);
			if (success) {
				// Wenn die Registrierung erfolgreich ist, zur Login-Seite weiterleiten
				goto('/');
			} else {
				error = 'Registrierung fehlgeschlagen';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ein Fehler ist aufgetreten';
		} finally {
			loading = false;
		}
	}
</script>

<header class="sticky-header">
	<div class="header-content">
		<h1 class="header-title">Registrieren</h1>
	</div>
</header>

<div class="login-container">
	<form on:submit|preventDefault={handleSubmit}>
		{#if error}
			<div class="error">{error}</div>
		{/if}

		<div>
			<label for="username">Benutzername</label>
			<input id="username" type="text" bind:value={username} required />
		</div>

		<div>
			<label for="password">Passwort</label>
			<input id="password" type="password" bind:value={password} required />
		</div>

		<div>
			<label for="confirmPassword">Passwort bestätigen</label>
			<input id="confirmPassword" type="password" bind:value={confirmPassword} required />
		</div>

		<button type="submit" disabled={loading}>
			{loading ? 'Bitte warten...' : 'Registrieren'}
		</button>

		<p>Bereits registriert? <a href="/login">Anmelden</a></p>
	</form>
</div>

<style>
	form {
		max-width: 400px;
		margin: 0 auto;
	}

	.error {
		color: red;
		margin-bottom: 1rem;
	}

	label {
		display: block;
		margin-bottom: 0.5rem;
	}

	input {
		width: 100%;
		padding: 0.5rem;
		margin-bottom: 1rem;
	}

	button {
		width: 100%;
		padding: 0.5rem;
		background: #4caf50;
		color: white;
		border: none;
		cursor: pointer;
	}

	button:disabled {
		background: #cccccc;
	}
</style>
