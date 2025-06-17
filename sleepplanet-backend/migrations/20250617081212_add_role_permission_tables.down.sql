-- 回滚用户-角色关联表（依赖users和roles表）
DROP TABLE IF EXISTS user_roles;

-- 回滚角色-权限关联表（依赖roles和permissions表）
DROP TABLE IF EXISTS role_permissions;

-- 回滚权限表
DROP TABLE IF EXISTS permissions;

-- 回滚角色表
DROP TABLE IF EXISTS roles;

-- 回滚索引（表删除时索引会自动删除，此处仅为显式说明）
-- DROP INDEX IF EXISTS idx_roles_name;
-- DROP INDEX IF EXISTS idx_permissions_code;
-- DROP INDEX IF EXISTS idx_permissions_resource;
-- DROP INDEX IF EXISTS idx_role_permissions_role_id;
-- DROP INDEX IF EXISTS idx_role_permissions_permission_id;
-- DROP INDEX IF EXISTS idx_user_roles_user_id;
-- DROP INDEX IF EXISTS idx_user_roles_role_id;