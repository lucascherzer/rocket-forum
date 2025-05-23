<script lang="ts">
    import { goto } from '$app/navigation';
    import { createPost } from '$lib/stores/posts';
    import '../../style/mainpage.css'; // Wiederverwendung von Styles von der Hauptseite
    import { onMount } from 'svelte';
    import { checkAuthStatus } from '$lib/stores/auth';
    import "../style/new.css";

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