/**
 * Represents the structure of a post object as it is viewed in the frontend.
 */
export interface ViewPost {
    id: string;
    heading: string;
    text: string;
    hashtags: string[];
    created_at: string;
}