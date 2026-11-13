# 测试与开发工具

## asr_mock.py

本地 ASR 替身，实现 ticket 端点与 realtime WebSocket。不会发出任何网络请求。

```bash
python3 tools/asr_mock.py --http-port 18080 --ws-port 18081
```

- 依赖：Python 3.10+、`websockets`（本机为 10.4）。
- 在 Sayline 设置里把服务地址改为 `http://127.0.0.1:18080`。
- 用 `SAYLINE_ZCODE_TOKEN=mock-token` 启动应用可跳过 ZCode 凭据读取。
- `GET http://127.0.0.1:18080/__state` 查看 ticket 请求与会话字节数。

## paste_target.py

一次性 Tk 窗口，用来验证自动粘贴。

```bash
DISPLAY=:99 python3 tools/paste_target.py
```

窗口里的 Entry 会持续获得焦点；9 秒后打印 `ENTRY=...` 并退出。

## 合成测试音频

```bash
ffmpeg -f lavfi -i sine=frequency=440:duration=2 -ar 16000 -ac 1 -y /tmp/sayline-test.wav
```

纯音没有语音，真实 ZCode ASR 会返回空文本；它适合验证协议而不是识别质量。

## 虚拟麦克风

```bash
pactl load-module module-null-sink sink_name=sayline_test
pactl set-default-source sayline_test.monitor
paplay --device=sayline_test /tmp/sayline-test.wav
```

测试结束恢复原默认源并卸载模块。
