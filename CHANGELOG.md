# Changelog

## 未发布 — 发布前清理

- 发布形态改为源码；README 增加非官方、协议风险与账号风险声明。
- 移除 `enc:v1` 旧凭据回退，只保留 `enc:v2`。
- 补充第三方许可清单与 deb `copyright` 文件。
- 更新 NOTICE 表述与打包元数据。

## 0.1.0 — 2026-11-13

首个独立版本。

- 复用 ZCode 桌面端登录态与 ASR：ticket + WebSocket realtime。
- 全局热键按住说话，松开识别；悬浮胶囊与 10 段音量波形。
- 识别后自动粘贴到当前焦点输入框；也可手动复制。
- 本机 SQLite 历史、搜索、删除、Markdown / JSON 导出。
- 音频文件转写：wav / mp3 / flac / ogg / m4a / aac。
- 替换规则、中文 / English、热键与服务地址配置。
- 跟随系统 / 浅色 / 深色主题；UI 按 ZCode `VoiceInputControls` 设计语言实现。
- 托盘菜单；关闭窗口隐藏到托盘（无托盘时直接退出）。
- Linux deb 打包与安装验证。
