<!-- Home page for the Rocket-Forum application -->
<script lang="ts">
    import { goto } from '$app/navigation';
    import { checkAuthStatus, logout } from '$lib/stores/auth';
    import { onMount } from 'svelte';

    let isLoading = true;
    let showOverlay = false;
    let isAuthenticated = false;

    onMount(async () => {
        isLoading = true;
        isAuthenticated = await checkAuthStatus();
        isLoading = false;
    });

    function navigateToLogin() {
        goto('/login');
    }

    function handleLogout() {
        logout();
        showOverlay = false;
    }

    function toggleOverlay() {
        showOverlay = !showOverlay;
    }

    function closeOverlay() {
        showOverlay = false;
    }
</script>

<header class="sticky-header">
    <div class="header-content">
        <h1 class="header-title">Rocket-Forum</h1>
        <div class="header-right">
            {#if !isAuthenticated}
                <button class="login-button" on:click={navigateToLogin}>Login</button>
            {:else}
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

{#if isLoading}
    <div class="loading-indicator">Lade...</div>
{:else}
    <div class="welcome-container">
        <div class="welcome-text">
            <p>Deine neue Community-Plattform für Diskussionen und Austausch.</p>
            <p>Melde dich an, um an Gesprächen teilzunehmen oder eigene Themen zu erstellen.</p>
        </div>
    </div>
{/if}

<style>

    .header-right {
        position: absolute;
        right: 1.5rem;
        display: flex;
        align-items: center;
    }

    .login-button {
        background-color: #4caf50;
        color: white;
        border: none;
        padding: 0.5rem 1.2rem;
        font-size: 1rem;
        border-radius: 4px;
        cursor: pointer;
        transition: background-color 0.2s;
    }

    .login-button:hover {
        background-color: #3e9142;
    }

    .login-success.user-icon {
        color: #4caf50;
        font-size: 1.5rem;
        font-weight: bold;
        cursor: pointer;
        border-radius: 10%;
        padding: 0.2rem 0.4rem;
        transition: background 0.2s;
        outline: none;
    }
    .login-success.user-icon:focus,
    .login-success.user-icon:hover {
        background: #e8f5e9;
		outline: none;
    }

    .user-overlay {
        position: absolute;
        top: 2.2rem;
        right: 0;
        background: #fff;
        border: 1px solid #eee;
        border-radius: 8px;
        box-shadow: 0 2px 12px rgba(0,0,0,0.12);
        padding: 1rem 1.2rem;
        z-index: 300;
        /*min-width: 120px;*/
        display: flex;
        flex-direction: column;
        align-items: flex-end;
    }

    .logout-button {
        background: #e53935;
        color: #fff;
        border: none;
        border-radius: 4px;
        padding: 0.5rem 1.2rem;
        font-size: 1rem;
        cursor: pointer;
        transition: background 0.2s;
    }
    .logout-button:hover {
        background: #b71c1c;
    }

    .overlay-backdrop {
        position: fixed;
        top: 0; left: 0; right: 0; bottom: 0;
        background: transparent;
        z-index: 250;
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

    .welcome-text {
        margin-bottom: 2.5rem;
    }

    p {
        color: #666;
        line-height: 1.6;
        font-size: 1.1rem;
        margin-bottom: 1rem;
    }

    .loading-indicator {
        text-align: center;
        margin-top: 3rem;
        font-size: 1.2rem;
        color: #666;
    }

</style>
