# Codex Token Monitor

简体中文 | [English](README.md)

Codex Token Monitor 是一个面向 Windows 10/11 的本地优先桌面工具，用于统计 Codex Desktop 与 Codex CLI 产生的 token 元数据。技术栈为 Tauri 2、React、TypeScript、Rust 和 SQLite。

> 本项目是独立的社区开源项目，与 OpenAI 无隶属、背书或官方支持关系。Codex 本地数据格式和账号用量接口可能随时变化。

## 下载与安装

从 [GitHub Releases](../../releases) 下载最新 Windows 安装包。升级前请从系统托盘完全退出旧版本。由于社区构建暂未进行商业代码签名，Windows SmartScreen 可能显示风险提示。

## 主要功能

- 自动发现 `CODEX_HOME` 或 `%USERPROFILE%\.codex`
- 增量解析 JSONL，仅提取 token 元数据
- 分类统计输入、缓存输入、输出、推理输出和总 token
- 今日、7 天、30 天用量趋势与项目匿名排行
- 当前 Session 模型、上下文容量和占用比例
- 真实账号 5 小时与 7 天额度显示，可用时每 15 分钟刷新
- 本地自定义 5 小时/7 天额度、预测与 80%/90%/95% 提醒
- Windows 系统托盘、通知与暂停同步
- 可拖动、置顶、点击穿透的透明悬浮窗，支持紧凑和详细模式
- 中英文界面、浅色/深色/跟随系统主题
- SQLite 本地存储和 CSV 导出
- 每日同步 OpenAI 官方公开模型价格并保存本地缓存
- Token 费用计算器，支持模型别名和本地价格 fallback

## 隐私边界

应用不会上传 prompt、AI 回复、源码、文件内容、本地统计或项目路径。解析器先识别元数据事件，正文类事件会被跳过，不会写入数据库。

真实账号额度功能仅向 `https://chatgpt.com/backend-api/wham/usage` 发起只读请求。`auth.json` 中的访问令牌和账号 ID 只在内存中使用，不写入数据库、CSV 或日志。关闭额度同步或接口不可用时，本地统计仍可正常工作。详见 [隐私模型](docs/privacy-model.md)。

价格同步仅向 `https://developers.openai.com/api/docs/pricing` 发起不带认证信息的公开 GET 请求，不会发送账号、Token 用量、Session 或项目数据。每次成功同步会同时更新官方页面中当前可定价的模型目录及其价格，本地内置模型仅在离线时兜底，不会混入在线目录。结果在本机缓存 24 小时；字段不完整时显示“价格不可用”，不会猜测价格。

## 开发环境

- Windows 10 1803+ 或 Windows 11
- Node.js 20+
- Rust stable MSVC 工具链
- Visual Studio C++ Build Tools 与 Windows SDK
- Microsoft Edge WebView2 Runtime

```powershell
npm.cmd install
npm.cmd run tauri dev
```

仅启动前端：

```powershell
npm.cmd run dev
```

## 测试

```powershell
npm.cmd test
cargo test --manifest-path src-tauri\Cargo.toml
powershell -ExecutionPolicy Bypass -File scripts\verify-local-only.ps1
```

## 构建安装包

```powershell
powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1
```

输出目录：

```text
src-tauri\target\release\bundle\nsis
src-tauri\target\release\bundle\msi
```

更多信息参见 [贡献指南](CONTRIBUTING.md)、[架构说明](docs/architecture.md) 和 [故障排查](docs/troubleshooting.zh-CN.md)。

## 许可证

[MIT](LICENSE)
