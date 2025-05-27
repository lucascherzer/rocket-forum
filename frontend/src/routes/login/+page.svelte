<!-- src/routes/login/+page.svelte -->
<script lang="ts">
	import { login, checkAuthStatus } from '$lib/stores/auth';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import '../../style/login.css';

	let username = '';
	let password = '';
	let loading = false;
	let error = '';
	let authChecked = false;
	let isLoggedIn = false;

	onMount(async () => {
		isLoggedIn = await checkAuthStatus();
		authChecked = true;
	});

	async function handleSubmit() {
		loading = true;
		error = '';

		try {
			const success = await login(username, password);
			if (success) {
				setTimeout(() => {
					goto('/');
				}, 100);
			} else {
				error = 'Login failed';
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'An error occurred';
		} finally {
			loading = false;
		}
	}

	function goToIndex() {
		goto('/index.html');
	}
</script>

<svelte:head>
    <title>Login</title>
</svelte:head>

<div class="login-page-wrapper">
	{#if !authChecked}
		<span class="loader"></span>
	{:else if isLoggedIn}
		<div class="already-logged-in-box">
			<p>You are already logged in.</p>
			<button on:click={goToIndex}>Go to Home</button>
		</div>
	{:else}
		<div class="login-container">
			<h1 class="login-title">Login</h1>
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
		</div>
	{/if}
</div>
