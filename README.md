# 多租户微信点餐系统

> 📖 **完整用户手册（含界面截图与分步引导）**：[docs/USER_MANUAL.md](docs/USER_MANUAL.md)

## 项目定位

同一套系统服务多个餐饮门店（租户）。顾客扫描门店实体二维码进入对应租户的点餐页，微信无感登录后选桌号、点菜（含口味与备注）、下单取餐，同一轮消费内可追加菜品；租户管理员在小程序内维护店铺介绍、分类、菜品、价格上下架与菜单图片，并通过**排餐看板**按桌出餐（同桌相同菜品与口味自动合并）；超级管理员可登录小程序切换任意门店管理其全部内容，也可通过 PC 后台管理全部租户、门店二维码、全局经营数据与管理员账号。

## 项目结构

- `backend`：Rust 后端服务（Axum + sqlx + PostgreSQL）
- `miniprogram`：微信原生小程序（扫码入口、顾客端、租户管理端、排餐看板）
- `admin-web`：PC 后台，仅包含超级管理员功能（Vue 3 + Vite + Element Plus）
- `database`：独立数据库创建脚本
- `backend/migrations`：数据库建表迁移脚本
- `docs/USER_MANUAL.md`：用户手册（下载、安装、运行与使用引导，含界面截图）
- `docs/images`：用户手册界面截图

## 技术栈

- 后端：Rust、Axum、Tokio、sqlx、PostgreSQL、Argon2
- 小程序：微信原生小程序
- PC 后台：Vue 3、Vite、Element Plus、Axios

## 多租户与登录模型

- 所有业务数据（店铺信息、分类、菜品、订单、营业状态、管理员账号、顾客账号等）都带 `tenant_id`，接口查询与写入全部按租户过滤。
- 三类账号：
  - `CUSTOMER`：顾客，注册/登录后只能在本租户下单、查看自己的订单。
  - `TENANT_ADMIN`：租户管理员，只能操作所属租户数据。
  - `SUPER_ADMIN`：超级管理员，可查看和操作任意租户，调用租户接口时必须用 `tenantCode` 或 `tenantId` 指定租户。
- 登录成功后返回随机会话令牌；除公开接口外均需请求头 `Authorization: Bearer <token>`。令牌保存在 `auth_session` 表，按 `TOKEN_TTL_HOURS` 过期。
- 小程序二维码内容为 `orderfood://tenant/{租户编码}`；入口页会解析租户编码并调用 `/api/tenants/{code}/entry`，缺少租户标识时提示先扫码。

## 环境要求

- Rust 1.80 及以上（已在 Rust 1.97 验证）
- PostgreSQL，默认连接 `127.0.0.1:15432`，用户 `postgres`，密码 `postgres`

## 从零启动

### 1. 创建本项目数据库

```powershell
$env:PGPASSWORD = 'postgres'
psql -h 127.0.0.1 -p 15432 -U postgres -f database/create_database.sql
```

脚本会创建独立数据库 `order_food`，重复执行不会报错。若 `psql` 不在 PATH，请使用 PostgreSQL 安装目录下的完整路径，例如 `D:\DevelopTools\pg1418\bin\psql.exe`。

### 2. 配置并启动 Rust 后端

```powershell
cd backend
Copy-Item .env.example .env   # 首次使用时创建；按需修改配置
cargo run -- migrate         # 显式执行迁移（服务启动时也会自动执行）
cargo run
```

服务默认监听 `http://localhost:8080`。启动时会自动初始化初始租户、超级管理员、租户管理员与示例菜单（已存在则跳过）。

### 3. 启动 PC 超级管理员后台

```powershell
cd admin-web
npm install
npm run dev
```

访问 `http://localhost:5173`。PC 端只有超级管理员界面，租户管理员账号无法登录。

### 4. 运行微信小程序

