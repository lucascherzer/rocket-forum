/**
 * Represents the structure of a post object as it is viewed in the frontend.
 */
export interface ViewPost {
	id: string;
	heading: string;
	text: string;
	author: string;
	created_at: string;
	hashtags: string[];
	comment_ids: string[]; // Stelle sicher, dass dieses Feld existiert
	likes: number;
}

export interface Comment {
    id: string;
    post_id: string;
    author: string;
    text: string;
    created_at: string; // z.B. ISO 8601 Zeitstempel
}
