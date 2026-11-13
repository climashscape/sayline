# 本轮验证证据（2026-11-13）

环境：Linux X11（Cinnamon），Rust 1.98，Node 25，Xvfb `:99` 用于隔离测试。
真实 ZCode 端点测试使用本机已登录的 `zcodejwttoken`，只发送 1.5 s 正弦音，
不包含人声。

## 1. 凭据解密

- 独立 Node 脚本读取 keyring 主密钥并解密 `zcodejwttoken`，只输出长度与哈希。
- 结果：`decrypt ok length=204`。
- Sayline 内 `zcode_status` 显示「已连接」，真实端点 ticket 申请成功。

## 2. 文件转写（mock）

```bash
ffmpeg -f lavfi -i sine=frequency=440:duration=2 -ar 16000 -ac 1 -y /tmp/sayline-test.wav
python3 tools/asr_mock.py --http-port 18080 --ws-port 18081
DISPLAY=:99 SAYLINE_ZCODE_TOKEN=mock-token SAYLINE_DISABLE_TRAY=1 \
  GTK_IM_MODULE=gtk-im-context-simple XMODIFIERS=@im=none \
  ./impl/src-tauri/target/debug/sayline
```

- 文件页路径输入 `/tmp/sayline-test.wav`，点击「转写」。
- Mock 日志：`session done bytes=64000 controls=['finish']`。
- 历史库：`duration_ms=2000, source=file, title=sayline-test.wav`。
- UI 截图：`docs/evidence/file-result.png`。

## 3. 麦克风 + 全局热键（mock）

虚拟音频设备：

```bash
pactl load-module module-null-sink sink_name=sayline_test
pactl set-default-source sayline_test.monitor
paplay --device=sayline_test /tmp/sayline-test.wav
```

`xdotool keydown ctrl keydown alt keydown space` → 2.4 s → keyup。

- Mock 日志：`session done bytes=76800 controls=['finish']`。
- 历史库：`duration_ms=2400, source=mic`。
- 悬浮胶囊截图：`docs/evidence/overlay.png`。

## 4. 真实 ZCode ASR

- 端点 `https://zcode.z.ai`，1.5 s 正弦音。
- 日志：
  - `asr ticket acquired`
  - `asr started source=mic provider=volcengine`
  - `asr completed source=mic bytes=48000 duration_ms=1500 length=0`
  - `asr failed source=mic code=no_speech`（纯音没有人声，符合预期）
- 说明 ticket、WebSocket、`started`、`completed` 全链路可用。

### 4.1 真实语音样本

- 文件：`/usr/share/sounds/alsa/Front_Center.wav`（系统自带英语语音样本）。
- 结果：`asr completed source=file bytes=45696 duration_ms=1428 length=14`。
- 历史库文本：`Front, center.`。
- 说明真实语音经文件解码、16 kHz 分块、真实 ZCode ASR 后返回了正确文本。

## 5. 自动粘贴

- Tk 目标窗口 Entry 获得焦点。
- 识别后 `ENTRY=你好，这是 Sayline 的测试。`。
- 剪贴板在粘贴期间保持 X11 selection owner。

## 6. 历史导出

- 历史页点击 Markdown，导出文件写入系统下载目录。

## 7. 安装包

```bash
cd impl
pnpm tauri build --bundles deb
pkexec bash -c "dpkg -i src-tauri/target/release/bundle/deb/Sayline_0.1.0_amd64.deb"
dpkg -l sayline
```

- 产物 6.60 MiB，`Package: sayline`，依赖
  `libayatana-appindicator3-1`、`libwebkit2gtk-4.1-0`、`libgtk-3-0`。
- 已安装 `/usr/bin/sayline` 与 `/usr/share/applications/Sayline.desktop`。
- 安装后的二进制显示新版语音控制台，并再次把 `Front_Center.wav` 识别为
  `Front, center.`（1428 ms）。

## 8. UI 设计系统

参照 `zai-org/ZCode` 的 `packages/ui/src/voice-input/ui/VoiceInputControls.tsx`
与 `packages/ui/src/styles.css`：

- 录音胶囊：`--color-voice-input-surface` / `--color-voice-input-wave`
  （浅色 `#e0f2fe` / `#0284c7`，深色 `#082f49` / `#7dd3fc`）。
- 波形：10 根、3 px 宽、4 px 间距、高度 3–14 px，对应 -50 到 -10 dBFS。
- 控件：28 px 圆形 ghost 按钮；停止方块 10 px、圆角 3 px；pill 展开 200 ms。
- 卡片：12 px 圆角；控件 6–8 px 圆角；按钮高 28 px；正文 14 px，辅文 12/10 px。
- 主题：跟随系统 / 浅色 / 深色，选中后写入 `settings.json` 并立即生效。

截图：

- `docs/evidence/recording-light.png`：浅色录音胶囊。
- `docs/evidence/main.png` / `main-dark.png`：浅色 / 深色结果页。
- `docs/evidence/settings-light.png` / `settings-dark.png`：两套主题。
- `docs/evidence/history.png` / `file-result.png`：历史与文件转写。
- `docs/evidence/overlay.png`：悬浮胶囊；Xvfb 无合成器，黑色边缘来自测试环境。

验证结果：浅色、深色均通过截图检查；主题切换写入配置并即时生效；安装后的
二进制使用同一套样式。

## 9. 单元检查

```bash
cd impl/src-tauri
cargo check    # 通过，无警告
cargo test     # 见下
```

测试覆盖：替换规则长词优先、16 kHz 时长换算、静音/过轻分类、WS URL。

## 10. 发布前清理（2026-11-13）

- `cargo check`：通过。
- `cargo test`：7 passed，0 failed；新增 `legacy_credentials_are_rejected`。
- `pnpm tauri build --bundles deb`：成功，产物 6.59 MiB。
- `dpkg-deb -I`：Maintainer 为
  `Sayline contributors <68630738+climashscape@users.noreply.github.com>`。
- `dpkg-deb -c`：包含 `/usr/share/doc/sayline/copyright` 与
  `/usr/share/doc/sayline/THIRD-PARTY-NOTICES.md`。
- release 二进制在 Xvfb `:99` 下启动 8 秒未崩溃（exit 124，仅 EGL/DRI3 警告）。
- 未安装新包；本次只改发布元数据与凭据兼容分支，不影响已验证的语音链路。

## 未覆盖

- 托盘菜单在真实桌面上的可见性（Xvfb 无 StatusNotifier host）。
- 悬浮胶囊在真实合成器下的透明效果（Xvfb 无合成器）。
- Wayland 下的热键与模拟粘贴。
- 原生文件选择框在无窗口管理器环境下的行为；已提供路径输入回退。
- 带 HTTP 代理的 WebSocket 与自定义 CA。
