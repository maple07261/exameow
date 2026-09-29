# 录屏搜题：答案浮窗

适用于 Windows 和 macOS 桌面版。在“录屏搜题”的答案浮窗右上角点击调节图标，打开“浮窗外观”。

- 窗口背景透明度：0% 不透明，100% 完全透明；答案卡片、选项和右上角三个按钮的背景也随之变化。
- 文字透明度：0% 不透明，100% 隐藏文字及右上角三个按钮的图标；独立于背景调节。
- 答案字号：10–36 px，应用于答案、题干和选项；长文本换行，内容超出窗口时滚动查看。
- 设置立即生效，并保存到本机；关闭浮窗后重新打开仍保留。“恢复默认”恢复不透明、13 px。
- 外观设置面板保持可见。右上角三个按钮随透明度设置变化；鼠标移入按钮区域或使用键盘聚焦按钮时临时恢复可见，移出后恢复设置的透明度。小窗口中设置面板可以滚动。

浮窗使用系统级 always-on-top，失去焦点后重新设置置顶，不抢占其他应用的键盘焦点。macOS 另外启用跨 Spaces 和全屏辅助窗口行为。普通 Office / VMware 窗口属于预期覆盖范围；系统安全桌面、独占全屏程序及其他置顶窗口的优先级由操作系统决定，不能保证覆盖所有情况。

## 自用编译

需要符合 `frontend/package.json` 要求的 Node.js、pnpm、Rust，以及平台原生编译工具。Windows 使用 MSVC 编译工具链与 WebView2；macOS 使用 Xcode 命令行工具。先在项目根目录执行：

```sh
pnpm install --frozen-lockfile
```

macOS：

```sh
pnpm tauri build --bundles app --config src-tauri/tauri.local.conf.json
```

产物为 `target/release/bundle/macos/Exameow.app`，架构默认与编译机器一致。

Windows（在 Windows 终端中执行）：

```sh
pnpm tauri build --bundles nsis --config src-tauri/tauri.local.conf.json
```

安装包位于 `target/release/bundle/nsis/`。本地配置不生成需要上游私钥的更新签名，并清空官方更新地址，避免自用修改版被上游更新替换；不改变正式发布配置。自用版的手动检查更新也不可用，后续修改需重新编译安装。

## 实机验收

1. 打开录屏搜题，分别调节背景、文字和字号；关闭重开，确认值保留。
2. 背景透明度调至 100%，确认能透过浮窗看到其他应用，文字仍可独立显示；两项均为 100% 时确认可以通过按钮恢复。
3. 在 Office / VMware 与浮窗之间切换焦点、最大化应用、最小化主窗口，检查答案浮窗仍可见，键盘输入仍留在当前应用。
4. macOS 切换 Spaces / 应用全屏；Windows 测试 VMware 窗口模式与全屏模式，记录受操作系统限制的情况。
5. 使用长题干、长选项和 36 px 字号，缩小浮窗，检查换行与滚动；暂停、调整录制区域、继续和退出仍正常。

浏览器界面验证不能替代系统级置顶、原生透明窗口和真实录屏的实机验证。

## macOS 交叉编译 Windows x64

本次已在 Apple 芯片 Mac 上通过 cargo-xwin 生成 Windows x64 NSIS 安装包。需要 LLVM、LLD、NSIS、cargo-xwin 和 Rust 的 `x86_64-pc-windows-msvc` 目标；将这些工具加入当前终端 PATH 后执行：

```sh
pnpm tauri build --runner cargo-xwin --target x86_64-pc-windows-msvc --bundles nsis --config src-tauri/tauri.local.conf.json
```

产物：`target/x86_64-pc-windows-msvc/release/bundle/nsis/Exameow_1.5.1_x64-setup.exe`，旁边的 `.sha256` 文件用于核对传输完整性。已确认主程序为 Windows x86-64 PE 格式，安装包构建成功；未进行 Windows 实机运行验证。安装包未使用商业代码签名，可能显示未知发布者提示。

## 1.5.1：VMware 全屏置顶修正

Windows 答案浮窗新增原生窗口定时器，每 500 ms 通过 `SetWindowPos(HWND_TOPMOST)` 重新调整层级，使用 `SWP_NOACTIVATE` 保留其他应用的键盘焦点。窗口隐藏或最小化时不提升；关闭窗口后 Windows 自动销毁绑定的定时器。此逻辑只在 Windows 上启用。

验收环境：Exameow 安装在物理 Windows 10（A），VMware Workstation 的 Windows 10 来宾（B）全屏。退出 A 上的旧版并安装 1.5.1，启动录屏搜题后让 B 全屏，检查浮窗可见、B 中键盘输入正常；反复切换全屏、退出和重开浮窗。若仍遮挡，需要记录 VMware 版本、全屏显示设置，以及窗口模式是否正常，以进一步定位显示模式限制。

本次修正针对普通桌面窗口层级竞争；不保证覆盖独占渲染或系统安全桌面。用户已确认 1.5.1 在上述 VMware 全屏环境中可正常置顶。

