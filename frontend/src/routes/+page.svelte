<!-- Home page for the Rocket-Forum application -->
<script lang="ts">
    import { goto } from '$app/navigation';
    import { checkAuthStatus, logout } from '$lib/stores/auth';
    import { onMount } from 'svelte';
    import '../style/mainpage.css';

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
        isAuthenticated = false;
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

