# ZCode ASR 客户端协议（Sayline 派生规格）

> 本文档仅用于互操作研究。使用该协议访问 ZCode 服务受 ZCode 用户协议约束；
> 公开传播规避客户端校验的方法可能违反该协议。

derived-from: `zai-org/ZCode` commit `c5904686`（v3.18.1），
`packages/services/src/voice-input/voiceInputService.ts`、
`packages/ui/src/voice-input/adapters/pcmCapture.ts`、
`docs/chat/composer-voice-input.md`。
服务端 `zcode-server` 仓库未公开；服务端行为以客户端代码与
`packages/desktop/test/e2e/helpers/asr-mock-server.ts` 为准。

## 1. 音频格式

- 采样率：16 000 Hz
- 声道：1
- 采样：有符号 16 位小端（`pcm_s16le`）
- 分块：1 600 samples = 100 ms = 3 200 bytes
- 尾块：停止时 flush 不足 100 ms 的样本，仍发送

Sayline 用系统录音命令直接产出该格式；ZCode 用 Web Audio + AudioWorklet
在 Chromium 内重采样并降混。

## 2. Ticket

```
POST {endpoint}/api/v1/asr/tickets
Authorization: Bearer <zcodejwttoken>
Content-Type: application/json
User-Agent: ZCode/3.18.0
HTTP-Referer: {endpoint}
X-Title: Z Code@electron
X-ZCode-App-Version: 3.18.0
X-Platform: linux-x64
X-Release-Channel: production
X-Client-Language: zh-CN | en-US
X-Client-Timezone: Asia/Shanghai
X-Os-Category: linux

{"mode":"realtime"}
```

成功响应：

```json
{"code":0,"msg":"","data":{"ticket":"<one-shot>","expires_in":60,"ws_path":"/api/v1/asr/realtime"}}
```

失败：HTTP 401 / 429 / 503，或 `data.error_code` 为
`unauthorized` / `rate_limited` / `unavailable` / `upstream_unavailable` /
`upstream_timeout` / `audio_limit_exceeded` / `empty_audio` / `audio_too_short`。

## 3. WebSocket

- `ws_path` 相对 endpoint 解析；`http:` → `ws:`，`https:` → `wss:`。
- 追加 `?ticket=<ticket>`；ticket 一次性。
- 不发送 `Origin` 头（服务端按 `allowed_origins` 拒绝浏览器来源）。
- Sayline 使用 rustls + 系统根证书直连；ZCode Host 还支持 HTTP 代理与自定义 CA。

客户端文本帧：

```json
{"type":"start","audio":{"encoding":"pcm_s16le","sample_rate":16000,"channels":1},"language":"zh-CN","options":{"punctuation":true,"itn":true}}
```

之后发送二进制 PCM16。结束：

```json
{"type":"finish"}
```

取消：

```json
{"type":"cancel"}
```

服务端事件：

| type | 字段 | 说明 |
| --- | --- | --- |
| `started` | `session_id`、`provider`、`max_audio_duration_ms` | 会话建立 |
| `result` | `text` | 当前完整文本快照，不是增量 |
| `completed` | `text`、`audio_duration_ms` | 最终文本 |
| `error` | `code`、`retryable` | 失败 |
| `cancelled` | — | 取消确认 |

`started` 之前到达的音频由客户端按序缓存；Sayline 最多缓存 200 块（20 s）。

## 4. 超时

| 阶段 | 超时 |
| --- | --- |
| ticket 请求 | 10 s |
| 等待 `started` | 8 s（服务端 start 5 s + 余量） |
| 等待 `completed` | 60 s（服务端 finish 15 s，GLM 40 s） |
| 单次最长音频 | 服务端下发 `max_audio_duration_ms`，mock 为 30 min |

## 5. 本地判定

Sayline 在发送 `finish` 前检查麦克风采集统计：

| 条件 | code |
| --- | --- |
| `peak == 0` | `silent` |
| `peak < -50 dBFS` | `too_quiet` |
| 有效音频 < 300 ms | `audio_too_short` |

命中时发送 `cancel`，不等待识别结果。

## 6. Sayline 与 ZCode 的差异

- Sayline 是独立进程，直接调用 ZCode ASR；不经过 ZCode Host。
- 录音来自 `parec` / `pw-record` / `arecord`，不是 Web Audio。
- 文件转写把解码后的 16 kHz 单声道样本按节奏送入同一 realtime 协议；
  没有批量接口。
- 当前不支持 HTTP 代理下的 WebSocket 与自定义 CA；ticket 请求复用 reqwest
  的代理环境变量，WS 直连。需要代理时请使用可直连网络。
- 识别文本不写入 ZCode 编辑器，而是显示在 Sayline 并可选自动粘贴。
