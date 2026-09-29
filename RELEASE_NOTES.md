# Exameow 1.5.1 桌面自用版

基于上游 `70e0d70f3bbe17aeac2ebd4865b6435f8b322519` 的录屏搜题浮窗修改版。

## 更新内容

- 可分别调整窗口背景与文字透明度（0–100%），并调整字号（10–36 px）。
- 右上角三个按钮的背景和图标跟随透明度；悬停或键盘聚焦时恢复可见。
- Windows 使用每 500 ms 的原生置顶维护，不主动夺取键盘焦点；用户已确认可覆盖 VMware Workstation 中全屏的 Windows 10 虚拟机。
- macOS 支持跨 Spaces 和全屏辅助窗口；设置保存在本机。

## 下载与构建

- Windows x64：`Exameow_1.5.1_x64-setup.exe`（NSIS 安装程序）。
- macOS Apple Silicon：`Exameow_1.5.1_macos_arm64.zip`（解压得到 Exameow.app）。本次不提供 Intel Mac 包。
- `SHA256SUMS.txt` 用于核对下载完整性。安装包为此前已交付的构建结果，未重新签发。
- macOS 采用本地临时签名，未公证；Windows 未使用商业代码签名。
- 构建命令与使用方法见 `BUILD_LOCAL.md`，使用 `src-tauri/tauri.local.conf.json` 关闭上游自动更新。

本发布仅提供桌面端安装文件，不发布移动端 OTA、Docker 或 Cloudflare 服务。标签采用 `custom-v1.5.1`，避免触发上游的 `v*` 多平台发布流水线。

## 源码与验证记录

1.5.1 在开发时未单独提交；此标签按当时的修改范围从本地开发记录整理，移除了之后 1.6.0 的关闭保护和全局快捷键功能。安装文件保留此前交付且已测试的版本。此源码快照未重新生成安装文件，不声明二进制可逐字节复现。

此前 Windows / macOS 构建成功；Windows VMware 全屏置顶已由用户实测通过。
