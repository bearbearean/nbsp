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
