# 领域词汇

| 词 | 含义 | 不要叫 |
| --- | --- | --- |
| attempt | 一次录音或一次文件转写尝试，用 UUID 隔离；旧 attempt 的事件直接丢弃 | session、task |
| ticket | ZCode ASR 的一次性连接凭证，`POST /api/v1/asr/tickets` 获取 | token、key |
| JWT / zcodejwttoken | ZCode 登录态，用于申请 ticket；Sayline 只读 | 密码 |
| capture | 从麦克风或系统录音命令取得 16 kHz 单声道 PCM16 的过程 | 录音文件 |
| source | 事件来源：`mic` 或 `file` | type |
| 语音文字 | 识别过程中写入目标编辑器的临时文本；Sayline 在独立工具里直接显示并粘贴 | 草稿 |
| 替换规则 | 最终文本的后处理规则，长词优先 | 热词（服务端热词未接入） |
| chunk | 100 ms、1600 samples、3200 bytes 的 PCM 块 | buffer、frame |

## 外部名词

- ZCode：`zai-org/ZCode`，提供登录态与 ASR 协议。
- zcode-server：ASR 服务端，仓库未公开；协议以客户端和 E2E mock 为准。
- parec / pw-record / arecord：系统录音命令，按顺序探测。
