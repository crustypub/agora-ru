use sqlx::PgPool;
use uuid::Uuid;
use crate::models::chat::{
    ChatListItem, ChatMessage, RoomMemberInfo, RoomRequesterInfo, MemberRolesInfo,
    MessageInfoForDeletion, DbMessageAttachment,
};

pub async fn get_user_rooms_count(
    pool: &PgPool,
    user_id: Uuid,
    search_pattern: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM rooms r
        JOIN room_members rm ON r.id = rm.room_id
        WHERE rm.user_id = $1
          AND (
              $2::text IS NULL OR (
                  r.name ILIKE $2 OR
                  (
                      r.type = 'direct' AND EXISTS (
                          SELECT 1 FROM room_members orm
                          JOIN users ou ON orm.user_id = ou.id
                          WHERE orm.room_id = r.id AND orm.user_id != $1
                            AND (
                                ou.username ILIKE $2 OR
                                ou.first_name ILIKE $2 OR
                                ou.last_name ILIKE $2
                            )
                      )
                  )
              )
          )
        "#,
    )
    .bind(user_id)
    .bind(search_pattern)
    .fetch_one(pool)
    .await
}

pub async fn get_user_rooms(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
    search_pattern: Option<&str>,
) -> Result<Vec<ChatListItem>, sqlx::Error> {
    sqlx::query_as::<_, ChatListItem>(
        r#"
        SELECT
            r.id,
            r.type::text as room_type,
            r.name,
            r.description,
            r.direct_key,
            EXTRACT(EPOCH FROM r.created_at)::BIGINT as created_at,
            EXTRACT(EPOCH FROM r.updated_at)::BIGINT as updated_at,
            (
                SELECT COUNT(*)
                FROM messages m
                WHERE m.room_id = r.id
                  AND m.created_at > rm.last_read_at
                  AND (m.sender_id IS NULL OR m.sender_id != $1)
                  AND m.deleted_at IS NULL
                  AND NOT EXISTS (
                      SELECT 1 FROM deleted_messages dm
                      WHERE dm.message_id = m.id AND dm.user_id = $1
                  )
            ) AS unread_count,
            (
                SELECT json_build_object(
                    'id', m.id,
                    'room_id', m.room_id,
                    'sender_id', m.sender_id,
                    'content', m.content,
                    'created_at', EXTRACT(EPOCH FROM m.created_at)::BIGINT,
                    'author', json_build_object(
                        'id', u.id,
                        'username', u.username,
                        'first_name', u.first_name,
                        'last_name', u.last_name,
                        'avatar_url', u.avatar_url
                    ),
                    'is_read', CASE
                        WHEN m.sender_id = $1 THEN 
                            EXISTS (
                                SELECT 1 
                                FROM room_members rm_other 
                                WHERE rm_other.room_id = m.room_id 
                                  AND rm_other.user_id != $1 
                                  AND rm_other.last_read_at >= m.created_at
                            )
                        ELSE 
                            EXISTS (
                                SELECT 1 
                                FROM room_members rm_self 
                                WHERE rm_self.room_id = m.room_id 
                                  AND rm_self.user_id = $1 
                                  AND rm_self.last_read_at >= m.created_at
                            )
                    END
                )
                FROM messages m
                LEFT JOIN users u ON m.sender_id = u.id
                WHERE m.room_id = r.id
                  AND m.deleted_at IS NULL
                  AND NOT EXISTS (
                      SELECT 1 FROM deleted_messages dm
                      WHERE dm.message_id = m.id AND dm.user_id = $1
                  )
                ORDER BY m.created_at DESC
                LIMIT 1
            ) AS last_message,
            CASE 
                WHEN r.type = 'direct' THEN (
                    SELECT json_build_object(
                        'id', ou.id,
                        'username', ou.username,
                        'first_name', ou.first_name,
                        'last_name', ou.last_name,
                        'avatar_url', ou.avatar_url
                    )
                    FROM room_members orm
                    JOIN users ou ON orm.user_id = ou.id
                    WHERE orm.room_id = r.id AND orm.user_id != $1
                    LIMIT 1
                )
                ELSE NULL
            END AS direct_user
        FROM rooms r
        JOIN room_members rm ON r.id = rm.room_id
        WHERE rm.user_id = $1
          AND (
              $4::text IS NULL OR (
                  r.name ILIKE $4 OR
                  (
                      r.type = 'direct' AND EXISTS (
                          SELECT 1 FROM room_members orm
                          JOIN users ou ON orm.user_id = ou.id
                          WHERE orm.room_id = r.id AND orm.user_id != $1
                            AND (
                                ou.username ILIKE $4 OR
                                ou.first_name ILIKE $4 OR
                                ou.last_name ILIKE $4
                            )
                      )
                  )
              )
          )
        ORDER BY COALESCE(
            (
                SELECT MAX(m.created_at)
                FROM messages m
                WHERE m.room_id = r.id
                  AND m.deleted_at IS NULL
                  AND NOT EXISTS (
                      SELECT 1 FROM deleted_messages dm
                      WHERE dm.message_id = m.id AND dm.user_id = $1
                  )
            ),
            r.updated_at
        ) DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .bind(search_pattern)
    .fetch_all(pool)
    .await
}

pub async fn get_direct_room_by_key(
    pool: &PgPool,
    direct_key: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM rooms WHERE direct_key = $1")
        .bind(direct_key)
        .fetch_optional(pool)
        .await
}

pub async fn create_direct_room_transaction(
    pool: &PgPool,
    author_id: Uuid,
    user_2: Uuid,
    direct_key: &str,
) -> Result<Uuid, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let room_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO rooms (type, direct_key)
        VALUES ($1::room_type, $2)
        ON CONFLICT (direct_key) DO UPDATE SET updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind("direct")
    .bind(direct_key)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        ON CONFLICT (room_id, user_id) DO NOTHING
        "#,
    )
    .bind(room_id)
    .bind(author_id)
    .bind("member")
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        ON CONFLICT (room_id, user_id) DO NOTHING
        "#,
    )
    .bind(room_id)
    .bind(user_2)
    .bind("member")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(room_id)
}

