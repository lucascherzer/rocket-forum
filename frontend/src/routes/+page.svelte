<!-- Home page for the Rocket-Forum application -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { checkAuthStatus, logout } from '$lib/stores/auth';
	import { onMount } from 'svelte';
	import { fetchLatestPosts, posts } from '$lib/stores/posts';
	import { getHashtagColor, getTextColor } from '$lib/utils/tagColors';
	import { createComment } from '$lib/stores/comments';
	import { fetchCommentById, fetchCommentsByIds } from '$lib/stores/comments'; // fetchCommentsByIds hinzufügen
	import type { Comment, ViewPost } from '$lib/types';
	import '../style/mainpage.css';

	let isLoading = true;
	let showOverlay = false;
	let isAuthenticated = false;
	let postsLoading = true;

	// State for comment text areas and expansion
	let commentTexts: { [postId: string]: string } = {};
	let activeCommentBox: string | null = null;

	// State for like button loading
	let likingPosts: { [postId: string]: boolean } = {};

	// State für geladene Kommentare und deren Sichtbarkeit
	let postComments: { [postId: string]: Comment[] } = {};
	let numVisibleComments: { [postId: string]: number } = {};
	let commentsLoading: { [postId: string]: boolean } = {};

	onMount(async () => {
		try {
			isLoading = true;
			isAuthenticated = await checkAuthStatus();
			if (isAuthenticated) {
				postsLoading = true;
				await fetchLatestPosts();
				// Die initiale Kommentarladung erfolgt jetzt reaktiv auf $posts
			}
		} catch (error) {
			console.error('Error loading page:', error);
		} finally {
			isLoading = false;
			postsLoading = false; // Wird auch hier gesetzt, falls nicht authentifiziert
		}
	});

	// Initialisiert commentTexts, wenn Posts geladen werden
	$: {
		if ($posts && $posts.length > 0) {
			$posts.forEach((post) => {
				if (!(post.id in commentTexts)) {
					commentTexts[post.id] = '';
				}
			});
		}
	}

	// Lädt alle Kommentare für jeden Post, wenn Posts aktualisiert werden
	$: {
		if ($posts && Array.isArray($posts)) {
			loadCommentsForPosts();
		}
	}

	async function handleLikePost(postId: string) {
		if (likingPosts[postId]) return; // Prevent double-clicking

		likingPosts[postId] = true;
		try {
			// Pass the post ID directly (it already contains "Posts:" prefix)
			await likePost(postId);
		} catch (error) {
			console.error('Failed to like post:', error);

			// Check if it's a 403 error (already liked)
			if (error instanceof Error) {
				if (error.message.includes('403')) {
					alert('Du hast diesen Post bereits geliked!');
				} else {
					alert('Fehler beim Liken des Posts. Bitte versuche es erneut.');
				}
			} else {
				alert('Fehler beim Liken des Posts. Bitte versuche es erneut.');
			}
		} finally {
			likingPosts[postId] = false;
		}
	}

	async function loadCommentsForPosts() {
		for (const post of $posts) {
			if (
				post.comments &&
				post.comments.length > 0 &&
				!postComments[post.id] &&
				!commentsLoading[post.id]
			) {
				commentsLoading[post.id] = true;
				commentsLoading = { ...commentsLoading }; // Trigger reactivity

				try {
					console.log(`Loading comments for post ${post.id}:`, post.comments);
					const comments = await fetchCommentsByIds(post.comments);
					console.log(`Loaded ${comments.length} comments for post ${post.id}`);

					// Sortiere Kommentare nach Erstelldatum (neueste zuerst)
					const sortedComments = comments.sort((a, b) => {
						const dateA = new Date(a.created_at);
						const dateB = new Date(b.created_at);
						return dateB.getTime() - dateA.getTime(); // Neueste zuerst
					});

					postComments[post.id] = sortedComments;
					numVisibleComments[post.id] = Math.min(1, sortedComments.length);

					// Trigger reactivity
					postComments = { ...postComments };
					numVisibleComments = { ...numVisibleComments };
				} catch (error) {
					console.error(`Konnte Kommentare für Post ${post.id} nicht laden:`, error);
					postComments[post.id] = [];
					numVisibleComments[post.id] = 0;
					postComments = { ...postComments };
				} finally {
					commentsLoading[post.id] = false;
					commentsLoading = { ...commentsLoading };
				}
			} else if (post.comments && post.comments.length === 0) {
				postComments[post.id] = [];
				numVisibleComments[post.id] = 0;
			}
		}
	}

	async function handleAddComment(postId: string) {
		const text = commentTexts[postId];
		if (!text || text.trim() === '') {
			alert('Kommentar darf nicht leer sein.');
			return;
		}
		try {
			await createComment(postId, text);
			commentTexts[postId] = '';
			activeCommentBox = null;

			// Posts neu laden, um aktualisierte comment_ids und Kommentare zu erhalten
			postsLoading = true;
			// Bestehende Kommentardaten für diesen Post zurücksetzen
			delete postComments[postId];
			delete numVisibleComments[postId];
			await fetchLatestPosts();
		} catch (error) {
			console.error('Fehler beim Hinzufügen des Kommentars:', error);
			alert('Fehler beim Hinzufügen des Kommentars. Bitte versuche es erneut.');
		} finally {
			postsLoading = false;
		}
	}

	async function handleLoadMoreComments(post: ViewPost) {
		if (!postComments[post.id] || !numVisibleComments[post.id] || commentsLoading[post.id]) {
			return;
		}

		const currentlyVisible = numVisibleComments[post.id] || 0;
		const totalLoadedComments = postComments[post.id]?.length || 0;

		// Prüfe, ob es mehr geladene Kommentare gibt, die angezeigt werden können
		if (currentlyVisible < totalLoadedComments) {
			// Zeige 3 weitere Kommentare (statt nur einen)
			numVisibleComments[post.id] = Math.min(currentlyVisible + 3, totalLoadedComments);
			// Trigger reactivity
			numVisibleComments = { ...numVisibleComments };
		}
	}

	function navigateToLogin() {
		goto('/login');
	}

	function handleLogout() {
		logout();
		showOverlay = false;
		isAuthenticated = false;
		// Kommentardaten zurücksetzen beim Logout
		postComments = {};
		numVisibleComments = {};
	}

	function toggleOverlay() {
		showOverlay = !showOverlay;
	}

	function closeOverlay() {
		showOverlay = false;
	}

	function navigateToCreatePost() {
		goto('/new.html'); // Annahme: Dies ist die korrekte Route für neue Posts
	}
