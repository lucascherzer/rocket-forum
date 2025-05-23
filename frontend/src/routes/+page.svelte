<!-- Home page for the Rocket-Forum application -->
<script lang="ts">
    import { goto } from '$app/navigation';
    import { checkAuthStatus, logout } from '$lib/stores/auth';
    import { onMount } from 'svelte';
    import { fetchLatestPosts } from '$lib/stores/posts'; 
    import { posts } from '$lib/stores/posts';
    import { getHashtagColor, getTextColor } from '$lib/utils/tagColors';
    import '../style/mainpage.css';

    let isLoading = true;
    let showOverlay = false;
    let isAuthenticated = false;
    let postsLoading = true;

    onMount(async () => {
        try {
            isLoading = true;
            // Check authentication status
            isAuthenticated = await checkAuthStatus();
            
            if (isAuthenticated) {
                // Only load posts if user is authenticated
                postsLoading = true;
                await fetchLatestPosts();
            }
        } catch (error) {
            console.error("Error loading page:", error);
        } finally {
            isLoading = false;
            postsLoading = false;
        }
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

    function navigateToCreatePost() {
        goto('/new');
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
    <span class="loader"></span>
{:else}
    {#if !isAuthenticated}
        <div class="welcome-container">
            <div class="welcome-text">
                <p>Deine neue Community-Plattform für Diskussionen und Austausch.</p>
                <p>Melde dich an, um an Gesprächen teilzunehmen oder eigene Themen zu erstellen.</p>
            </div>
        </div>
    {:else}
        <div class="main-container">
            <div class="forum-header-container">
                <h1>Willkommen im Forum</h1>
                <button class="create-post-button" on:click={navigateToCreatePost}>Create Post</button>
            </div>
            
            {#if postsLoading}
                <span class="loader"></span>
            {:else if $posts.length === 0}
                <p>Keine Posts gefunden. Sei der Erste, der einen Post erstellt!</p>
            {:else}
                <div class="posts-container">
                    {#each $posts as post (post.id)}
                        <div class="post-card">
                            <h2>{post.heading}</h2>
                            <p>{post.text}</p>
                            <div class="post-footer">
                                <div class="post-date">{new Date(post.created_at).toLocaleString()}</div>
                                <div class="post-hashtags">
                                    {#each post.hashtags as tag}
                                        {@const bgColor = getHashtagColor(tag)}
                                        {@const textColor = getTextColor(bgColor)}
                                        <span class="hashtag" style="background-color: {bgColor}; color: {textColor}">
                                            {tag}
                                        </span>
                                    {/each}
                                </div>
                            </div>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    {/if}
{/if}


