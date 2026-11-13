# HANDOFF — Sayline 0.1.0

> 给下一个接手会话的最短上下文。先读本文件，再按 `AGENTS.md` 的文件地图查细节。

## 当前状态

- 独立 Linux 桌面语音输入工具，代码在 `impl/`，Tauri 2 + React 19 + Rust。
- 发布形态：只发布源码，不发布二进制；README 顶部有非官方、协议风险与账号风险声明。
- 已实现并通过真机验证：ZCode 登录态复用、麦克风录音、真实 ZCode ASR、全局热键、
  悬浮胶囊、自动粘贴、历史/导出、文件转写、替换规则、浅/深色主题、托盘菜单代码。
- 已打出 deb 并安装到本机：`/usr/bin/sayline`，版本 `0.1.0`。
- 安装后的二进制用真实语音样本验证过：`Front, center.`（1428 ms）。
- UI 已按 ZCode `packages/ui/src/voice-input/ui/VoiceInputControls.tsx` 的设计
  语言重做；证据在 `docs/evidence/`。
- Git 仓库在本次交接时初始化，首版提交即当前工作区。

## 一分钟跑起来

```bash
cd impl
pnpm install
pnpm tauri dev          # 开发；会启动 Vite
# 或直接运行已安装版本
sayline
```

首次使用要求本机 ZCode 桌面端已登录。配置在 `~/.config/sayline/settings.json`，
历史在 `~/.local/share/sayline/history.db`。

## 关键事实

- **不要改 ZCode 安装和 `~/.zcode`**。只读 `~/.zcode/v2/credentials.json`，
  通过系统 keyring 解密 `zcodejwttoken`。
- 录音用系统命令 `parec` → `pw-record` → `arecord` 顺序探测，直接产出
  16 kHz 单声道 PCM16；不是 Web Audio。
- ASR 是 ZCode 的 ticket + WebSocket realtime 协议；完整规格见
  `spec/asr-protocol.md`。真实服务返回 `provider=volcengine`。
- 事件统一走 Tauri 事件名 `"sayline"`，payload 以 `event` 字段区分：
  `state` / `level` / `partial` / `final` / `error` / `progress`。
- 主题事件名 `"sayline-theme"`，payload 为 `light` 或 `dark`。
- 麦克风与文件转写共用同一 ASR 会话实现；`source` 为 `mic` 或 `file`。
- 不保存音频；日志不写识别文本、音频、ticket、JWT。

## 已验收

| 项目 | 证据 |
| --- | --- |
| 真实 ZCode ticket + WS | `asr ticket acquired` / `provider=volcengine` |
| 真实语音识别 | `Front_Center.wav` → `Front, center.` |
| 麦克风链路 | mock 76 800 bytes + `finish` |
| 文件转写 | mock 64 000 bytes + `finish` |
| 自动粘贴 | Tk Entry 收到最终文本 |
| 历史导出 | 下载目录 Markdown 文件 |
| 浅/深色与语音胶囊 | `docs/evidence/recording-light.png`、`settings-dark.png` |
| 安装包 | `dpkg -l sayline` = 0.1.0；安装后真实语音复测通过 |

完整记录：`docs/agents/verification.md`。

## 还开着的问题

1. **Wayland 未支持**：全局热键与 `xdotool` 模拟粘贴是 X11 路径。
2. **WS 代理/自定义 CA 未支持**：ticket 走 reqwest 环境代理，WebSocket 直连。
3. **托盘未在真实 Cinnamon 会话验收**：Xvfb 没有 StatusNotifier host；
   托盘失败时应用会在关闭窗口时直接退出，不会变成隐形进程。
4. **悬浮层透明依赖合成器**：Xvfb 截图有黑底；真实桌面需要合成器。
5. **悬浮层没有 click-through**：为避免 tao 在未实现窗口上
   `set_ignore_cursor_events` 触发 panic，当前只靠 CSS `pointer-events:none`；
   若后续要严格不吃点击，需要在窗口 show 后延迟设置并做回归。
6. **原生文件选择框在无窗口管理器的 Xvfb 下不稳定**：文件页保留路径输入回退。
7. **真实中文语音未测**：真实端点只用英语系统样本验证过。

## 下一步建议

1. 在真实 Cinnamon 会话跑一遍托盘、热键、悬浮层透明、自动粘贴。
2. 用一段中文真人语音验证识别与替换规则。
3. 如需要，给 WebSocket 加代理/自定义 CA，或改用支持 proxy 的 transport。
4. Wayland 方案：XDG portal global shortcuts + `wtype`/`ydotool` 或 libei。
5. 可选：开机自启、单实例锁、热键冲突提示。

## 容易踩的坑

- **调试二进制加载 devUrl**：直接运行 `target/debug/sayline` 会去连
  `http://localhost:1420`，必须先 `pnpm dev` 或直接用 release/安装版。
- **Mimosa L2 误报**：`file_decode.rs:63` 被报“命令注入”，实际是
  `spec.channels.count()`；该文件没有 `Command`/`std::process`。真正的
  外部命令只在 `capture.rs`（固定录音程序名）和 `paste.rs`（固定 xdotool 参数）。
- **录音默认源**：测试虚拟麦克风后要恢复 `pactl get-default-source` 原值并
  卸载 null sink，否则用户真实录音会录到测试音。
- **ZCode 凭据格式**：`enc:v2:os:` + keyring；ZCode 升级后若格式变化，
  解密会失败，先重启/重登 ZCode。
- **主题**：`settings.json` 的 `theme` 为 `system`/`light`/`dark`；缺失时
  后端会补 `system`。
