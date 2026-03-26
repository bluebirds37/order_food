# 微信点餐系统

## 项目定位

本项目的顾客端前端采用微信原生小程序开发，普通用户通过扫码直接进入点餐页面完成下单、查看订单详情和查询历史订单。

老板端提供两种管理入口：

1. 微信原生小程序老板入口，用于移动端快速查看营业状态、菜单与订单。
2. `admin-web` Web 管理后台，基于 Vue 3、Vite、Element Plus 开发，用于更高效地进行菜单维护、营业设置与订单管理。

因此，Vue 并不是顾客点餐前端，只用于老板后台管理系统，这一点已经与项目结构保持一致。

## 项目结构

- `backend`：Spring Boot 3 后端服务
- `miniprogram`：微信原生小程序，包含顾客端和老板移动端入口
- `admin-web`：老板 Web 管理后台
- `database`：数据库初始化脚本

## 技术栈

- 后端：Spring Boot 3、Spring Web、Spring Data JPA、PostgreSQL
- 顾客端前端：微信原生小程序
- 老板移动端：微信原生小程序
- 老板后台：Vue 3、Vite、Element Plus、Axios

## 数据库配置

- 地址：`localhost:15432`
- 数据库：`postgres`
- 用户名：`postgres`
- 密码：`postgres`

启动后端后会自动建表，并初始化管理员账号：

- 用户名：`admin`
- 密码：`admin`

## 启动说明

### 后端

```bash
cd backend
mvn spring-boot:run
```

访问地址：`http://localhost:8080`

### 老板 Web 管理后台

```bash
cd admin-web
npm install
npm run dev
```

访问地址：`http://localhost:5173`

### 微信小程序

1. 打开微信开发者工具。
2. 导入目录：`miniprogram`
3. 使用 `project.config.json` 编译运行。
4. 普通顾客扫码可携带桌号场景值，例如 `A01`。
5. 老板扫码可约定场景值为 `admin`，进入老板管理入口。

## 当前已实现能力

- 普通用户扫码进入点餐页
- 普通用户提交订单并查看订单详情
- 普通用户按手机号查询历史订单
- 老板在小程序中查看营业状态、菜单与订单
- 老板通过 Web 管理后台管理菜单、营业状态与订单状态