1. 使用微信开发者工具导入 `miniprogram` 目录。
2. 首屏为门店入口页：扫描实体二维码（内容形如 `orderfood://tenant/T001`）或手动输入门店编码。
3. 顾客登录后可下单、查看自己的历史订单；租户管理员从入口页进入管理端。
4. 小程序请求地址为 `http://localhost:8080/api`。

## 配置项

后端读取 `backend/.env` 或环境变量：

| 配置项 | 说明 | 示例/默认值 |
| --- | --- | --- |
| `DATABASE_URL` | PostgreSQL 连接串，必填 | 无（示例见 `.env.example`） |
| `APP_PORT` | 后端监听端口 | `8080` |
| `IMAGE_DIR` | 图片统一存储目录 | `./data/images` |
| `PUBLIC_BASE_URL` | 生成图片/二维码访问地址的基础地址 | `http://localhost:8080` |
| `TOKEN_TTL_HOURS` | 登录令牌有效期（小时） | `168` |
| `SUPER_ADMIN_USERNAME` | 初始超级管理员账号 | `superadmin` |
| `SUPER_ADMIN_PASSWORD` | 初始超级管理员密码 | 见 `.env` |
| `SUPER_ADMIN_DISPLAY_NAME` | 初始超级管理员显示名 | `超级管理员` |
| `INITIAL_TENANT_CODE` | 初始租户编码 | `T001` |
| `INITIAL_TENANT_NAME` | 初始租户名称 | `食光小馆` |
| `INITIAL_TENANT_STORE_NAME` | 初始门店名称 | `食光小馆` |
| `INITIAL_TENANT_NOTICE` | 初始营业公告 | `欢迎光临，现点现做` |
| `INITIAL_TENANT_INTRO` | 初始店铺介绍 | `精选时令食材……` |
| `INITIAL_TENANT_ADMIN_USERNAME` | 初始租户管理员账号 | `tenantadmin` |
| `INITIAL_TENANT_ADMIN_PASSWORD` | 初始租户管理员密码 | 见 `.env` |
| `INITIAL_TENANT_ADMIN_DISPLAY_NAME` | 初始租户管理员显示名 | `门店管理员` |

密码、连接串、目录均通过配置文件或环境变量提供，不写在业务代码中。

## 数据库迁移

- 建库脚本：[database/create_database.sql](database/create_database.sql)
- 迁移脚本：[backend/migrations/0001_init.sql](backend/migrations/0001_init.sql)
- 迁移由 sqlx 管理，执行记录保存在 `_sqlx_migrations` 表；服务启动时会自动执行未应用的迁移。
- 初始化数据（租户、超管、租户管理员、默认分类与示例菜品）由 `backend/src/bootstrap.rs` 幂等创建，因为密码哈希无法在 SQL 中生成。

需要完全重建时：

```powershell
$env:PGPASSWORD = 'postgres'
psql -h 127.0.0.1 -p 15432 -U postgres -c "DROP DATABASE IF EXISTS order_food;"
psql -h 127.0.0.1 -p 15432 -U postgres -f database/create_database.sql
cd backend
cargo run -- migrate
```

## 图片存储与访问

所有上传图片和生成的租户二维码都写入 `IMAGE_DIR` 指定的同一目录，通过 `/images/{filename}` 静态访问。上传、覆盖、删除统一由后端图片存储模块处理，接口需要管理员登录态。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/api/images` | 上传图片，`multipart/form-data`，文件字段 `file`，可选字段 `filename` |
| `PUT` | `/api/images/{filename}` | 覆盖已有图片（不存在时创建），文件字段 `file` |
| `DELETE` | `/api/images/{filename}` | 删除图片（不存在时同样返回成功） |
| `GET` | `/images/{filename}` | 访问图片静态资源 |

限制与规则：

- 支持 `jpg`、`jpeg`、`png`、`gif`、`webp`、`bmp`，单张不超过 10MB。
- 未指定 `filename` 时自动生成 UUID 文件名。
- 返回 `filename`、`path`（`/images/...`）、`url`（基于 `PUBLIC_BASE_URL` 的完整地址）。

## 主要接口

所有接口保持 `{ success, message, data }` 响应包裹；业务错误返回 HTTP 200 且 `success=false`，鉴权失败返回 401/403。

### 登录与入口

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/api/auth/customer/register` | 顾客注册（租户编码 + 手机号 + 密码） |
| `POST` | `/api/auth/customer/login` | 顾客登录 |
| `GET` | `/api/auth/me` | 当前登录账号信息 |
| `POST` | `/api/auth/logout` | 退出登录（令牌失效） |
| `POST` | `/api/admin/login` | 租户管理员 / 超级管理员登录 |
| `GET` | `/api/tenants/{code}/entry` | 扫码入口校验，返回门店信息（公开） |

