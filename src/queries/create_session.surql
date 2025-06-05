SELECT type::string(id) as id FROM (CREATE Sessions CONTENT { created: time::now(), id: $id, user_id: type::thing('Users', $user_id) });
