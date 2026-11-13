# AGENTS.md — Sayline

> 本文件是路由，不是宪法。与代码冲突时以代码为准，并把本文件改对。
> **任何新会话先读 `HANDOFF.md`**，再按下面地图查细节。

## 0. 项目一句话

用 Rust（Tauri 2）+ React 独立重做 ZCode 桌面端语音输入，并扩展到全局热键、
自动粘贴、历史、文件转写、替换规则与浅/深色主题。代码参考 Apache-2.0 的
`zai-org/ZCode`。当前版本 `0.1.0`。发布形态为源码，不发布二进制；公开版本
与 Z.ai 无关联，README 顶部保留非官方、协议风险与账号风险声明。

## 1. 文件地图

| 要做的事 | 读哪里 |
| --- | --- |
| 接手项目、看现状/未决项 | `HANDOFF.md` |
| 看模块、状态机、事件契约 | `docs/agents/architecture.md` |
| 方法循环与完成判据 | `docs/agents/loop.md` |
| 语音协议规格 | `spec/asr-protocol.md` |
| 功能清单与验收状态 | `spec/inventory.toml` |
| 本轮验证证据 | `docs/agents/verification.md` |
| 截图证据 | `docs/evidence/` |
| 术语 | `CONTEXT.md` |
| 本地 mock / 测试工具 | `tools/README.md` |
| 最近变更 | `log/devlog.md` |

## 2. 硬约束

- 🔴 不修改 ZCode 安装目录与 `~/.zcode` 下的任何文件；只读凭据。
- 🔴 不记录、不打印、不上传 JWT、ticket、音频或识别文本到日志。
- 🔴 录音与识别结果只发往 ZCode ASR；历史只存本机 SQLite。
- 🔴 自动粘贴用参数列表调用 `xdotool`，不经 shell；识别文本只走剪贴板。
- 🔴 只支持 X11 的声明必须保留；Wayland 不得宣称可用。
- 🔴 公开产物不得宣称与 Z.ai 有关联；README 顶部必须保留非官方与风险声明。
- 🔴 只发布源码，不发布预编译二进制；构建产物由使用者自行生成。
- 🔴 悬浮层的 `set_ignore_cursor_events` 曾触发 tao panic；恢复它必须先在
  未映射窗口上做回归，或在 show 后延迟设置。
- 🟡 新增外部依赖要有明确用途；优先复用系统已有命令。
- 🟡 改协议常量时同步 `spec/asr-protocol.md` 与 `tools/asr_mock.py`。
- 🟡 改 UI 时先读 `docs/agents/architecture.md#设计系统`；语音胶囊专用 token
  不得挪作他用。
- 🟡 每次会话结束在 `log/devlog.md` 追加：落了什么、证据、还开着什么、下一步。

## 3. 构建与验证

```bash
cd impl
pnpm install
pnpm tauri dev          # 开发模式（Vite + Tauri）
pnpm tauri build --bundles deb
```

```bash
cd impl/src-tauri
cargo test
cargo check
```

- 调试二进制加载 devUrl，直接运行 `target/debug/sayline` 前必须先起 Vite；
  否则用 release 或已安装的 `/usr/bin/sayline`。
- 本地 ASR mock：`python3 tools/asr_mock.py --http-port 18080 --ws-port 18081`；
  配合 `SAYLINE_ZCODE_TOKEN=mock-token` 可跳过真实凭据。
- 详细验收脚本见 `docs/agents/verification.md`。

## 4. 容易踩的坑

- Mimosa L2 会把 `file_decode.rs` 的 `spec.channels.count()` 误报为命令注入；
  该文件没有 `Command`/`std::process`。真正的外部命令只在 `capture.rs`
  （固定录音程序名）和 `paste.rs`（固定 xdotool 参数）。
- 测试虚拟麦克风后必须恢复原默认源并卸载 null sink。
- ZCode 凭据为 `enc:v2:os:` + 系统 keyring；ZCode 升级可能改变格式。
- 主题缺失时后端补 `system`；`settings.json` 不应提交个人路径或 token。
- 悬浮层透明依赖桌面合成器；Xvfb 截图会有黑底。

## 5. 仓库卫生

- `impl/node_modules/`、`impl/dist/`、`impl/src-tauri/target/`、`gen/`、
  `.mimosa/`、`private/` 不入库。
- 发布前确认提交作者为 noreply 邮箱；上述目录不得进入发布包。
- 提交信息用英文、简短；首版提交后不要重写历史（发布前清理除外）。
- 未经用户明确要求不创建远端、不改 Git 历史。
