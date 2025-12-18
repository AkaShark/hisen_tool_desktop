# Hisen Desk

一个使用 Tauri + React + TypeScript 构建的跨平台（macOS/Windows）桌面应用，用于：

- 网络测试（外网 IP、HTTP 延迟、近似下载速度）
- 系统信息展示（OS、CPU、内存、网络接口流量）
- 音频设备枚举（输入/输出）
- 摄像头设备枚举

## 运行前置

- Node.js ≥ 18（建议 20）
- Rust 工具链（rustup）
  - macOS: `xcode-select --install` 安装命令行工具
  - Windows: 安装 VS Build Tools（含 C++ 生成工具）

## 开发运行

```bash
# 安装依赖
npm install

# 启动前端（Vite）开发服务器
npm run dev

# 另外再开一个终端，启动 Tauri Dev（会打开桌面窗口）
npm run tauri:dev
```

也可以一次由 Tauri 拉起前端（更简便，但首次构建较慢）：

```bash
npm run tauri:dev
```

## 打包

```bash
npm run build         # 构建前端
npm run tauri:build   # 构建桌面安装包（macOS/Windows）
```

## 主要文件

- 前端入口: [index.html](index.html), [src/main.tsx](src/main.tsx), [src/App.tsx](src/App.tsx)
- 样式: [src/styles.css](src/styles.css)
- Tauri 配置: [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)
- Rust 后端: [src-tauri/src/main.rs](src-tauri/src/main.rs)

## 说明

- 网络测试采用 HTTP 方式估算延迟与下载速率，避免 ICMP 权限问题。下载速率通过下载约 3MB 的测试数据计算，结果受网络波动与测试端点影响，仅供参考。
- 音频设备通过 `cpal` 枚举，摄像头通过 `nokhwa` 枚举，应在大多数 macOS/Windows 设备上可用。
