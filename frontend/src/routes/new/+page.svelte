<script lang="ts">
    import { goto } from '$app/navigation';
    import { createPost } from '$lib/stores/posts';
    import '../../style/mainpage.css'; // Wiederverwendung von Styles von der Hauptseite
    import { onMount } from 'svelte';
    import { checkAuthStatus } from '$lib/stores/auth';

    let heading = '';
    let text = '';
    let isLoading = false;
    let error = '';
    let isCheckingAuth = true;

    onMount(async () => {
        const isAuthenticated = await checkAuthStatus();
        if (!isAuthenticated) {
            goto('/login'); // Weiterleitung zum Login, wenn nicht authentifiziert
        }
        isCheckingAuth = false;
    });

    async function handleSubmit() {
        isLoading = true;
        error = '';

        if (!heading.trim() || !text.trim()) {
            error = 'Titel und Text dürfen nicht leer sein.';
            isLoading = false;
            return;
        }

        try {
            await createPost(heading, text);
            goto('/'); // Zurück zur Hauptseite nach erfolgreichem Erstellen
        } catch (e) {
            error = e instanceof Error ? e.message : 'Ein Fehler ist beim Erstellen des Posts aufgetreten.';
            console.error('Fehler beim Erstellen des Posts:', e);
        } finally {
            isLoading = false;
        }
    }
</script>

<svelte:head>
    <title>Neuen Post erstellen - Rocket-Forum</title>
</svelte:head>

{#if isCheckingAuth}
    <div class="loading-indicator">Authentifizierung wird überprüft...</div>
{:else}
    <div class="main-container">
        <h1>Neuen Post erstellen</h1>

        <form on:submit|preventDefault={handleSubmit} class="new-post-form">
            {#if error}
                <div class="error-message">{error}</div>
            {/if}

            <div class="form-group">
                <label for="heading">Titel</label>
                <input type="text" id="heading" bind:value={heading} required disabled={isLoading} />
            </div>

            <div class="form-group">
                <label for="text">Text</label>
                <textarea id="text" bind:value={text} rows="10" required disabled={isLoading}></textarea>
            </div>

            <button type="submit" class="button primary" disabled={isLoading}>
                {isLoading ? 'Wird erstellt...' : 'Post erstellen'}
            </button>
        </form>
    </div>
{/if}

<style>
    .new-post-form {
        background-color: white;
        padding: 2rem;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        margin-top: 2rem;
    }

    .form-group {
        margin-bottom: 1.5rem;
    }

    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: bold;
        color: #333;
    }

    .form-group input[type='text'],
    .form-group textarea {
        width: 100%;
        padding: 0.75rem;
        border: 1px solid #ccc;
        border-radius: 4px;
        font-size: 1rem;
        box-sizing: border-box;
    }

    .form-group textarea {
        resize: vertical;
        min-height: 150px;
    }

    .button.primary {
        background-color: #007bff; /* Beispiel Primärfarbe */
        color: white;
        padding: 0.75rem 1.5rem;
        border: none;
        border-radius: 4px;
        font-size: 1rem;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .button.primary:hover {
        background-color: #0056b3;
    }

    .button.primary:disabled {
        background-color: #ccc;
        cursor: not-allowed;
    }

    .error-message {
        background-color: #f8d7da;
        color: #721c24;
        padding: 0.75rem;
        border: 1px solid #f5c6cb;
        border-radius: 4px;
        margin-bottom: 1rem;
    }

    .loading-indicator {
        text-align: center;
        margin-top: 3rem;
        font-size: 1.2rem;
        color: #666;
    }
</style>