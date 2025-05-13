// In this query, we check whether a fingerprint exists. which is similar enough
// to be considered equal. If none exist, we insert it, opening a new rate
// limiting bucket
LET $new_point = type::array($embeddings);

LET $nn = (
    SELECT * FROM (
        SELECT *, vector::distance::manhattan(embeddings, $new_point) AS dist
        FROM ClientFingerprints
        WHERE embeddings <|2|> $new_point
    )
    WHERE dist <= $MAX_DIST
);
IF array::len($nn) = 0 {
    // We only insert when no neighbours within the threshold are found.
    LET $latest = (CREATE ClientFingerprints CONTENT {
        embeddings: $embeddings,
        user_agent: $user_agent,
        source_ip: $source_ip,
        session_id: $session_id
    });
    RETURN {
        found_before: false,
        created: $latest[0].id
    };
} ELSE {
    RETURN {
        found_before: true,
        nearest_neighbour: $nn[0].id
    };
}
