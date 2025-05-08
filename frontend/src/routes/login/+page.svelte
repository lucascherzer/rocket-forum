<!-- src/routes/login/+page.svelte -->
<script lang="ts">
    import { login, isAuthenticated } from '$lib/stores/auth';
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

    function goToIndex() {
        goto('/');
    }
</script>

<header class="sticky-header">
    <div class="header-content">
        <h1 class="header-title">Login</h1>
    </div>
</header>

{#if $isAuthenticated}
    <div class="already-logged-in-box">
        <p>Du bist bereits eingeloggt.</p>
        <button on:click={goToIndex}>Zur Startseite</button>
    </div>
{:else}
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
{/if}

<style>
    .sticky-header {
        position: sticky;
        top: 0;
        width: 100%;
        background: #f36906;
        box-shadow: 0 2px 8px rgba(0,0,0,0.06);
        z-index: 200;
        border-radius: 8px;
        margin-bottom: 2rem;
    }

    .header-content {
        display: flex;
        align-items: center;
        justify-content: center;
        max-width: 900px;
        margin: 0 auto;
        padding: 0.7rem 1.5rem;
        position: relative;
    }

    .header-title {
        margin: 0 auto;
        font-size: 1.7rem;
        font-weight: 700;
        color: #333;
        letter-spacing: 1px;
    }

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

    body, input, button, textarea,
    .header-title, label, .error, .already-logged-in-box, a, p, h1, h2, h3, h4, h5, h6, div {
        font-family: system-ui, sans-serif;
    }

</style>
