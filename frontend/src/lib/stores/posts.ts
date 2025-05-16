import { writable } from 'svelte/store';
import { apiRequest } from '$lib/api';
import type { ViewPost } from '$lib/types';

export const posts = writable<ViewPost[]>([]);

export async function fetchLatestPosts(timeOffset?: string): Promise<ViewPost[]> {
    try {
        const url = timeOffset 
            ? `/api/post/latest?time_offset=${encodeURIComponent(timeOffset)}`
            : '/api/post/latest?time_offset=1970-01-01';
            
        const postData = await apiRequest(url);
        
        // Make sure postData is an array before setting it
        if (Array.isArray(postData)) {
            posts.set(postData);
            return postData;
        } else {
            console.error('Unexpected response format:', postData);
            return [];
        }
    } catch (error) {
        console.error('Failed to fetch posts:', error);
        posts.set([]); // Reset posts store on error
        return [];
    }
}
