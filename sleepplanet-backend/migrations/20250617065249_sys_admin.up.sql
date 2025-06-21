-- 创建管理员用户表
CREATE TABLE admin_user (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

-- 创建管理员角色表
CREATE TABLE admin_roles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES admin_user(id) ON DELETE CASCADE,
    role_name VARCHAR(50) NOT NULL,
    permissions JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, role_name)
);

-- 创建索引
CREATE INDEX idx_admin_user_username ON admin_user(username);
CREATE INDEX idx_admin_user_email ON admin_user(email);
CREATE INDEX idx_admin_roles_user_id ON admin_roles(user_id);

-- 插入默认系统管理员 (初始密码: admin123456，建议首次登录后修改，已使用bcrypt哈希)
INSERT INTO admin_user (username, email, password_hash)
VALUES (
    'sys_admin',
    'admin@example.com',
    '$2a$10$K7DZ1zXj7pK5wH5GQ5GQ5e.3QZ3QZ3QZ3QZ3QZ3QZ3QZ3QZ3QZ'
);

-- 分配管理员角色
INSERT INTO admin_roles (user_id, role_name, permissions)
VALUES (
    (SELECT id FROM admin_user WHERE username = 'sys_admin'),
    'super_admin',
    '{"all_permissions": true}'
);