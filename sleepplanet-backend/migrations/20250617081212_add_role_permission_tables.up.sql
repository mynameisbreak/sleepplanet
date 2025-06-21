-- 创建角色表
-- 存储系统中所有可用角色定义
CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    display_name VARCHAR(100) NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

-- 创建权限表
-- 存储系统中所有细粒度权限项
CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT permissions_resource_action_key UNIQUE(resource, action)
);

-- 创建角色-权限关系表
-- 实现角色与权限的多对多关联
CREATE TABLE role_permissions (
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT role_permissions_pkey PRIMARY KEY (role_id, permission_id)
);

-- 创建用户-角色关系表
-- 实现用户与角色的多对多关联
CREATE TABLE user_roles (
    user_id INTEGER NOT NULL REFERENCES admin_user(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_roles_pkey PRIMARY KEY (user_id, role_id)
);

-- 创建索引提升查询性能
CREATE INDEX idx_roles_name ON roles(name);
CREATE INDEX idx_permissions_code ON permissions(code);
CREATE INDEX idx_permissions_resource ON permissions(resource);
CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission_id ON role_permissions(permission_id);
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);

-- 添加列注释
COMMENT ON COLUMN roles.name IS '角色名称，如：admin, editor';
COMMENT ON COLUMN roles.display_name IS '角色显示名称，用于UI展示';
COMMENT ON COLUMN roles.description IS '角色功能描述';
COMMENT ON COLUMN roles.created_at IS '创建时间';
COMMENT ON COLUMN roles.updated_at IS '更新时间';
COMMENT ON COLUMN roles.is_active IS '是否启用该角色';

COMMENT ON COLUMN permissions.code IS '权限唯一标识，如：user:create, audio:delete';
COMMENT ON COLUMN permissions.name IS '权限显示名称';
COMMENT ON COLUMN permissions.description IS '权限详细描述';
COMMENT ON COLUMN permissions.resource IS '权限所属资源，如：user, audio, category';
COMMENT ON COLUMN permissions.action IS '操作类型，如：create, read, update, delete';
COMMENT ON COLUMN permissions.created_at IS '创建时间';
COMMENT ON COLUMN permissions.updated_at IS '更新时间';

COMMENT ON COLUMN role_permissions.role_id IS '角色ID，关联roles表';
COMMENT ON COLUMN role_permissions.permission_id IS '权限ID，关联permissions表';
COMMENT ON COLUMN role_permissions.created_at IS '关联创建时间';

COMMENT ON COLUMN user_roles.user_id IS '用户ID，关联admin_user表';
COMMENT ON COLUMN user_roles.role_id IS '角色ID，关联roles表';
COMMENT ON COLUMN user_roles.created_at IS '关联创建时间';

-- 添加约束注释
COMMENT ON CONSTRAINT permissions_resource_action_key ON permissions IS '确保同一资源的同一操作只有一个权限项';
COMMENT ON CONSTRAINT role_permissions_pkey ON role_permissions IS '复合主键确保角色-权限关联唯一';
COMMENT ON CONSTRAINT user_roles_pkey ON user_roles IS '复合主键确保用户-角色关联唯一';

-- 插入默认角色数据
INSERT INTO roles (name, display_name, description)
VALUES 
    ('super_admin', '超级管理员', '拥有系统全部权限'),
    ('content_admin', '内容管理员', '负责音频内容管理'),
    ('user_admin', '用户管理员', '负责用户账户管理');

-- 插入默认权限数据（用户管理相关）
INSERT INTO permissions (code, name, description, resource, action)
VALUES 
    ('user:create', '创建用户', '允许创建新用户', 'user', 'create'),
    ('user:read', '查看用户', '允许查看用户信息', 'user', 'read'),
    ('user:update', '修改用户', '允许修改用户信息', 'user', 'update'),
    ('user:delete', '删除用户', '允许删除用户', 'user', 'delete'),
    ('user:list', '用户列表', '允许查看用户列表', 'user', 'list');

-- 插入默认权限数据（音频管理相关）
INSERT INTO permissions (code, name, description, resource, action)
VALUES 
    ('audio:create', '创建音频', '允许上传新音频', 'audio', 'create'),
    ('audio:read', '查看音频', '允许查看音频信息', 'audio', 'read'),
    ('audio:update', '修改音频', '允许修改音频信息', 'audio', 'update'),
    ('audio:delete', '删除音频', '允许删除音频', 'audio', 'delete'),
    ('audio:list', '音频列表', '允许查看音频列表', 'audio', 'list');

-- 为超级管理员分配所有权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'super_admin'),
    id
FROM permissions;

-- 为内容管理员分配音频相关权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'content_admin'),
    id
FROM permissions
WHERE resource = 'audio';

-- 为用户管理员分配用户相关权限
INSERT INTO role_permissions (role_id, permission_id)
SELECT 
    (SELECT id FROM roles WHERE name = 'user_admin'),
    id
FROM permissions
WHERE resource = 'user';

-- 将默认管理员用户关联到超级管理员角色
INSERT INTO user_roles (user_id, role_id)
VALUES (
    (SELECT id FROM admin_user WHERE username = 'sys_admin'),
    (SELECT id FROM roles WHERE name = 'super_admin')
);