import { writable } from 'svelte/store';
import { apiRequest } from '$lib/api';
import type { ViewPost } from '$lib/types';

export const posts = writable<ViewPost[]>([]);

/**
 * Fetches the latest posts from the API.
 * @param timeOffset - Optional ISO 8601 timestamp to fetch posts older than this date. Defaults to 1970-01-01.
 * @returns A promise that resolves to an array of ViewPost objects.
 */
export async function fetchLatestPosts(timeOffset?: string): Promise<ViewPost[]> {
    try {
        const url = timeOffset 
            ? `/api/post/latest?time_offset=${encodeURIComponent(timeOffset)}`
            : '/api/post/latest?time_offset=1970-01-01';
            
        const postData = await apiRequest(url);
        
        if (Array.isArray(postData)) {
            posts.set(postData);
            return postData;
        } else {
            console.error('Unexpected response format:', postData);
            return [];
        }
    } catch (error) {
        console.error('Failed to fetch posts:', error);
        posts.set([]); 
        return [];
    }
}

/**
 * Creates a new post.
 * @param heading - The title of the post.
 * @param text - The content of the post.
 * @returns A promise that resolves to an object containing the id of the newly created post.
 * @throws Will throw an error if the post creation fails.
 */
export async function createPost(heading: string, text: string): Promise<{ id: string }> {
    try {
        const newPostData = await apiRequest('/api/post/new', {
            method: 'POST',
            body: JSON.stringify({ heading, text }),
        });
        return newPostData as { id: string };
    } catch (error) {
        console.error('Failed to create post:', error);
        throw error;
    }
}
