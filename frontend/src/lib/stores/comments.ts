import { writable } from 'svelte/store'; // Falls noch nicht vorhanden
import { apiRequest } from '$lib/api';
import type { Comment } from '$lib/types'; // Importiere den Comment-Typ

/**
 * Interface for the data returned when a comment is successfully created.
 */
export interface CommentId {
	id: string;
}

/**
 * Creates a new comment for a given post.
 * @param postId - The ID of the post to comment on.
 * @param text - The content of the comment.
 * @returns A promise that resolves to an object containing the id of the newly created comment.
 * @throws Will throw an error if the comment creation fails.
 */
export async function createComment(postId: string, text: string): Promise<CommentId> {
	try {
		const commentData = await apiRequest('/api/post/comment', {
			method: 'POST',
			body: JSON.stringify({ post: postId.replace(/^Posts:/, ''), text }),
			headers: {
				'Content-Type': 'application/json'
			}
		});
		return commentData as CommentId;
	} catch (error) {
		console.error('Failed to create comment:', error);
		throw error;
	}
}

/**
 * Ruft einen einzelnen Kommentar anhand seiner ID ab.
 * @param commentId - Die ID des abzurufenden Kommentars (z.B. "commented:xyz123").
 * @returns Ein Promise, das zum Comment-Objekt aufgelöst wird.
 * @throws Wirft einen Fehler, wenn das Abrufen des Kommentars fehlschlägt.
 */
export async function fetchCommentById(commentId: string): Promise<Comment> {
    try {
        // Entferne das Präfix "commented:", falls vorhanden, da der Backend-Endpunkt dies erwartet.
        const cleanedCommentId = commentId.replace(/^commented:/, '');
        
        // Stelle sicher, dass der Pfad mit deiner Rocket-Routenstruktur übereinstimmt.
        // Wenn route_get_comment unter /api/post/comment/... erreichbar ist:
        const commentData = await apiRequest(`/api/post/comment/${cleanedCommentId}`, {
            method: 'GET',
        });
        // Die API gibt ein ViewComment zurück, das dem Frontend-Typ Comment entsprechen sollte.
        // Das 'id'-Feld im zurückgegebenen Kommentar sollte die volle ID sein (z.B. "commented:xyz123").
        return commentData as Comment;
    } catch (error) {
        console.error(`Fehler beim Abrufen des Kommentars ${commentId}:`, error);
        throw error;
    }
}

/**
 * Lädt mehrere Kommentare anhand ihrer IDs.
 * @param commentIds - Array von Kommentar-IDs (ohne "commented:" prefix).
 * @returns Ein Promise, das zu einem Array von Comment-Objekten aufgelöst wird.
 */
export async function fetchCommentsByIds(commentIds: string[]): Promise<Comment[]> {
    const comments: Comment[] = [];
    
    for (const commentId of commentIds) {
        try {
            const comment = await fetchCommentById(commentId);
            comments.push(comment);
        } catch (error) {
            console.error(`Fehler beim Laden von Kommentar ${commentId}:`, error);
            // Weiter mit den anderen Kommentaren, auch wenn einer fehlschlägt
        }
    }
    
    return comments;
}
