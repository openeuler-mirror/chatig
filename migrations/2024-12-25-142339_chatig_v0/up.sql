-- Your SQL goes here
-- 测试用数据表，后续有真实表设计后去除
CREATE TABLE test01 (
    id SERIAL PRIMARY KEY,         
    name VARCHAR(255) NOT NULL,      
    description TEXT,               
    created_at TIMESTAMP DEFAULT NOW(), 
    updated_at TIMESTAMP DEFAULT NOW()  
);