<!-- src/routes/login/+page.svelte -->
<script lang="ts">
    import { login, checkAuthStatus } from '$lib/stores/auth';
    import { goto } from '$app/navigation';
    import { onMount } from 'svelte';

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
        goto('/');
    }
</script>

{#if !authChecked}
    <div>Loading...</div>
{:else if isLoggedIn}
    <div class="already-logged-in-box">
        <p>You are already logged in.</p>
        <button on:click={goToIndex}>Go to Home</button>
    </div>
{:else}
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

            <button type="submit" disabled={loading}>
                {loading ? 'Bitte warten...' : 'Anmelden'}
            </button>

            <p>Noch kein Konto? <a href="/signup">Registrieren</a></p>
        </form>
    </div>
{/if}

<style>

    form {
        max-width: 400px;
        margin: 0 auto;
    }

    .already-logged-in-box {
        max-width: 400px;
        margin: 2rem auto;
        padding: 2rem;
        background: #e8f5e9;
        border: 1px solid #b2dfdb;
        border-radius: 8px;
        text-align: center;
    }

    .already-logged-in-box button {
        margin-top: 1rem;
        padding: 0.5rem 1.2rem;
        background: #4caf50;
        color: white;
        border: none;
        border-radius: 4px;
        cursor: pointer;
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
        width: 95%;
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
