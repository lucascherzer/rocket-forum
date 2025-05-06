<!-- src/routes/login/+page.svelte -->
<script lang="ts">
	import { login } from '$lib/stores/auth';
	import { goto } from '$app/navigation';

	let username = '';
	let password = '';
	let loading = false;
	let error = '';

	async function handleSubmit() {
		loading = true;
		error = '';

		try {
			console.log('Login-Versuch für:', username);
			const success = await login(username, password);
			console.log('Login-Ergebnis:', success);

			if (success) {
				console.log('Weiterleitung zur Startseite nach erfolgreichem Login');
				// Kurze Verzögerung hinzufügen, um sicherzustellen, dass der Status gesetzt wird
				setTimeout(() => {
					goto('/');
				}, 100);
			} else {
				error = 'Login fehlgeschlagen';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Ein Fehler ist aufgetreten';
		} finally {
			loading = false;
		}
	}
</script>

<h1>Login</h1>

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

	<button type="submit" disabled={loading}>
		{loading ? 'Bitte warten...' : 'Anmelden'}
	</button>

	<p>Noch kein Konto? <a href="/signup">Registrieren</a></p>
</form>

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
