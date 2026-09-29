# Exameow 1.6.0 桌面自用版

基于上游 `70e0d70f3bbe17aeac2ebd4865b6435f8b322519` 的录屏搜题浮窗修改版。

## 更新内容

包含 1.5.1 的全部浮窗功能，并新增：

- 防误关闭：默认首次点击后 3 秒内点击叉号 3 次；可设 2–5 次、1–10 秒。超时清零。
- 拦截普通系统关闭请求，提示按配置的连续点击操作关闭；结束录屏及退出应用仍正常。
- 可自定义全局快捷键，切换浮窗置顶／置后。默认 Windows 为 Ctrl+Shift+F8，macOS 为 Command+Shift+F8。
- 快捷键与关闭保护设置位于“浮窗外观”面板；注册冲突时保留原有效快捷键。
- Windows 置后时暂停置顶定时器，恢复置顶时重新启动；面板也可切换层级。

## 下载与构建

- Windows x64：`Exameow_1.6.0_x64-setup.exe`（NSIS 安装程序）。
- macOS Apple Silicon：`Exameow_1.6.0_macos_arm64.zip`（解压得到 Exameow.app）。本次不提供 Intel Mac 包。
- `SHA256SUMS.txt` 用于核对下载完整性。安装包为此前已交付的构建结果，未重新签发。
- macOS 采用本地临时签名，未公证；Windows 未使用商业代码签名。
- 构建命令与使用方法见 `BUILD_LOCAL.md`，使用 `src-tauri/tauri.local.conf.json` 关闭上游自动更新。

本发布仅提供桌面端安装文件，不发布移动端 OTA、Docker 或 Cloudflare 服务。标签采用 `custom-v1.6.0`，避免触发上游的 `v*` 多平台发布流水线。

## 验证记录

源码为 1.6.0 构建时的本地工作区。Windows / macOS 构建成功，前端类型检查通过；浏览器交互验证覆盖关闭次数、超时清零、快捷键保存与失败保留、持久化、层级切换和小窗口滚动。浏览器验证模拟原生 IPC，原生全局快捷键及 VMware 独占键盘下的响应仍需实机验收。
