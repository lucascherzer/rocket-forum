import { apiRequest } from '$lib/api';

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
