-- 删除管理员角色表
DROP TABLE IF EXISTS admin_roles;

-- 删除用户表
DROP TABLE IF EXISTS admin_user;

-- 删除索引
DROP INDEX IF EXISTS idx_admin_user_username;
DROP INDEX IF EXISTS idx_admin_user_email;
DROP INDEX IF EXISTS idx_admin_roles_user_id;