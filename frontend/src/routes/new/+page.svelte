<script lang="ts">
    import { goto } from '$app/navigation';
    import { createPost } from '$lib/stores/posts';
    import { onMount } from 'svelte';
    import { checkAuthStatus, logout } from '$lib/stores/auth'; 
    import '../../style/new.css';
    import '../../style/app.css';

    let heading = '';
    let text = '';
    let isLoading = false;
    let error = '';
    let isCheckingAuth = true;
    let showOverlay = false;

    onMount(async () => {
        const isAuthenticated = await checkAuthStatus();
        if (!isAuthenticated) {
            goto('/login');
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
            goto('/');
        } catch (e) {
            error = e instanceof Error ? e.message : 'Ein Fehler ist beim Erstellen des Posts aufgetreten.';
            console.error('Fehler beim Erstellen des Posts:', e);
        } finally {
            isLoading = false;
        }
    }

    function handleLogout() {
        logout();
        showOverlay = false;
        goto('/');
    }

    function toggleOverlay() {
        showOverlay = !showOverlay;
    }

    function closeOverlay() {
        showOverlay = false;
    }

    function goBack() {
        goto('/');
    }
</script>

<svelte:head>
    <title>Neuen Post erstellen</title>
</svelte:head>

<header class="sticky-header">
    <div class="header-content">
        <a href="/" class="header-title">Rocket-Forum</a>
        <div class="header-right">
            {#if !isCheckingAuth}
                <button class="login-success user-icon" aria-label="User menu" on:click={toggleOverlay}>&#128100;</button>
                {#if showOverlay}
                    <div class="user-overlay" role="dialog" aria-label="User menu" tabindex="0" on:click|stopPropagation on:keydown={(e) => e.key === 'Escape' && closeOverlay()}>
                        <button class="logout-button" on:click={handleLogout}>Logout</button>
                    </div>
                    <button class="overlay-backdrop" aria-label="Close overlay" on:click={closeOverlay} on:keydown={(e) => e.key === 'Enter' && closeOverlay()}></button>
                {/if}
            {/if}
        </div>
    </div>
</header>

{#if isCheckingAuth}
    <div class="loading-indicator">
        <span class="loader"></span>
    </div>
{:else}
    <div class="new-page-wrapper">
        <div class="new-content-container">
            <div class="create-post-container">
                <button class="back-button" on:click={goBack}>
                    ← Zurück
                </button>
                <h1>Neuen Post erstellen</h1>
                <form on:submit|preventDefault={handleSubmit} class="new-post-form">
                    {#if error}
                        <div class="error-message">{error}</div>
                    {/if}
                    <div class="form-group">
                        <label for="heading">Titel</label>
                        <input type="text" id="heading" maxlength="1000" bind:value={heading} required disabled={isLoading} />
                    </div>
                    <div class="form-group">
                        <label for="text">Text</label>
                        <textarea id="text" maxlength="10000" bind:value={text} rows="10" required disabled={isLoading}></textarea>
                    </div>
                    <div class="button-row">
                        <button type="submit" class="button primary" disabled={isLoading}>
                            {isLoading ? 'Wird erstellt...' : 'Post erstellen'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}