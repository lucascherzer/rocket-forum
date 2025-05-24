<script lang="ts">
	import { signup } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import '../../style/signup.css';

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
				goto('/login');
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

<svelte:head>
    <title>Neuen Account erstellen</title>
</svelte:head>

<div class="login-page-wrapper">
	<div class="login-container">
		<h1 class="login-title">Registrieren</h1>
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
</div>
