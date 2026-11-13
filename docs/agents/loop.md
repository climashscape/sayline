# 净室循环执行版（六门 + 覆盖规则）

> 把方法论变成可执行程序。孵化/接续项目时复制到 `<项目>/docs/agents/loop.md`。
>
> **本项目适配**：用户选择“改造 ZCode 源码（Apache-2.0）”路线，没有
> `.parity-floor` / `.ratchet.json` / `impl/state.json`。完成判据以
> `spec/inventory.toml` + `docs/agents/verification.md` 的真机/mock 证据为准；
> 下文提到这些文件时按此替代。

## 覆盖规则（优先于当前计划，直到证据推翻）

- **R1 规格先行**：任何功能先有 inventory id + 规格（或路线 B 提取记录），
  再写代码。歧义标 `[需要澄清]` 并向用户升级，不许猜。
- **R2 oracle 决定**：功能完成的判据是黄金样本/语料/parity 证据通过，
  不是「代码看起来对」。
- **R3 棘轮不降**：`.parity-floor` 与 `.ratchet.json` 只升不降；任何让数字
  变差的改动都是错的。
- **R4 两振出局**：同一形态的失败出现两次 = 停下重读规格换模型，
  不是继续调参数。
- **R5 路线纪律**：路线 B 的 `private/` 内容永不离开本机；派生规格带
  `derived-from` 头。
- **R6 接线 ≠ 行为**：`impl/state.json` 只在行为差分通过后更新；禁止预填
  未实现的功能 id。

## 六门程序

| 门 | 内容 | 完成定义 |
|---|---|---|
| G0 分类 | 行为可自动比对？oracle 可建？ | 一页写明目标/参照版本/oracle 策略；「人工评审」= 停 |
| G1 法务 | 净室政策 + 路线决定 + ATTRIBUTION | 政策提交并链接进 AGENTS.md，先于任何实现代码 |
| G2 规格 | inventory + 行为记录 + EARS 规格 | `inventory_parity.py check` 可运行且通过（0% + 地板 0.00 起步） |
| G3 oracle | 黄金样本/语料/预算接线 | 故意弄坏一个行为，检查几分钟内红并指名失败的比对 |
| G3.5 布局 | 首个功能落地时定义 `impl/` 目录布局并写进 devlog | 布局图入 devlog，之后并行代理按此分权 |
| G4 收尾 | 每落一功能 | state 更新 → 地板上调 → devlog 一条 |
| G5 汇报 | 诚实口径 | 「落了什么+测量数字」「还开着什么」；门没全过 = 检查点不是完成 |

## 排障

oracle 变红先走 `diagnosing-bugs` 类排查；失败模式速查见 SKILL.md
（命中一行 = 停止信号，先读对策再动手）。
