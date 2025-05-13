-- ------------------------------
-- OPTION
-- ------------------------------

OPTION IMPORT;

-- ------------------------------
-- FUNCTIONS
-- ------------------------------

DEFINE FUNCTION IF NOT EXISTS fn::get_userid_from_session($session_id: any) -> uuid {
    LET $session_data = (SELECT user_id FROM Sessions WHERE id = $session_id);
    IF array::is_empty($session_data) THEN (RETURN []) END;
    LET $user_data = (SELECT * FROM $session_data[0].user_id);
    IF array::is_empty($user_data) OR $user_data[0].id = NONE THEN (RETURN []) END;
    SELECT * FROM $user_data[0].id;
    RETURN record::id($user_data[0].id);
} PERMISSIONS FULL;
DEFINE FUNCTION IF NOT EXISTS fn::new_session($user_id: any) -> uuid {
    LET $id = rand::uuid::v7();
    INSERT INTO Sessions { created: time::now(), id: $id, user_id: type::thing('Users', $user_id) };
    RETURN $id;
} PERMISSIONS FULL;

-- ------------------------------
-- ANALYZERS
-- ------------------------------

DEFINE ANALYZER IF NOT EXISTS fingerprint_analyzer TOKENIZERS BLANK,CLASS,CAMEL,PUNCT;

-- ------------------------------
-- TABLE: ClientFingerprints
-- ------------------------------

DEFINE TABLE IF NOT EXISTS ClientFingerprints TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD embeddings ON ClientFingerprints TYPE array<number> PERMISSIONS FULL;
DEFINE FIELD embeddings[*] ON ClientFingerprints TYPE number PERMISSIONS FULL;
DEFINE FIELD session_id ON ClientFingerprints TYPE string PERMISSIONS FULL;
DEFINE FIELD source_ip ON ClientFingerprints PERMISSIONS FULL;
DEFINE FIELD user_agent ON ClientFingerprints TYPE string PERMISSIONS FULL;


DEFINE INDEX IF NOT EXISTS fingerprint_index ON ClientFingerprints
    FIELDS embeddings
    SEARCH ANALYZER fingerprint_analyzer
    MTREE DIMENSION 384 // the dimension of the embeddings model
    DIST MANHATTAN; // l1 norm is best in high dimensional space: https://link.springer.com/chapter/10.1007/3-540-44503-X_27

-- ------------------------------
-- TABLE: Posts
-- ------------------------------

DEFINE TABLE IF NOT EXISTS Posts TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS created_at ON Posts TYPE datetime PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS deleted ON Posts TYPE bool PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS hashtags ON Posts TYPE array<string> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS hashtags[*] ON Posts TYPE string PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS heading ON Posts TYPE string PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS images ON Posts TYPE set<string> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS images[*] ON Posts TYPE string PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS text ON Posts TYPE string PERMISSIONS FULL;

-- ------------------------------
-- TABLE: Sessions
-- ------------------------------

DEFINE TABLE IF NOT EXISTS Sessions TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS created ON Sessions TYPE datetime PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS user_id ON Sessions TYPE record<Users> PERMISSIONS FULL;

-- ------------------------------
-- TABLE: Users
-- ------------------------------

DEFINE TABLE IF NOT EXISTS Users TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS created ON Users TYPE datetime PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS password ON Users TYPE string PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS role ON Users TYPE string PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS username ON Users TYPE string PERMISSIONS FULL;

-- ------------------------------
-- TABLE: commented
-- ------------------------------

DEFINE TABLE IF NOT EXISTS commented TYPE RELATION IN Users OUT Posts SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS created_at ON commented TYPE datetime PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS deleted ON commented TYPE bool PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS dislikes ON commented TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS hashtags ON commented TYPE array<string> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS hashtags[*] ON commented TYPE string PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS in ON commented TYPE record<Users> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS likes ON commented TYPE int DEFAULT 0 PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS out ON commented TYPE record<Posts> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS text ON commented TYPE string PERMISSIONS FULL;

-- ------------------------------
-- TABLE: created
-- ------------------------------

DEFINE TABLE IF NOT EXISTS created TYPE RELATION IN Users OUT Posts SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS in ON created TYPE record<Users> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS out ON created TYPE record<Posts> PERMISSIONS FULL;

-- ------------------------------
-- TABLE: disliked
-- ------------------------------

DEFINE TABLE IF NOT EXISTS disliked TYPE RELATION IN Users OUT Posts SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS in ON disliked TYPE record<Users> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS out ON disliked TYPE record<Posts> PERMISSIONS FULL;


-- ------------------------------
-- TABLE: follows
-- ------------------------------

DEFINE TABLE IF NOT EXISTS follows TYPE RELATION IN Users OUT Users SCHEMALESS PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS in ON follows TYPE record<Users> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS out ON follows TYPE record<Users> PERMISSIONS FULL;


-- ------------------------------
-- TABLE: liked
-- ------------------------------

DEFINE TABLE IF NOT EXISTS liked TYPE RELATION IN Users OUT Posts | commented SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD IF NOT EXISTS created_at ON liked TYPE datetime PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS in ON liked TYPE record<Users> PERMISSIONS FULL;
DEFINE FIELD IF NOT EXISTS out ON liked TYPE record<Posts | commented> PERMISSIONS FULL;