pub async fn create_group_room_transaction(
    pool: &PgPool,
    author_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let room_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO rooms (type, name, description)
        VALUES ($1::room_type, $2, $3)
        RETURNING id
        "#,
    )
    .bind("group")
    .bind(name)
    .bind(description)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        "#,
    )
    .bind(room_id)
    .bind(author_id)
    .bind("owner")
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(room_id)
}

pub async fn get_room_requester_info(
    pool: &PgPool,
    room_id: Uuid,
    requester_id: Uuid,
) -> Result<Option<RoomRequesterInfo>, sqlx::Error> {
    sqlx::query_as::<_, RoomRequesterInfo>(
        r#"
        SELECT type::text as room_type,
               (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $2) as requester_role
        FROM rooms WHERE id = $1
        "#,
    )
    .bind(room_id)
    .bind(requester_id)
    .fetch_optional(pool)
    .await
}

pub async fn check_user_exists(pool: &PgPool, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(exists)
}

pub async fn insert_room_member(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query(
        r#"
        INSERT INTO room_members (room_id, user_id, role)
        VALUES ($1, $2, $3::room_role)
        ON CONFLICT (room_id, user_id) DO NOTHING
        "#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(role)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn get_member_roles_info(
    pool: &PgPool,
    room_id: Uuid,
    requester_id: Uuid,
    target_user_id: Uuid,
) -> Result<Option<MemberRolesInfo>, sqlx::Error> {
    sqlx::query_as::<_, MemberRolesInfo>(
        r#"
        SELECT 
            (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $2) as requester_role,
            (SELECT role::text FROM room_members WHERE room_id = $1 AND user_id = $3) as target_role,
            (SELECT type::text FROM rooms WHERE id = $1) as room_type
        "#,
    )
    .bind(room_id)
    .bind(requester_id)
    .bind(target_user_id)
    .fetch_optional(pool)
    .await
}

pub async fn remove_room_member_transaction(
    pool: &PgPool,
    room_id: Uuid,
    target_user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM room_members WHERE room_id = $1 AND user_id = $2")
        .bind(room_id)
        .bind(target_user_id)
        .execute(&mut *tx)
        .await?;

    let remaining_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM room_members WHERE room_id = $1",
    )
    .bind(room_id)
    .fetch_one(&mut *tx)
    .await?;

    if remaining_count == 0 {
        sqlx::query("DELETE FROM rooms WHERE id = $1")
            .bind(room_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(remaining_count)
}

pub async fn get_message_info_for_deletion(
    pool: &PgPool,
    message_id: Uuid,
    requester_id: Uuid,
) -> Result<Option<MessageInfoForDeletion>, sqlx::Error> {
    sqlx::query_as::<_, MessageInfoForDeletion>(
        r#"
        SELECT 
            m.sender_id,
            m.room_id,
            (SELECT role::text FROM room_members rm WHERE rm.room_id = m.room_id AND rm.user_id = $2) as user_role,
            (SELECT EXISTS(SELECT 1 FROM room_members rm WHERE rm.room_id = m.room_id AND rm.user_id = $2)) as is_member
        FROM messages m
        WHERE m.id = $1 AND m.deleted_at IS NULL
        "#,
    )
    .bind(message_id)
    .bind(requester_id)
    .fetch_optional(pool)
    .await
}

pub async fn delete_message_everyone_transaction(
    pool: &PgPool,
    message_id: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("UPDATE messages SET deleted_at = NOW() WHERE id = $1")
        .bind(message_id)
        .execute(&mut *tx)
        .await?;

    #[derive(sqlx::FromRow)]
    struct AttachmentKey {
        file_key: String,
    }

    let attachments = sqlx::query_as::<_, AttachmentKey>(
        "SELECT file_key FROM message_attachments WHERE message_id = $1",
    )
    .bind(message_id)
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM message_attachments WHERE message_id = $1")
        .bind(message_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(attachments.into_iter().map(|a| a.file_key).collect())
}

pub async fn delete_message_me_transaction(
    pool: &PgPool,
    message_id: Uuid,
    requester_id: Uuid,
    room_id: Uuid,
) -> Result<(bool, Vec<String>), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO deleted_messages (message_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    )
    .bind(message_id)
    .bind(requester_id)
    .execute(&mut *tx)
    .await?;

    let total_members = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM room_members WHERE room_id = $1",
    )
    .bind(room_id)
    .fetch_one(&mut *tx)
    .await?;

    let deleted_by_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM deleted_messages WHERE message_id = $1",
    )
    .bind(message_id)
    .fetch_one(&mut *tx)
    .await?;

    let mut attachments_to_delete = Vec::new();
    let mut fully_deleted = false;

    if deleted_by_count >= total_members {
        fully_deleted = true;
        
        #[derive(sqlx::FromRow)]
        struct AttachmentKey {
            file_key: String,
        }

        let attachments = sqlx::query_as::<_, AttachmentKey>(
            "SELECT file_key FROM message_attachments WHERE message_id = $1",
        )
        .bind(message_id)
        .fetch_all(&mut *tx)
        .await?;

        attachments_to_delete = attachments.into_iter().map(|a| a.file_key).collect();

        sqlx::query("DELETE FROM messages WHERE id = $1")
            .bind(message_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok((fully_deleted, attachments_to_delete))
}

pub async fn check_is_room_member(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let is_member: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM room_members WHERE room_id = $1 AND user_id = $2)"
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(is_member)
}

pub async fn save_message_and_attachments_transaction(
    pool: &PgPool,
    msg_id: Uuid,
    room_id: Uuid,
    sender_id: Uuid,
    content: &str,
    created_at: chrono::DateTime<chrono::Utc>,
    attachments: &Option<Vec<crate::models::chat::SendMessageAttachment>>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO messages (id, room_id, sender_id, content, created_at) 
         VALUES ($1, $2, $3, $4, $5)"
    )
    .bind(msg_id)
    .bind(room_id)
    .bind(sender_id)
    .bind(content)
    .bind(created_at)
    .execute(&mut *tx)
    .await?;

    if let Some(atts) = attachments {
        for att in atts {
            let att_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO message_attachments (id, message_id, file_key, file_name, file_size, file_mime)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(att_id)
            .bind(msg_id)
            .bind(&att.file_key)
            .bind(&att.file_name)
            .bind(att.file_size)
            .bind(&att.file_mime)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn get_room_member_ids(
    pool: &PgPool,
    room_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM room_members WHERE room_id = $1"
    )
    .bind(room_id)
    .fetch_all(pool)
    .await
}

pub async fn update_last_read_at(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        "UPDATE room_members SET last_read_at = NOW() 
         WHERE room_id = $1 AND user_id = $2
         RETURNING EXTRACT(EPOCH FROM last_read_at)::BIGINT",
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn get_other_room_member_ids(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM room_members WHERE room_id = $1 AND user_id != $2",
    )
    .bind(room_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn count_room_messages(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
    search_pattern: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM messages m
        WHERE m.room_id = $1
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1 FROM deleted_messages dm
              WHERE dm.message_id = m.id AND dm.user_id = $2
          )
          AND ($3::text IS NULL OR m.content ILIKE $3)
        "#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(search_pattern)
    .fetch_one(pool)
    .await
}

pub async fn get_room_messages(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
    search_pattern: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ChatMessage>, sqlx::Error> {
    sqlx::query_as::<_, ChatMessage>(
        r#"
        SELECT 
            m.id,
            m.room_id,
            m.sender_id,
            m.content,
            EXTRACT(EPOCH FROM m.created_at)::BIGINT as created_at,
            CASE 
                WHEN m.sender_id IS NOT NULL THEN json_build_object(
                    'id', u.id,
                    'username', u.username,
                    'first_name', u.first_name,
                    'last_name', u.last_name,
                    'avatar_url', u.avatar_url
                )
                ELSE NULL
            END as author,
            CASE
                WHEN m.sender_id = $2 THEN
                    EXISTS (
                        SELECT 1
                        FROM room_members rm_other
                        WHERE rm_other.room_id = m.room_id
                          AND rm_other.user_id != $2
                          AND rm_other.last_read_at >= m.created_at
                    )
                ELSE
                    EXISTS (
                        SELECT 1
                        FROM room_members rm_self
                        WHERE rm_self.room_id = m.room_id
                          AND rm_self.user_id = $2
                          AND rm_self.last_read_at >= m.created_at
                    )
            END as is_read
        FROM messages m
        LEFT JOIN users u ON m.sender_id = u.id
        WHERE m.room_id = $1
          AND m.deleted_at IS NULL
          AND NOT EXISTS (
              SELECT 1 FROM deleted_messages dm
              WHERE dm.message_id = m.id AND dm.user_id = $2
          )
          AND ($3::text IS NULL OR m.content ILIKE $3)
        ORDER BY m.created_at DESC
        LIMIT $4 OFFSET $5
        "#,
    )
    .bind(room_id)
    .bind(user_id)
    .bind(search_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn get_message_attachments_raw(
    pool: &PgPool,
    message_ids: &[Uuid],
) -> Result<Vec<DbMessageAttachment>, sqlx::Error> {
    sqlx::query_as::<_, DbMessageAttachment>(
        r#"
        SELECT id, message_id, file_key, file_name, file_size, file_mime
        FROM message_attachments
        WHERE message_id = ANY($1)
        "#,
    )
    .bind(message_ids)
    .fetch_all(pool)
    .await
}

pub async fn get_room_members_info(
    pool: &PgPool,
    room_id: Uuid,
) -> Result<Vec<RoomMemberInfo>, sqlx::Error> {
    sqlx::query_as::<_, RoomMemberInfo>(
        r#"
        SELECT 
            u.id,
            u.username,
            u.first_name,
            u.last_name,
            u.avatar_url,
            rm.role::text as role
        FROM room_members rm
        JOIN users u ON rm.user_id = u.id
        WHERE rm.room_id = $1
        "#,
    )
    .bind(room_id)
    .fetch_all(pool)
    .await
}