### 菜单与营业状态

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/menu/overview?tenantCode=` | 菜单总览（营业状态 + 分类 + 菜品，公开） |
| `GET` | `/api/menu/items?tenantCode=` | 菜品列表（公开） |
| `POST` | `/api/menu/items?tenantCode=` | 新增菜品（管理员） |
| `PUT` | `/api/menu/items/{id}?tenantCode=` | 更新菜品、价格、上下架、图片（管理员） |
| `DELETE` | `/api/menu/items/{id}?tenantCode=` | 删除菜品（管理员） |
| `POST` | `/api/menu/categories?tenantCode=` | 新增分类（管理员） |
| `PUT` | `/api/menu/categories/{id}?tenantCode=` | 更新分类（管理员） |
| `DELETE` | `/api/menu/categories/{id}?tenantCode=` | 删除分类（管理员，分类下有菜品时拒绝） |
| `GET` | `/api/business/status?tenantCode=` | 查询营业状态与店铺介绍（公开） |
| `PUT` | `/api/business/status?tenantCode=` | 更新营业状态、店铺介绍、公告（管理员） |

### 订单

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `POST` | `/api/orders` | 创建订单（顾客令牌，租户来自登录态） |
| `GET` | `/api/orders` | 管理员查看本租户订单 |
| `GET` | `/api/orders/{id}` | 订单详情（顾客本人或本租户管理员） |
| `GET` | `/api/orders/query?orderNo=` | 按订单号查询 |
| `GET` | `/api/orders/history` | 顾客查看自己的历史订单 |
| `PUT` | `/api/orders/{id}/status` | 更新订单状态（管理员） |

### 超级管理员专属

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| `GET` | `/api/super/overview` | 全局数据概览与最近订单 |
| `GET` | `/api/super/tenants` | 租户列表（含订单/菜品/管理员/顾客计数） |
| `POST` | `/api/super/tenants` | 创建租户，可同时创建初始租户管理员 |
| `PUT` | `/api/super/tenants/{id}` | 修改租户名称、启停、门店信息 |
| `GET` | `/api/super/tenants/{id}/qrcode` | 生成/查看租户二维码 PNG |
| `GET` | `/api/super/accounts` | 管理员账号列表，可按 `role` / `tenantId` 过滤 |
| `POST` | `/api/super/accounts` | 创建超管或租户管理员账号 |
| `PUT` | `/api/super/accounts/{id}` | 修改显示名、密码、启停 |
| `DELETE` | `/api/super/accounts/{id}` | 删除账号（禁止删除自己，保留至少一个启用超管） |

订单状态取值：`PENDING_PAYMENT`、`PAID`、`PREPARING`、`COMPLETED`、`CANCELLED`。时间字段格式保持 `yyyy-MM-dd HH:mm:ss`。

## 默认账号

账号由环境变量初始化，实际值见 `backend/.env`：

- 超级管理员：`superadmin / superadmin123`（PC 后台使用）
- 初始租户 `T001` 管理员：`tenantadmin / tenantadmin123`（小程序管理端使用）

首次部署请修改 `.env` 中的密码并重新初始化数据库。