</script>

<svelte:head>
	<title>Rocket-Forum</title>
</svelte:head>

<header class="sticky-header">
	<div class="header-content">
		<h1 class="header-title">Rocket-Forum</h1>
		<div class="header-right">
			{#if !isAuthenticated}
				<button class="login-button" on:click={navigateToLogin}>Login</button>
			{:else}
				<button class="login-success user-icon" aria-label="User menu" on:click={toggleOverlay}
					>&#128100;</button
				>
				{#if showOverlay}
					<div
						class="user-overlay"
						role="dialog"
						aria-label="User menu"
						tabindex="0"
						on:click|stopPropagation
						on:keydown={(e) => e.key === 'Escape' && closeOverlay()}
					>
						<button class="logout-button" on:click={handleLogout}>Logout</button>
					</div>
					<button
						class="overlay-backdrop"
						aria-label="Close overlay"
						on:click={closeOverlay}
						on:keydown={(e) => e.key === 'Enter' && closeOverlay()}
					></button>
				{/if}
			{/if}
		</div>
	</div>
</header>

{#if isLoading}
	<span class="loader"></span>
{:else if !isAuthenticated}
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
		{:else if !$posts || $posts.length === 0}
			<p>Keine Posts gefunden. Sei der Erste, der einen Post erstellt!</p>
		{:else}
			<div class="posts-container">
				{#each $posts as post (post.id)}
					<div class="post-card" class:expanded={activeCommentBox === post.id}>
						<h2>{post.heading}</h2>
						<div class="post-author-date">
							Erstellt von {post.author} am {new Date(post.created_at).toLocaleDateString('de-DE')} um {new Date(post.created_at).toLocaleTimeString('de-DE', { hour: '2-digit', minute: '2-digit' })}
						</div>
						<div class="post-text">{post.text}</div>
						<div class="post-hashtags">
							{#each post.hashtags as tag}
								{@const bgColor = getHashtagColor(tag)}
								{@const textColor = getTextColor(bgColor)}
								<span class="hashtag" style="background-color: {bgColor}; color: {textColor}">
									{tag}
								</span>
							{/each}
						</div>
						
						<div class="post-interaction-bar">
							<div class="post-likes">
								<button
									class="like-button"
									on:click={() => handleLikePost(post.id)}
									disabled={likingPosts[post.id]}
									aria-label="Like this post"
								>
									🚀
								</button>
								<span class="like-count">{post.likes}</span>
							</div>
							<div class="comment-count">
                                {post.comments ? post.comments.length : 0} Kommentar{post.comments && post.comments.length !== 1 ? 'e' : ''}
                            </div>
						</div>

						<!-- Kommentar-Eingabebereich -->
						<div class="comment-input-section">
							<textarea
								bind:value={commentTexts[post.id]}
								placeholder="Schreibe einen Kommentar..."
								on:focus={() => (activeCommentBox = post.id)}
								class:expanded={activeCommentBox === post.id}
							></textarea>
							{#if activeCommentBox === post.id}
								<button class="comment-button" on:click={() => handleAddComment(post.id)}
									>Kommentieren</button
								>
							{/if}
						</div>

						<!-- Kommentar-Anzeigebereich -->
						<div class="comments-display-section">
                            {#if commentsLoading[post.id]}
                                <p class="comment-loading">Lade Kommentare...</p>
                            {/if}

                            {#if postComments[post.id] && postComments[post.id].length > 0}
                                {#each postComments[post.id].slice(0, numVisibleComments[post.id] || 0) as comment (comment.id)}
                                    <div class="comment-card">
                                        <p class="comment-author">
                                            {comment.author} am {new Date(comment.created_at).toLocaleDateString('de-DE')} um {new Date(comment.created_at).toLocaleTimeString('de-DE', { hour: '2-digit', minute: '2-digit' })}
                                        </p>
                                        <p class="comment-text">{comment.text}</p>
                                    </div>
                                {/each}
                                {#if postComments[post.id] && (numVisibleComments[post.id] || 0) < postComments[post.id].length}
                                    <button
                                        class="load-more-comments-button"
                                        on:click={() => handleLoadMoreComments(post)}
                                    >
                                        More ({postComments[post.id].length - (numVisibleComments[post.id] || 0)} weitere)
                                    </button>
                                {/if}
                            {:else if !commentsLoading[post.id] && post.comments && post.comments.length > 0 && (!postComments[post.id] || postComments[post.id].length === 0)}
                                <p class="no-comments">
                                    Kommentare konnten nicht geladen werden.
                                    <button
                                        on:click={() => {
                                            // Kommentare erneut laden
                                            delete postComments[post.id];
                                            delete numVisibleComments[post.id];
                                            // Trigger reactive loading
                                            posts.set($posts);
                                        }}
                                        class="load-more-comments-button"
                                    >
                                        Erneut versuchen
                                    </button>
                                </p>
                            {/if}
                        </div>
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}
