-- 删除管理员角色表
DROP TABLE IF EXISTS admin_roles;

-- 删除用户表
DROP TABLE IF EXISTS users;

-- 删除索引
DROP INDEX IF EXISTS idx_users_username;
DROP INDEX IF EXISTS idx_users_email;
DROP INDEX IF EXISTS idx_admin_roles_user_id;