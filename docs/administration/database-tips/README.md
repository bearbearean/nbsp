# Database tips

Various tips and SQL snippets for commonly used nbsp operations.

The SQL snippets can be run via `psql`.

## Creating invite codes

Invite codes need a `user_creator_id` that indicates who created the invite code.

If you do not want to use a specific user for this, use the special nbsp system user that has `-1` as its `user_id`:

```sql
INSERT INTO invites (user_creator_id)
VALUES (-1), (-1), (-1)
RETURNING invite_code;
```

## Granting invites to users

To let users create invite codes themselves via the web interface, the `user_invite_settings` table record needs to be updated for that user.

If a user does not yet have a record in the `user_invite_settings` table, create one:

```sql
INSERT INTO user_invite_settings (user_id)
VALUES (3)
ON CONFLICT DO NOTHING;
```

Then to grant 10 invite codes to this user:

```sql
UPDATE user_invite_settings
SET available_invite_count = 10
WHERE user_id = 3;
```

Or using `username` by joining on the `users` table:

```sql
UPDATE user_invite_settings uis
SET available_invite_count = 10
FROM users u
WHERE uis.user_id = u.user_id AND u.username = 'bearbearean';
```
