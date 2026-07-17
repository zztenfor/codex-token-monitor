# 故障排查

## 启动后出现终端窗口

请确认使用的是 `src-tauri/target/release/bundle` 或 GitHub Releases 中的安装包，而不是开发命令。发布构建使用 Windows GUI 子系统，不应保留命令行窗口。

## 页面未响应

先从系统托盘退出应用后重新启动。若问题持续，确认 Microsoft Edge WebView2 Runtime 已安装，并在 issue 中提供应用版本、Windows 版本和不含敏感信息的错误描述。

## 没有检测到使用数据

1. 确认 Codex Desktop 或 CLI 已产生 Session。
2. 在设置中检查 Codex 路径，默认是 `%USERPROFILE%\.codex`。
3. 不要选择源码目录；数据源应指向 Codex Home。
4. 点击手动同步并等待当前 JSONL 文件完成写入。

## 真实额度不可用

- “请先登录 Codex”：没有发现 `%USERPROFILE%\.codex\auth.json` 或缺少所需字段。
- “身份验证已过期”：重新登录 Codex 后再刷新。
- “连接失败”：检查网络、代理和防火墙。
- “额度不可用”：账号接口没有返回兼容的 5 小时或 7 天窗口。

真实额度接口不是稳定的公开 API，可能因 Codex 更新发生变化。应用不会使用估算值冒充真实额度。

## 30 天图表恢复成 7 天

此问题已在 `0.5.1` 修复。升级后范围选择会保存在本机，旧的异步刷新结果也不会覆盖当前选择。

## 如何安全提交问题

不要上传 `auth.json`、访问令牌、真实 JSONL、聊天正文、源码或数据库。请使用合成元数据复现，并隐藏 Session ID、账号 ID 和项目路径。
