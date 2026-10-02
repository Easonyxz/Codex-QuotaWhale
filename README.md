# Codex-QuotaWhale

住在 Windows 桌面的 Codex 额度小鲸鱼。基于 **Tauri 2 + Rust + 原生 HTML/CSS/JavaScript**，由 **GPT-6 Astra 构建**。

![默认鲸鱼形象](src/assets/whale.svg)

## 功能

- 透明、无边框、始终置顶，可拖动并记住位置。
- 底部常驻周额度条；悬停展开剩余额度和重置时间，不适用的 5h 自动隐藏。
- 每分钟自动刷新，也可以手动刷新。周额度 ≤25% 变橙，≤10% 变红，不发送通知。
- 右键菜单及系统托盘：显示、隐藏、刷新、缩放、开机自启动和退出。
- 75%、100%、125%、150% 四档缩放，重启后恢复选择。
- 防重复启动；外接屏移除后恢复到当前屏幕可见区域。
- 悬停 220 毫秒展开、离开 350 毫秒收起；收起后的上方透明区域不拦截鼠标。

## 使用条件

Windows x64，需要 Microsoft Edge WebView2，以及已使用 ChatGPT 账户登录的本机 Codex。当前源码以 Windows 为目标，尚未适配 macOS/Linux。

启动后拖动鲸鱼移动。气泡里的 × 会隐藏桌宠；左击托盘图标可以找回，右键菜单的“退出”才会结束程序。开机自启动需要手动勾选。移动可执行文件后，应重新关闭并开启自启动以更新路径。

额度来自客户端内部接口，可能随服务变化而需要适配。如果账户未返回 5h，界面会隐藏对应行；网络错误时保留最近成功数据并标记状态。

## 从源码构建

推荐安装 Node.js LTS、Rust MSVC 工具链、Visual Studio C++ Build Tools（含 Windows SDK）和 WebView2。在项目目录的 PowerShell 中执行：

```powershell
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
npm ci
npm run build
```

程序生成于 `src-tauri/target/release/codex-pet.exe`。开发运行使用 `npm run dev`。

本项目也已在 Rust GNU + MSYS2 UCRT64 环境完成构建验证；`rust-toolchain.toml` 保留该工具链。使用 GNU 时，需要将 MSYS2 UCRT64 的 `bin` 目录加入 PATH。中文辅助脚本中的 MSYS2 路径是本机默认值，其他电脑应按安装位置调整。`构建.ps1` 会整理 `release/` 目录；GNU 版分发时需要把 `WebView2Loader.dll` 与程序放在一起。

```powershell
npm test
cargo test --manifest-path src-tauri/Cargo.toml
```

测试覆盖额度解析、显示格式、颜色阈值、悬停行为和屏幕坐标恢复。真实启动、额度查询、重复启动及重启位置恢复已在 Windows 本机验证；没有模拟真实拔插显示器的端到端测试。

## 登录信息与隐私

从 `CODEX_HOME/auth.json` 读取登录信息；未设置时使用 `%USERPROFILE%/.codex/auth.json`。登录令牌只在 Rust 后端使用，仅发送至 `https://chatgpt.com/backend-api/wham/usage`，不传给界面，不写入日志，不复制或修改 Codex 登录文件。窗口位置和缩放保存在本机应用配置目录。

仓库不包含账户登录信息、额度记录、本机配置或编译产物。不要上传或转发自己的 `auth.json`。本项目不是 OpenAI 官方产品。

## 灵感来源与致谢

- [MeteorNOX / DeepSeek-Balance-Whale-Widget](https://github.com/MeteorNOX/DeepSeek-Balance-Whale-Widget)：鲸鱼桌宠、额度气泡及交互风格的灵感来源。
- [imwushuai-maker / quota-float](https://github.com/imwushuai-maker/quota-float)：Codex/ChatGPT 额度读取方式与轻量桌面工具的参考实现。相关 MIT 许可保留于 [licenses/quota-float-MIT.txt](licenses/quota-float-MIT.txt)。
- **由 GPT-6 Astra 构建**，Easonyxz 提出需求、迭代与维护。

参考提交分别为 `49d688d46673fbf4221b458afc943276c68a0839` 和 `401923cc6c897fba99d3f9578985900748401d43`。

## 美术素材与许可

公开仓库自带原创简易鲸鱼 `src/assets/whale.svg` 及由其生成的应用图标，与本项目代码采用 MIT 许可。

本机原型曾使用 DeepSeek-Balance-Whale-Widget 的 `DSniang1.png`。上游说明该美术素材不在其代码 MIT 许可范围内，因此本仓库不再分发该图片、衍生图标或包含它的二进制包。上游说明保留于 [licenses/whale-PROVENANCE.md](licenses/whale-PROVENANCE.md)。如需自定义本机形象，可将自己拥有使用权限的图片放到 `src/assets/whale.png`；此文件默认被 Git 忽略，存在时优先显示。对外分发编译包前也需要确保嵌入素材具有相应授权。

## 代码结构

| 文件 | 职责 |
| --- | --- |
| `src/app.js` | 额度显示与刷新 |
| `src/desktop.js` | 悬停延迟、窗口鼠标区域 |
| `src-tauri/src/quota.rs` | 本机登录信息读取和额度解析 |
| `src-tauri/src/menu.rs` | 托盘、菜单、缩放、自启动 |
| `src-tauri/src/desktop.rs` | 位置记忆、多屏恢复、Windows 窗口区域 |

[MIT License](LICENSE)
