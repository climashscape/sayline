# Devlog

## 2026-11-13 — 发布前清理（源码发布）

**落了什么**

- README 增加非官方、协议风险、账号风险与“只发布源码”声明。
- NOTICE 改为“参考并独立实现”的表述；LICENSE 增加 Sayline 版权行。
- 新增 `THIRD-PARTY-NOTICES.md` 与 `packaging/copyright`；deb 安装后包含
  `/usr/share/doc/sayline/` 下两份文件。
- 移除 `enc:v1` 旧凭据回退，只保留 `enc:v2`；新增拒绝旧格式的单元测试。
- tauri identifier 改为 `io.github.climashscape.sayline`，publisher 改为
  GitHub noreply。
- 移除 `docs/method/README.md` 的个人仓库外链；提交作者改为 noreply。

**证据**

- `cargo check` 通过；`cargo test` 7 passed。
- `pnpm tauri build --bundles deb` 成功；`dpkg-deb -I` 显示 Maintainer 为
  `Sayline contributors <68630738+climashscape@users.noreply.github.com>`；产物含
  `copyright` 与 `THIRD-PARTY-NOTICES.md`。
- release 二进制在 Xvfb `:99` 下启动 8 秒未崩溃（exit 124，仅 EGL/DRI3 警告）。
- 发布前扫描无 token、私钥、个人路径与真实姓名。

**还开着什么**

- 未创建远端仓库；未发布二进制。
- ZCode 用户协议风险未消除，README 已声明。

## 2026-11-13 — UI 按 ZCode VoiceInputControls 重构

**落了什么**

- 全量重写 `impl/src/styles.css`：采用 ZCode 的语义 token（background/card/
  popover/input/border/foreground）、`text-ui-*` 字号、12 px 卡片圆角与 28 px 控件。
- 主界面改成 composer 式语音控制台：空闲是 28 px ghost 麦克风按钮，录音时
  展开为 sky 胶囊 + 10 根 3 px 波形 + 10 px 停止方块；展开 200 ms。
- 悬浮层使用同一套 `--color-voice-input-*` token。
- 历史、文件、设置统一样式：中性卡片、细边框、紧凑按钮、底部 toast。
- 新增主题设置：跟随系统 / 浅色 / 深色；主题写入 `settings.json` 并即时生效。
- 录音开始时清空上一轮结果，避免旧文本留在预览区。

**证据**

- `docs/evidence/recording-light.png`、`main.png`、`settings-light.png`、
  `settings-dark.png`、`history.png`、`file-result.png`、`overlay.png`。
- 浅色/深色切换后 `settings.json` 分别写入 `theme=light` / `dark`。
- 录音、识别、文件转写、自动粘贴功能回归通过。
- 新 deb 已装机；`/usr/bin/sayline` 显示新版界面，并再次识别
  `Front_Center.wav` 为 `Front, center.`。

## 2026-11-13 — 首版落地

**落了什么**

- 新建独立项目 `sayline/`：Tauri 2 + React 19 + Rust。
- 语音链路：ZCode 凭据解密 → ticket → WebSocket realtime → PCM16 分块 →
  completed → 替换规则 → SQLite 历史 → 可选自动粘贴。
- 全局热键按住说话、X11 悬浮胶囊、10 段音量波形。
- 历史搜索 / 导出 Markdown / JSON、文件转写、设置页、托盘菜单。
- 本地 mock 与测试目标窗口；验证证据见 `docs/agents/verification.md`。

**证据**

- `cargo check` 无警告；`cargo test` 通过。
- Mock 文件会话 64 000 bytes，麦克风会话 76 800 bytes，均 `finish`。
- 真实 ZCode ASR：`provider=volcengine`，1.5 s 音频，`completed`。
- 真实语音样本 `/usr/share/sounds/alsa/Front_Center.wav` 识别为 `Front, center.`。
- 自动粘贴：Tk Entry 收到最终文本。
- 导出：下载目录 `sayline-history-*.md`。
- deb 打包并安装：`dpkg -l sayline` 显示 0.1.0；`/usr/bin/sayline` 用内嵌前端
  再次把 `Front_Center.wav` 识别为 `Front, center.`。

**还开着什么**

- 未用真实人声测试识别质量；只验证到服务端 `completed`。
- 托盘菜单未在真实桌面验收（Xvfb 无 StatusNotifier host）。
- Wayland 不支持，未做 portal 方案。
- WebSocket 代理与自定义 CA 未实现（ticket 请求走 reqwest 代理环境变量）。
- deb 打包待跑。

**下一步**

1. `pnpm tauri build --bundles deb`，用 `dpkg-deb -I` 检查产物。
2. 在真实 Cinnamon 会话验证托盘、热键、粘贴、悬浮层透明。
3. 用一段真人语音验证中文识别与替换规则。
4. 如需要，补 WebSocket 代理支持与 Wayland portal 方案。

## 方法说明

用户选择「参考 ZCode 源码（Apache-2.0）并独立实现」路线，因此没有走路线 B 的
`private/` 提取。本项目保留 `spec/` 与 `docs/agents/loop.md` 作为验收骨架，
所有实现以本轮可重复的 mock/真机证据为准。
