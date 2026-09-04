# RFC-032: Proof-gated CoHDL Agent Harness（从硬件意图到可制造 PCB）

**Status:** Proposed
**Date:** 2026-09-05
**Scope:** 位于 CoHDL compiler 之外的 partner-layer agent harness；不修改 CoHDL 语言语法，不扩大现有四条 residual DRC
**Initial target:** 常规 MCU/IoT 裸板，JLCPCB 标准双层裸板工艺 Profile

## Problem

CoHDL 已通过演示展示一个重要但较窄的闭环：自然语言需求进入模型，模型生成 `.cohdl`，编译器检查语法、名称、单位、trait、required pin、网络与四条 residual DRC，失败时把结构化诊断反馈给模型，最终生成忠实的 KiCad netlist 与 BOM。仓库中的 `harness/repair_loop.py` 正是这一 generate → check → repair 演示。

但目标已经前移：agent harness 的终点不再是“原理图可编译”，而是“一块完整 PCB 可以进入裸板制造交付”。当前原生 `.kicad_pcb` emitter 是 layout starting point：生成板框、已放置或暂存的 net-bound footprint，以及 courtyard、silkscreen、mount-hole 等已有几何；它明确不生成走线、铜皮或用户 PCB 设计规则。因此，现有 `cohdl build` 的 clean verdict 不能推出以下事实：元件位于板框内、courtyard 不冲突、网络全部布通、不同网络铜图形没有短路、线宽/间距/钻孔符合工厂能力，或最终交付文件与被检查的中间表示一致。

单纯增强 prompt 不能关闭这个差距。模型仍会出现坐标失误、错误层、漏布网络、局部短路、违反制造规则、修复一个错误又引入另一个错误，以及把“工具没有检查”误报为“检查通过”。直接让模型操作 KiCad GUI 还会加入不可审计的隐藏状态。真正需要的是一个 harness：把模型限制为候选方案搜索器，把每个正确性主张交给确定性 oracle，并要求最终交付携带可重放的证据。

服务对象是生成候选的 AI author、批准意图与例外的硬件 reviewer、维护 schema/oracle 的工具开发者，以及核对制造包的交付人员。

## Goals

- 从一个经过确认的 PCB Job Contract 出发，生成完整的双层 `.kicad_pcb`：合法放置、完成布线、必要的铜皮、board outline、footprint、网络与制造设置均存在，并交付可下单的裸板制造包。
- 将 agent 定义为**不可信候选生成器**。模型、placement backend 或 routing backend 的成功返回永远不是最终 verdict。
- 建立不可跳级的 verdict ladder：输入合约 → CoHDL logical verdict → placement → routing → physical verification → KiCad DRC → factory Profile DFM → Lean proof → artifact binding → human gates。
- 首版只绑定一个版本化裸板制造工艺，canonical identity 为 `{id: "jlcpcb-standard-2l", version: 1, sha256: "<64 lowercase hex>"}`，显示名派生为 `jlcpcb-standard-2l-v1`。规则更新必须创建新版本，历史结果永不随网页变化而改变。
- 产出可复现、可审计的证据包。对一个固定候选板，所有 validator verdict、规范化 IR、报告和 proof manifest 必须确定；不承诺远程生成模型两次搜索出相同候选。
- 保持 CoHDL Rust compiler pipeline 与 emitters 的零外部依赖约束。Harness、KiCad adapter 和 Lean verifier 均作为独立工具存在。

Constitution priority 映射：正确性/gradeability（1）、AI-generatability（2）、可审查性与信任（3）、生态输出忠实度（5）；功能广度（7）让位。

## Non-goals

- **不证明 LLM、placement 算法或 routing 算法本身正确。** 它们可以任意搜索；只有候选输出需要通过独立检查。
- **不把 routing 语法加入 `.cohdl`。** CoHDL 继续是逻辑设计和显式布局约束的 source of truth；最终 KiCad board 是物理候选，Routed Board IR 是从实际候选解析出的验证投影。
- **不把 physical checks 加成 `src/drc.rs` 的第五条规则。** RFC-004 的四条 residual DRC 保持封闭；物理制造验证是新的、后置的 verdict rung。
- **不承诺“首次上电必然工作”。** 未建模的模拟行为、器件模型误差、EMI、热、机械公差组合、固件、焊接质量与工厂过程能力不可能由本 RFC 凭空证明。
- **不支持首版高级板型。** BGA、HDI、盲埋孔、microvia、via-in-pad、DDR、RF matching、天线、市电、刚挠结合、高铜厚、受控阻抗保证、panelization 和 footprint 内部开窗（`docs/provisional-syntax.md` 的 `window`，尚无 Accepted RFC）均排除。
- **不支持首版多工厂或用户自由编辑工艺参数。** Profile 是审核过的整体，不能按字段混搭。
- **不声明首版 PCBA-ready。** BOM 与 placement 可以作为辅助文件输出，但 assembly tier、DNP、CPL、fiducial、工艺边和贴装能力属于单独 Assembly Profile RFC。
- **不构建产品 GUI。** 首版是 CLI/脚本和稳定 JSON 协议；Explorer 或 KiCad 只作为审查视图。
- **不把厂商网页声明当成定理。** Lean 只能证明候选满足所选 Profile；Profile 与现实工厂能力一致是带来源、日期和版本的外部假设。

## Design

### 1. System boundary and trust model

Harness 的规范命令名为 `cohdl-agent`。它是 CoHDL 的 partner-layer 工具，调用已发布的 `cohdl` binary、一个或多个候选生成 backend、KiCad CLI、独立 physical verifier 和独立 Lean verifier；它不是语言编译器、不是 compiler 内置 place-and-route engine，也不把铜线写回 `.cohdl`。它不得链接进 compiler crate，也不得让 compiler pipeline 使用 Lean、KiCad、LLM SDK 或网络库。若未来要把 placement/routing 变成 `cohdl` 主产品自身的职责，必须先通过 Constitution 的 Goal Change Proposal，而不是借本 RFC 静默扩大边界。

系统按信任角色分层：

| 组件 | 角色 | 默认信任 |
|---|---|---|
| 人类确认的 Job Contract / CoHDL source / DXF / lock files | 输入事实 | 被审查并以哈希冻结 |
| LLM、placement backend、routing backend | 候选搜索 | 不可信 |
| pinned CoHDL compiler + `proof_ir` emitter | source → LogicalIR oracle | **v1 逻辑 TCB**；证书只声称相对于该投影成立 |
| Routed Board parser + physical verifier | 实际板检查 | 可信实现；结果由 Lean predicate 再检查 |
| KiCad CLI DRC/export | 独立生态交叉检查 | 外部可信工具，版本固定 |
| export canonicalizer/parser/comparator | 制造输出语义核对 | v1 artifact TCB，版本与 hash 固定 |
| Fab Profile | 制造假设 | 版本化外部事实，不是数学定理 |
| `scope-classification.json`（schematic reviewer 签署） | 范围假设：无 RF/天线/市电等 v1 排除特征 | 人工声明的外部假设；board class 不能机械推出 |
| Lean kernel + pinned formal definitions/parser | 证明检查 | 最小形式化可信基；固定 imports 和 axiom policy |
| Harness orchestrator | 门禁与证据装配 | 必须可重放；不能自行创造 pass |

威胁模型包含无意错误和带有任意内容的模型输出：语法错误、错误 part、错误坐标、非法层、越界元件、未布网络、短路、违反间距、伪造工具成功文本、陈旧候选、修改锁定连接器，以及从文档或模型回复中诱导 shell/network 操作。Agent 不持有 verdict 权限，不直接写最终目录，不拥有任意 shell 工具，也不能修改 Profile 或 proof checker。v1 **不声称**形式化证明 `.cohdl` 编译器实现的普遍 soundness；它信任被哈希固定的 compiler 与 canonical `proof_ir` emitter，并让 Lean 从该投影开始重查最终板。只有 source → IR 形式化证明落地后，compiler 才能从这部分 TCB 中移除。

### 2. PCB Job Contract

每次运行从 `job.json` 开始。它是 orchestration contract，不是第二套 netlist。器件、pin、net、part、footprint 与可表达的物理意图仍来自 `.cohdl`；板框仍来自被 CoHDL 引用的 DXF；依赖身份仍来自 `cohdl.toml` 与 `cohdl.lock`。Job 只冻结项目入口、能力范围、不可修改对象、制造 Profile、运行预算和人工义务。

Schema v1 的规范形状：

```json
{
  "schema_version": 1,
  "project": "./board",
  "design": "SensorNode",
  "intent": "ESP32-S3 sensor node with USB-C, microphone and status LED",
  "review_authority": "acme-hardware",
  "board_class": {"id": "mcu_iot_control", "version": 1, "sha256": "<64 lowercase hex>"},
  "fab_profile": {
    "id": "jlcpcb-standard-2l",
    "version": 1,
    "sha256": "<64 lowercase hex>"
  },
  "locked_instances": ["SensorNode::usb", "SensorNode::mount_1", "SensorNode::mount_2"],
  "allowed_sides": ["top", "bottom"],
  "obligations": [
    {
      "id": "usb2-length-skew",
      "kind": "diff_pair_length_skew",
      "params": {"pair": ["USB_DP", "USB_DM"]},
      "evidence": "formal",
      "owner": "harness"
    },
    {
      "id": "usb2-impedance",
      "kind": "impedance_process_review",
      "params": {"target": {"kind": "diff_pair", "nets": ["USB_DP", "USB_DM"]}, "tolerance": "10%"},
      "evidence": "review",
      "owner": "hardware_reviewer"
    }
  ],
  "limits": {
    "max_attempts": 5,
    "max_agent_actions_per_attempt": 200
  }
}
```

约束：

- 路径必须相对 job 文件，禁止 URL、绝对路径和 `..`；冻结器拒绝 symlink，并在 canonicalize 后验证目标仍位于 project root，再复制实际 bytes。
- `design` 必须解析到项目中唯一的 design。
- `fab_profile.sha256` 必须与本地 Profile bytes 匹配；只写 id/version 不足以开始运行。
- `review_authority` 通过本地受信 policy 映射到 organization root 公钥（policy 保存公钥 bytes 及其 sha256；只有 hash 无法还原公钥，也就无法验证 Ed25519 签名）。CLI 以 `--review-root-key FILE` 提供公钥 bytes，`job_valid` 校验其 sha256 与 policy 映射一致，并把该 hash 冻结进 `resolved-contract.json`。`verify` 仍只信任包外提供的公钥；bundle 可以携带 `review-root.pub` 作为非可信便利副本，`verify` 只在其 sha256 与包外提供的 hash 相等时才使用它。
- `locked_instances` 使用展开后的 stable hierarchical instance id，只能指向 CoHDL 中已经显式 `place` 的实例。位置、旋转与 side 从 CoHDL 读取，Job 不复制坐标；`resolved-contract.json` 在首个 baseline 冻结规范化的 `(instance, x, y, rotation, side)` 元组，之后每个 baseline 的 `layout.json` placements 中对应元组必须相等（比较元组，不比较整份文件字节，layout.json 其余内容的合法变化不受影响）。改动锁定实例 `place` 行的源修复不是 `source/candidate_fixable`，而是 agent 无权处理的 placement contract 失败。
- `intent` 完全是非规范化文本，只供生成和人工审查，永远不能直接生成 pass/fail；`job_valid` 限制其长度（v1 为 4096 UTF-8 bytes），因为 raw job 要作为 Lean literal 嵌入。所有机器判定事项必须出现在闭合的 `obligations[]` 中；agent 可以提议 obligation，但 schematic reviewer 必须签署“集合已覆盖本次意图”的声明。
- v1 没有 `required` 字段：每条 obligation 都是必需的，`id` 在 Job 内唯一。否则 agent 可以提议一条无人必须签署的 `required: false` review obligation 来满足 `RequiredObligationCoverage`。
- obligation-kind registry 是 versioned closed enum；未知 kind、未知/缺失 param、错误 evidence/owner 均使 `job_valid` hard-fail。v1 只有下表八项，新增 kind 必须经 RFC：

| kind | params schema | evidence / owner | sole verifier |
|---|---|---|---|
| `diff_pair_length_skew` | `pair`：两个 net 名。`job_valid` 只检查形状；绑定在 `logical_valid` 完成：`pair` 必须唯一匹配 `proof_ir` 中恰好一条有序 `diff_pair(p, n)`，且恰好一条只含这两个 net、带 `Length` tolerance 的 `length_match`（RFC-013）；零条或多条都使 `logical_valid` 失败（compiler 只查 arity 与 net 相异、不查重，`src/check/expand.rs:1874`，所以唯一性由 Harness 保证）；v1 只证明长度 skew，不证明成对间距、同层伴随或 uncoupled length，名称据实收窄；绑定后的 FQ net identity 写入 `resolved-contract.json` | `formal / harness` | `DiffPairLengthSkew` |
| `impedance_process_review` | `target, tolerance: Tolerance`。`target` 是 tagged union：`{kind: "diff_pair", nets: [p, n]}`（nominal 取 `differential_impedance`）、`{kind: "diff_pair_single_ended", nets: [p, n]}`（nominal 取同一 bracket 的 `single_ended_impedance`）或 `{kind: "single_ended", net: n}`（nominal 取 `#[impedance]`），不用 string/array 隐式重载。nominal 不在 Job 里，取自 `proof_ir` 中对应的 RFC-027 事实：三种 target 的 nominal 来源以本行前半句的逐项映射为准，三者不可互换（RFC-027 第 78/83 行分别定义单端与差分，`quilter.rs` 也分别输出）；缺失或多条即 `logical_valid` 失败（nominal 同样必须唯一）。Job 只保留 reviewer 的接受容差 | `review / hardware_reviewer` | signed `pre_order` review |
| `length_match_skew` | `nets: [a, b]`：恰好两个 net 的 `length_match(a, b) [tolerance: Length]`，且这两个 net 不构成 `diff_pair`；绑定与唯一性规则同上 | `formal / harness` | `DiffPairLengthSkew` |
| `current_capacity_review` | `target: {kind: "net", net}`，nominal 取自 `#[high_current(I)]` | `review / hardware_reviewer` | signed `pre_order` review |
| `compiler_warning_review` | `code, object_id, reason` | `review / hardware_reviewer` | signed `schematic` review |
| `analog_stability_review` | `instance_ids, document_hashes` | `review / hardware_reviewer` | signed `schematic` review |
| `thermal_review` | `instance_ids, ambient: Temperature` | `review / hardware_reviewer` | signed `pre_order` review |
| `emi_review` | `object_ids, document_hashes` | `review / hardware_reviewer` | signed `pre_order` review |

源码里每一种 RFC-013/027 结构化事实都必须有且只有一种处理结果，`RequiredObligationCoverage job logical` 检查的就是这张完整映射（任一事实实例没有结果、或结果多于一个，都失败），因此不能通过「不写 obligation」绕过源码约束：

| 源码事实（RFC-013/027） | v1 结果 |
|---|---|
| `diff_pair(p, n)` 无 bracket | 结构事实，只用于 pairing 绑定，本身不产生 verdict |
| `diff_pair [differential_impedance]` | 强制恰好一条 `impedance_process_review`（`diff_pair`） |
| `diff_pair [single_ended_impedance]` | 强制恰好一条 `impedance_process_review`（`diff_pair_single_ended`） |
| `diff_pair [frequency]` | 仅记录进 contract，不产生 verdict |
| `#[impedance(Z, frequency)]` on net | 强制恰好一条 `impedance_process_review`（`single_ended`） |
| `length_match(a, b) [tolerance: Length]`，a/b 构成 diff_pair | 机械验证：`diff_pair_length_skew` |
| `length_match(a, b) [tolerance: Length]`，不构成 diff_pair | 机械验证：`length_match_skew` |
| `length_match` 含三个以上 net、无 tolerance 或 tolerance 为 Time | v1 拒绝：`logical_valid` 失败，诊断说明 v1 只支持两 net、Length tolerance |
| `net_class NAME { … }`、`#[placement_hint]`、`#[intent]`、`#[ground]` | 仅作建议，记录进 contract，不产生 verdict |
| `#[high_current(I)]` | 强制恰好一条 `current_capacity_review` |
| `#[bypass]`、`#[crystal_oscillator]`、`#[switching_converter]` | 仅作 backend 放置建议，记录进 contract，不产生 verdict |
| `#[bga_fanout]` | v1 拒绝（BGA 已被 board class 禁止） |

未声明的自然语言含义既不被证明，也不被暗中当作 pass。已知限制：review obligation 若绑定 fn 内部实例路径（形如 `Design::__fn0_…::c`），改变调用顺序的源修复会改变路径，`ResolvedContractMatches` 会按设计终止 run，需要开新 run。这不收缩能力：fn 展开出的实例与其中声明的 net 同样可以被 obligation 引用（否则 fn 内的阻抗事实无法被强制 review），只是这类绑定对调用顺序变化敏感。
- 冻结对象分两层，避免把只有解析源码后才知道的事实塞进 Job 解析阶段：run-global、不可变的 `resolved-contract.json` 在首次 `logical_valid` 之后生成，冻结原始 `job.json` 的 sha256、Profile identity、`review_authority` 与其 root hash、每条 obligation 绑定后的 FQ net/instance identity，每个 locked instance 规范化的 `(instance, x, y, rotation, side)` 元组，以及 board outline 三元组 `(board_outline.source_path, raw_dxf_sha256, normalized_geometry_sha256)`（否则 agent 可以借源修复换 DXF 扩板，与「不得扩板」冲突）；per-baseline 的 `baseline-manifest.json` 保存该 baseline 的 source、manifest、dependency lock、DXF、`design.lock` 与 `proof_ir` 哈希。后续 attempt 只引用这两个冻结对象；source repair 产生新的 baseline-manifest，永远不改 resolved-contract。

`board_class` 与 Profile 同一形状：`{id, version, sha256}`，指向一个版本化、内容寻址的闭合 schema（`board-class.json`）。其 bytes 内联进 `resolved-contract.json`，`ResolvedContractMatches` 检查内联 bytes 的 sha256 等于 raw Job 声明的 sha256，因此 resolver 无法在名称不变的情况下放宽规则。`DesignWithinBoardClass` 是在 `proof_ir` 的 footprint identity、pad pitch 与层数上对它求值的可判定 checker。`mcu_iot_control` v1 字段：`copper_layers: 2`；`sides: [top, bottom]`；`min_passive_size: 0402`；`min_lead_pitch: 0.4mm`；`allowed_ipc7351_families`，只能取 RFC-021 真实闭集的子集（CHIP/MELF、SOT、SOIC/SOP、QFP/LQFP/TQFP、QFN/SON），v1 允许其中全部；`forbidden_ipc7351_families: [BGA]`；`allowed_footprints`，一份显式 allowlist，条目为 FQ footprint identity + geometry sha256，用于 RFC-021 命名之外的 footprint（THT header、USB 连接器、晶振、模组、安装孔），不在 allowlist 且不属于允许族的 footprint 即失败。via 种类不在 board class 里：via 只存在于 routed board，由 Profile 的 `FabricationRulesSatisfied profile board` 判定（Profile 表已固定「through via only」）。RFC-021 的 family 是几何命名分类，不是电路用途分类：RF matching 网络可以完全由合法的 0402 CHIP 器件组成，市电设计也可以只用普通连接器与无源器件，所以 board class 不能、也不声称能机械排除 Non-goals 中的 RF/天线/市电。v1 的选择是人工分类：`schematic` reviewer 签署 `scope-classification.json`，它进入 schematic-review-root，并在第 1 节的信任表中列为外部假设，与 Profile 同级。它只声明机器无法从当前 IR 判定的用途事实；能机器判定的（BGA、via 种类、层数、铜厚、panelization）由 board class 与 Profile 谓词负责，不进这里。v1 schema 是闭合的，未知字段、缺失字段或任一 `true` 都使 `schematic_confirmed` 失败：

```json
{
  "schema_version": 1,
  "rf_matching_present": false,
  "antenna_present": false,
  "mains_voltage_present": false,
  "ddr_or_high_speed_memory_present": false,
  "reviewer_note": ""
}
```
未来要机械排除或允许 RF，必须先经 RFC 引入可判定的 feature 分类，不能从自然语言或 footprint 名猜测。via-in-pad 需求同样不从 footprint 推导，由最终 board 上的 Profile checker 判断。「常见 MCU/IoT 板」这类自然语言描述只是 label，不参与判定；它描述设计子集，不代表贴装服务资格。USB 2.0 的差分几何与长度可以检查，但双层 Profile 不声明受控阻抗保证，因此每条单端 `#[impedance]` 事实与每条 `diff_pair [differential_impedance]` 事实都必须恰好对应一条 `impedance_process_review`，由闭合谓词 `RequiredObligationCoverage job logical` 按上文的完整映射表保证（零条或多条都失败），或切换到未来的受控阻抗 Profile。

本 RFC 不新增 `.cohdl` grammar；下列现有 source 片段说明 Job 锁定与引用的事实（锁定坐标、差分对与标称阻抗、长度匹配上限）仍来自语言本身，而不是复制进 JSON：

```cohdl
layout {
    place usb at (6mm, 12mm) rotate 90 side top
    place mount_1 at (3mm, 3mm)
    diff_pair(USB_DP, USB_DM) [differential_impedance: 90ohm]
    length_match(USB_DP, USB_DM) [tolerance: 0.25mm]
}
```

### 3. Immutable attempts and mutation tools

Run 内每次 source 修复产生新的不可变 baseline，每个候选按实际 bytes 内容寻址；attempt 只保存引用和动作链：

```text
runs/<run-id>/
  input/
    job.json
    fab-profile.json
    fabrication-order.json
  contract/
    resolved-contract.json  # 首次 logical_valid 后生成，run 内不可变
    scope-classification.json  # schematic reviewer 签署的范围声明
  baselines/00000001/
    source-snapshot/        # 含进入该 baseline 时的 design.lock（首个 baseline 可以没有）
    design.lock             # 该 baseline 首次成功 build 写出的 canonical lock，之后冻结
    baseline-manifest.json  # 该 baseline 的 source/manifest/dependency/DXF/design.lock/proof_ir 哈希
    draft-commit.json       # draft baseline 的提交记录：源码快照与 diagnostics 的 sha256（解析失败时没有 proof_ir、没有审批）
    baseline-commit.json    # 冻结记录：成员路径 + sha256（含 proof_ir、scope-classification、schematic approval、其 snapshot 与 authorization chain），最后一步 rename；只有通过 schematic_confirmed 的 baseline 才有
    dependencies/sha256/
    proof-ir.json
  objects/sha256/
    <aa>/<64-hex>            # 所有内容寻址成员（board、contract、approval、snapshot、chain …）都在这里，无类型扩展名；类型、逻辑路径与 schema 由 commit marker 描述
  attempts/0001/
    attempt.json
    actions.jsonl
    reports/
    export-staging/
  final/
    proof-manifest.json
    sha256sums.txt
  run-receipt.json          # ready | review_pending | failed；派生缓存，不进入 hash DAG
```

Agent 不直接编辑 `.kicad_pcb` 文本。Harness 暴露闭合、带 schema 的 mutation tools，例如：

- `move_instance(candidate, ref, x, y, rotation, side)`
- `add_track(candidate, net, layer, width, points)`
- `add_via(candidate, net, x, y, diameter, drill)`
- `add_zone(candidate, net, layer, polygon, clearance)`
- `remove_route(candidate, route_id)`
- `inspect_region(candidate, bbox)`
- `verify_candidate(candidate, checks)`

`candidate_id` 就是 exact board bytes 的 SHA-256，不接受 agent 自报字符串。每个 mutation 只读取一个 opaque candidate id，先检查对象身份、网络归属、单位、层和 Profile 规则，再产生新 digest；`attempt.json` 记录 baseline id、parent digest、tool call、result digest 和 strategy id，禁止原地覆盖。外部 backend 输出先进入 quarantine，只有 parse/support/Profile preflight 通过后才进入对象库。Tool 返回结构化结果和诊断，不返回一个由模型自行解释的“success”句子。Backend 可以在内部使用 solver、autorouter 或 KiCad automation，但其输出必须重新解析成同一个 Routed Board IR，不能绕过工具边界。

如果 CoHDL source、manifest、lock 或 DXF 在一次修复中变化，旧 baseline、候选和报告继续保留，但 routing、proof、schematic coverage 与 approval 全部失效；Harness 创建新的 baseline id，从 compiler 阶段重新开始，重新进入 `schematic_confirmed` 取得绑定新 `baseline_id` 的 schematic approval（旧 approval 的 root 含旧 baseline-manifest hash，对新 baseline 必然不匹配），再在 `resolved_contract_frozen` 重跑 `ResolvedContractMatches`。Routing 修复不得顺便换 part；placement 修复不得改变 net；proof failure 不允许通过 prompt 争辩或降级。

### 4. Verdict ladder and state machine

Machine state 和 rung 名称统一使用 `snake_case`。每个 attempt 按以下固定顺序运行：

```text
job_valid
  ⊂ fabrication_order_frozen
  ⊂ source_valid
  ⊂ logical_valid
  ⊂ schematic_confirmed
  ⊂ resolved_contract_frozen
  ⊂ board_normalized
  ⊂ placement_complete
  ⊂ routing_constructed
  ⊂ zone_materialized
  ⊂ routing_complete
  ⊂ physical_valid
  ⊂ fab_profile_valid
  ⊂ kicad_drc_valid
  ⊂ formal_valid
  ⊂ export_verified
  ⊂ artifact_bound
  ⊂ review_complete
  ⊂ ready
```

- `job_valid`：schema、路径、哈希、board class、Profile、closed obligations 的形状和预算合法。此时源码尚未解析，obligation 对 net/instance 的引用只查形状，不查存在性。
- `fabrication_order_frozen`：由固定 Profile 生成 canonical order bytes 并冻结为 input leaf；任何变化使全部后续 rung 与 approval 失效。
- `source_valid`：项目可离线解析，依赖与 lock hash 验证通过。
- `logical_valid`：CoHDL parses/resolves/type-checks/connects，四条 residual DRC 无 error，build-only part/footprint checks 通过，design 声明了 `board_outline`（RFC-020 允许缺省，但后续 `PlacementsInsideOutline` 需要它，缺失归 `source/candidate_fixable`），design 使用的 footprint 都不含 `window` 内部开窗（首版排除，见第 5 节），baseline 的 `design.lock` 已冻结（RFC-005 的 allocator 会在实例增删时合法地新增 designator 与 tombstone，并把 lock 重渲染为 canonical 字节，所以不能要求 lock 跨 baseline 字节不变：每个 baseline 首次成功 build 写出的 canonical lock 即为该 baseline 的冻结值，同一 baseline 内的任何重放必须复现相同字节，否则 hard fail，owner `artifact_integrity`），并由单独接受的 additive emitter 生成 canonical `proof_ir`。编译器输出的每条 warning（例如 D003 single-driver）必须映射到一条显式 `compiler_warning_review` obligation。随后 Harness 在 `proof_ir` 上执行 `FabricationReadyFootprints`：所有 populated electrical instance 必须有非空、非 placeholder footprint，且 geometry hash 与冻结依赖一致；mechanical-only 的判定规则固定为「零 terminal（不是任何 net 的成员且无 required pin）且 footprint 仅含 mount hole」，语言里没有统一的 mechanical 标记（`lib/misc` 的 `Mounting_Hole` 是包内 trait），所以不依赖 trait 名。这是 Harness 层检查，compiler 对空 placeholder footprint 的 RFC-018 例外不变。
- `schematic_confirmed`：冻结前的 **authoring 阶段**到此结束。在 `logical_valid` 之前 agent 可以生成与修复 `.cohdl` 源码（这正是今天 `harness/repair_loop.py` 的循环），每次修复产生一个 **draft baseline**（源码快照 + diagnostics，解析或类型检查失败时没有 `proof_ir`），首次通过 `logical_valid` 的才是 logical_valid baseline；draft baseline 不产生 candidate、proof 或 final；源码错误与缺失 `board_outline` 只在这个阶段是 `source/candidate_fixable`。`schematic` reviewer 在此签署 **schematic-review-root** `= SHA256("cohdl-agent-schematic-review-root-v1\0" || baseline_id || resolved_contract_sha256 || baseline_manifest_sha256 || proof_ir_sha256 || obligations_sha256 || scope_classification_sha256)`（所有 digest 为 raw 32 bytes，`baseline_id` 为 8 位零填充 ASCII 十进制（如 `00000001`），与 `baselines/<id>/` 目录名相同；前缀做 domain separation，未来 v2 字段不会与旧 root 混淆；`baseline_id` 进入被签数据，verifier 比较它与 `baseline-manifest.json` 内的 id，不信任 approval 文件里未签名的字段）：它同时绑定 contract 内容（obligation coverage、board outline 三元组、locked 元组、board class hash、FQ 绑定）和被审查的真实原理图（source、依赖、`proof_ir`），签名是 detached 的，签名及其 hash 都不进入任何被签对象，所以没有 `contract → signature → contract` 自环。`scope-classification.json` 是 reviewer 同时签署的范围声明（见第 2 节 board class 段）。没有这份签名不能冻结，agent 也无法伪造它（见第 4 节反馈权限表的 `review` 行）。
- `resolved_contract_frozen`：首个 baseline 在此把 `resolved-contract.json`、`proof-ir.json`、`design.lock`、`baseline-manifest.json`，以及 `schematic_confirmed` 的全部证据（`scope-classification.json`、schematic approval、该 approval 引用的 revocation snapshot、相关 role authorization chain）作为一个原子单元提交，这样本 rung 成立就能机械推出前一 rung 成立，不会出现 marker 存在而审批证据缺失的半提交状态：这些文件分布在 `contract/` 与 `baselines/NNNN/` 两个父目录，一次文件系统 rename 提交不了它们，所以先把全部内容写进内容寻址的 `objects/sha256/`，最后只原子 rename 一个 `baselines/NNNN/baseline-commit.json`（列出每个成员的路径与 sha256）；状态机只认可存在且成员哈希齐全的 commit marker，没有 marker 或哈希不齐等于什么都没发生，重放从 `logical_valid` 重新开始。之后每个 baseline 在此重跑 `ResolvedContractMatches`：所有 obligation 的 FQ 绑定、locked 元组与 board outline 三元组必须解析到相同结果，任一变化即 run `failed`，owner 为 agent 无权处理的 placement/source contract。agent 的第一个 candidate 动作只能发生在此 rung 之后；冻结前 locked placement 尚不是锁，它们在 `schematic_confirmed` 由人工定格。
- `board_normalized`：从 compiler 写出的无 net 表 board 到带受检 net 表的候选，是一次内容寻址的 import/serialization transformation（产生新 digest，记录 parent digest）；此后本 rung 是检查而不是变换。候选的 `(general …)`（板厚在这里，`src/emit/kicad_pcb.rs:53`）与 `(setup …)`（stackup、板级 mask/paste 默认值等）必须与 Harness 的固定模板语义相等（emitter 写的就是固定模板；backend 或 `--save-board` 之后有任何偏离即 hard-fail，否则 backend 可以从板文件内部改写 via 长度、mask 开窗与 KiCad 读取的设置），模板 hash 进入 manifest。板级默认值可被 footprint/pad 级设置覆盖（RFC-018 的 `mask_expansion`、paste override），所以冻结模板不能替代生效值检查：physical verifier 与 export parse-back 使用 PadPlan 携带的逐 pad 生效值。CoHDL emitter 写的是按名 `(net "name")`、没有 net 表；KiCad `--save-board` 之后文件才带 KiCad 自己分配的 net 表与 ordinal。Harness 序列化候选时按 net 名 bytewise 排序生成受检 net 表（0 保留给无 net 对象），mutation tools 与 Routed Board IR 始终以 net 名为身份；ordinal 永远不承载身份：每个 digest 上都检查 net 名唯一且与 `proof_ir` 一一对应、任何 ordinal 与 name 的绑定在文件内一致、track/via/zone 只引用已知 net。该检查在 `zone_materialized` 产生新 digest 后重跑。重复、未知或 name/ordinal 不一致均 hard-fail。
- `placement_complete`：每个实例有确定 position/rotation/side，locked placement 未变化；未出现在 `layout.json` placements 中的实例视为 `staged`（emitter 只把它们放在 shelf/grid 坐标上，不写 provenance，Harness 从 `layout.json` 的缺席推导），只有被显式 placement action/solver 接管且 courtyard 位于允许区域后才算完成。
- `routing_constructed`：定义 `terminal_set(net)` 为该 net 所有显式连接的 logical pin（`IrNet.members` 已含接入的 optional pin，`src/ir.rs:194`）展开出的物理 pad 集合，`route_required(net) := |terminal_set(net)| >= 2`；每个 route-required net（按名）至少被一个 track、via 或 zone 引用，且候选中不存在引用未知 net 的 copper 对象；zone 此时只算声明，不用于连通性结论。
- `zone_materialized`：固定 KiCad 10.0.4 在候选副本上执行 `pcb drc --refill-zones --save-board --exit-code-violations`。这是 transform + preliminary report：exit 0 或 5 且产出可解析 board 时形成**新** digest，5 中的 violations 进入 repair；加载/参数/保存失败才是 `external_tool`。此后所有 gate 只检查新 digest，证明后禁止再次 save。
- `routing_complete`：从 materialized copper 计算 terminal connectivity 和 ratsnest；不能把 zone 声明当作铜。非 route-required（单 terminal）net 没有 routing obligation，与上一级同一定义；一个 logical pin 展开出的所有物理 pad 都属于 `terminal_set`；optional pin 一旦接入 net 同样必须布通。
- `physical_valid`：板框、同面 courtyard、铜短路、clearance、track/via geometry、层和 dangling copper 的闭合谓词集成立。v1 没有 keepout：语言、Job 与 board class 都没有 keepout 来源，若以候选自身的 rule area 为准则谓词真空成立或可被 backend 任意改写，所以 v1 不设 `KeepoutsRespected`，候选中的任何 rule area 由 `SupportedConstructsOnly` 拒绝；安装孔周边的铜排除由 Profile 的 hole-to-copper 规则覆盖。未来经 RFC 引入冻结的 keepout 来源后再加谓词。
- `fab_profile_valid`：所有适用制造规则满足被哈希固定的闭合 Profile；不存在“Profile 未列即通过”。
- `kicad_drc_valid`：固定版本运行 `kicad-cli pcb drc --format json --severity-all --exit-code-violations`。工具执行状态与 policy verdict 分开记录；v1 不允许 DRC waiver，board 内 exclusion 不继承，error 与 warning 必须为零。候选板的 footprint 是嵌入式的、没有库链接，KiCad 的库一致性检查可能对每个 footprint 报 warning（spike 执行前不作断言）；对此本 RFC 的默认仍是零 warning，并把一条闭合 ignore 清单标为 **unresolved**：只有 Decision 所列 spike 用 KiCad 10.0.4 实测证明这类 warning 不可避免、能按精确 test 名识别、且通过 hash 固定的 `.kicad_pro` severity map 降级后不会掩盖真正的 footprint 问题，清单（候选名 `lib_footprint_issues`、`lib_footprint_mismatch`，仅供 spike 核对，不是规范）才能写入规范；否则 Harness 改为生成固定的本地 footprint library 消除该 warning，不做任何降级。清单之外的 test 一律不得 ignore/disable。canonical report 记录实际生效的 severity map hash。
- `formal_valid`：Lean 接受绑定 raw Job、resolved contract、Profile、`proof_ir`、fabrication order 与实际 board bytes 的 `ValidBoardBytes`；certificate 中的 order bytes 必须等于 final `fabrication-order.json`。
- `export_verified`：Gerber、Excellon 与 IPC-D-356 均从 staging 反向解析，并分别与已证明 board 核对 layer/outline/copper geometry、soldermask 开窗与 silkscreen 图形、drill/slot/plating 分类和 pad/net identity；hash 不能替代这一语义检查。若 parser 未实现，最高状态只能是 `formal_valid`。
- `artifact_bound`：所有已验证 payload 的无环 hash DAG 完整，且未在验证后改 byte。
- `review_complete`：required human review 在 v1 固定为 `schematic` 与 `pre_order` 两份，不是 Job 字段，Job 不能缩减或置空（否则空集会让本 rung 真空通过）。`schematic` approval 绑定具体 `baseline_id` 与 schematic-review-root，本 rung 只接受属于 winning baseline 的那份并重新验证它未被撤销（旧 baseline 的 approval 保留供审计，不能满足新 baseline 的 gate）；`pre_order` 在此签署 payload root（manifest sha256）与 obligation set；未完成时状态只能是 `review_pending`。
- `ready`：上述每一 rung 同时成立。不存在 `force`、`accept-anyway` 或“仅警告后视为 ready”的入口。

一个 attempt 失败后进入 `rejected`。以下表是反馈权限的唯一规范来源：

| Owner/subtype | Feed agent | Allowed mutation |
|---|---:|---|
| `source/candidate_fixable` | yes | 新 baseline 中修改 `.cohdl`，随后全量重跑 |
| `placement/candidate_fixable` | yes | 仅移动未锁定实例 |
| `routing/candidate_fixable` | yes | track/via/zone 的局部增删改 |
| `physical/candidate_fixable` | yes | 仅使用该诊断列出的 placement/routing mutation |
| `profile/candidate_violation` | yes | 修板，不得改 Profile |
| `profile/definition_invalid` | no | 独立 Profile 开发任务 |
| `formal/checker_integrity`、`external_tool`、`artifact_integrity` | no | 独立工具开发任务 |
| `review` | no | 等待或拒绝，不让模型伪造审批 |

达到 attempt/action 上限、工具超时或工具缺失时，run 以 `failed` 结束。`strategy_id` 由 Harness 从预先枚举的 strategy 列表中分配并记录，agent 不能自报或更换（否则换个 id 就能规避停止条件）；同一 `(failure_signature, strategy_id)` 连续出现三次后只允许切换到下一个预先枚举的 strategy；没有下一 strategy，或新 strategy 再达到三次，立即 `failed`。所有状态保留证据，不继续无界循环。

### 5. Routed Board IR

Routed Board IR 是从实际 `.kicad_pcb` bytes 解析出的规范化验证模型，不是新的手工 authoring format。首版只接受 harness mutation tools 能产生的 KiCad 10.0.4 子集：straight track、through via、materialized polygon zone，以及 pad。pad 的可接受集合由一个公开、版本化的 `PadPlan` proof schema 定义（子 RFC 固定；其 schema version 与 sha256 进入 formal-definition identity 与 proof manifest），它必须覆盖今天 `kicad_mod::pad_plans` 产出的全部三类对象，而不是以内部函数名充当规范：electrical pad（语言 `PadShape` 的 rect/circle/oval/annulus，加 rect 的 `corner_radius` 与 `chamfer`，含超过 KiCad 0.5 上限时的五顶点 chamfer 多边形；SMD、PTH 与 plated slot 钻孔形式）、paste aperture（circle/rect override，以及分段 annulus 的 custom paste 多边形，无编号、不带 net）、mount hole（RFC-022/023 的 PTH/NPTH 圆孔、矩形与 oval 槽）。库中的 QFN/QFP 倒角焊盘、annulus 焊盘与安装孔因此都在子集之内。track arc、schema 之外的 custom pad 与其他 copper construct 均 fail closed。非铜构造同样是闭合清单，按「节点 + 上下文 + 允许值」定义，且必须恰好覆盖首版支持范围内 emitter 写出的板文件（否则 compiler 自己的输出会被拒；支持范围之外的构造在 `logical_valid` 就拒绝，不会走到这里）：footprint 级 `property` 只允许 `Reference`/`Value`/`Datasheet`/`Description` 四个键，值必须等于 `proof_ir` 投影（designator、part identity 等）；`(embedded_fonts no)` 在板级与 footprint 级允许，且值只能是 `no`；footprint 的 reference/value 文本、RFC-031 silkscreen 图形、courtyard 图形；footprint 的 `window` 内部开窗（emitter 写为 Edge.Cuts 上的 `fp_rect`/`fp_circle`，`src/emit/kicad_mod.rs:182`）不在首版支持范围：它需要开窗来源绑定、随元件变换、内部切除区域与制造输出反查四项额外验证，v1 在 `logical_valid` 拒绝含 `window` footprint 的设计（owner source），而不是只放行图形节点；板级 `gr_line`/`gr_arc` 只允许出现在 Edge.Cuts 且构成 `OutlineMatchesContract` 比较的那条轮廓；`(general …)`、`(setup …)`、layer 表与 net 表。`group`、rule area、Edge.Cuts 之外的板级自由文本与图形、任何其他键的 `property`，以及清单之外的节点一律 `unsupported_construct`。这份清单以 emitter 的固定输出为基线并随 emitter 版本 hash 一起进 manifest。遇到未知、无法语义保持的 KiCad construct 必须报 `unsupported_construct`，不得忽略。

最小模型包括：

- board outline 的 line/arc 闭合轮廓；
- component identity、designator、footprint、position、rotation、side、courtyard；
- pad 的真实变换后几何、层、plating、drill 与 net；
- straight track segment、through via、filled zone polygon 与 net；
- Edge.Cuts、copper/soldermask/silkscreen 必需层；
- net 与 logical pin/pad 的 provenance；
- board `(general …)` 与 `(setup …)` 中参与 DRC 和制造的值（板厚、stackup、板级 mask/paste 默认值等），解析为规范化值，并与 pad 级覆盖合成逐 pad 生效值。

KiCad decimal 精确解析为有理数，不得用平台 `f64`。若采用整数近似，必须使用有方向证明的双包络：内包络只能证明正向连通，外包络用于拒绝短路/间距违规；containment 比较对象外包络与合法区域内包络。几何子 RFC 必须逐 shape 给出近似结果到 KiCad 实际几何的方向性定理，并固定旋转误差、开闭边界、可靠接触裕量、共享 conductive layer、via/PTH 跨层语义，以及 zone hole/island/thermal 集合语义。输出相邻结构显式排序。

### 6. Formal invariants

独立 `formal/` Lean 工程定义 `BoardJob`、`FabProfile`、`LogicalIR`、`RoutedBoard` 与 `ValidBoard`。首版证明最终候选相对于**固定 compiler projection 与 Profile assumptions**有效，不证明搜索算法，也不宣称 source → IR compiler soundness。最终入口绑定六类规范输入 bytes：原始 `job.json` 与 `resolved-contract.json` 都进入 Lean（只绑定 hash 不够，因为 contract resolver 若删 obligation、扩大 `allowed_sides` 或换 `board_class`，hash 检查发现不了），由 `ResolvedContractMatches` 证明 contract 是 raw Job 相对于 LogicalIR 的忠实解析；其余四类为 Profile、`proof_ir`、fabrication order 与实际 board bytes：

```lean
def ValidBoardBytes
    (rawJobBytes contractBytes profileBytes logicalIrBytes
     fabricationOrderBytes boardBytes : ByteArray) : Prop :=
  match parseRawJob rawJobBytes, parseContract contractBytes, parseProfile profileBytes,
        parseLogicalIR logicalIrBytes, parseFabricationOrder fabricationOrderBytes,
        parseBoard boardBytes with
  | .ok rawJob, .ok job, .ok profile, .ok logical, .ok order, .ok board =>
      ValidBoard rawJob job profile logical order board
  | _, _, _, _, _, _ => False

def ValidBoard (rawJob : RawJob) (job : ResolvedContract) (profile : FabProfile)
    (logical : LogicalIR) (order : FabricationOrder)
    (board : RoutedBoard) : Prop :=
  ResolvedContractMatches rawJob job logical ∧
  ProfileIdentityMatches job profile ∧
  FabricationOrderMatchesProfile profile order ∧
  UniqueDesignators logical ∧
  RequiredPinsResolved logical ∧
  FabricationReadyFootprints logical ∧
  PartFootprintPadExact logical ∧
  BoardMatchesLogical logical board ∧
  OutlineMatchesContract job board ∧
  LockedPlacementsPreserved job board ∧
  AllowedSidesRespected job board ∧
  DesignWithinBoardClass job logical ∧
  OutlineWellFormed board ∧
  PlacementsInsideOutline board ∧
  SameFaceCourtyardsDisjoint board ∧
  EveryRequiredNetConnected logical board ∧
  NoUnexpectedConnectivity logical board ∧
  NoCrossNetCopperIntersection board ∧
  CopperLayeringValid board ∧
  NoDanglingOrUnnettedCopper board ∧
  ZonesMaterialized board ∧
  FabricationRulesSatisfied profile board ∧
  FormalObligationsSatisfied job logical board ∧
  RequiredObligationCoverage job logical ∧
  SupportedConstructsOnly board
```

这是 v1 的闭合 predicate set；physical/Profile/Rust report 的每一 pass 字段必须映射到其中一个 predicate，不能另有隐藏 verdict。`NoDanglingOrUnnettedCopper` 只量化 track/via/zone；pad 的 net 归属由 `BoardMatchesLogical` 按 `proof_ir` provenance 决定。Profile tuple → `ProfileIdentityMatches`，raw Job ↔ contract 忠实解析 → `ResolvedContractMatches`，下单 exact choices → `FabricationOrderMatchesProfile`，locks/sides/formal obligations 分别映射同名 predicate，`board_class` 的闭合能力范围 → `DesignWithinBoardClass`，冻结板框 → `OutlineMatchesContract`；review obligations 由 review gate 关闭，但「每条阻抗事实恰好一条 review obligation」这一覆盖关系本身是机器谓词 `RequiredObligationCoverage`。v1 不允许 machine-check waiver；人工 review 不能覆盖上述任一 false predicate。

每个 predicate 必须同时提供：可读的 `Prop`、可执行的 `Decidable`/Boolean checker、以及 checker soundness 定理。Rust 可以提供加速 witness；Lean 必须检查 witness，不能信任 Rust 布尔值。大文件不能由 kernel reduction 通过 IO 读取；生成器把 canonical bytes 作为固定分块 literal 嵌入，或提供 Lean parser 检查的分块/hash certificate。raw Job、resolved contract、Profile、`proof_ir`、fabrication order、board 六类输入一视同仁。

首版实例证明优先使用 kernel-checkable reduction（例如 `decide_cbv`）。若规模导致不可接受的证明时间，后续可以加入可独立检查的 certificate；不得为了速度默认采用扩大可信基的 `native_decide`。固定 axiom whitelist 只有 `propext`、`Classical.choice`、`Quot.sound`；`sorryAx`、`Lean.trustCompiler`、任何 per-invocation native axiom 和自定义 axiom 均 hard-fail。正式门禁固定允许的 imports，并同时保存 `#print axioms` 输出与 `lean4checker --fresh` 结果。

证明必须绑定**实际交付文件**，而不只是 Rust 声称从文件解析出的对象。Lean 侧 pure parsers 对嵌入 bytes 计算六类模型；artifact binder 重新计算 exact byte hashes。只有 Rust projection 通过而任一 Lean parser/certificate 未实现时，最高状态是 `physical_valid`，不能称 `formal_valid`。

`proof_ir` 是 v1 的必要 compiler projection：版本化、canonical、byte-deterministic，并包含 pin role、NC、展开 instance/pad/net、part/footprint identity、geometry hash、placement provenance、RFC-020 board outline 的规范化几何（`layout.json` 今天已导出它；`ResolvedContractMatches` 检查 contract 的 `normalized_geometry_sha256` 等于 `proof_ir` 的板框几何 hash，使 compiler 投影成为板框的唯一来源），以及 RFC-013 layout constraints 与 RFC-027 physics attributes；现有 `.net`、BOM、`layout.json` 不能无损替代它。该 additive emitter 必须由独立子 RFC 接受后才能实现。Compiler soundness、expansion preservation 和 source → IR 证明属于未来独立 RFC；它们落地前不得提高本证书的措辞。

### 7. Initial manufacturing Profile

`{id: "jlcpcb-standard-2l", version: 1, sha256: "<64 lowercase hex>"}` 是经过人工审核、内容寻址的不可变**裸板** Profile，显示名为 `jlcpcb-standard-2l-v1`。它绑定普通 routed single board，不绑定 panelization 或 assembly。Profile schema 的 rule kind 闭集为 `minimum | maximum | exact | allowed_set | forbidden | conditional`；每条几何规则还必须声明 `from_shape`、`to_shape`、`measurement`、适用层与边界是否包含，不能用一个含糊的 “clearance” 数字代替。

下表是首版核心规则。「选定值」是 Profile 实际固定的值，「JLCPCB 页面值」是 2026-09-05 抓取官方 capabilities 页面得到的 1–2 层 1 oz 工厂能力基线，两者不必相等：首版刻意取不加价的标准工艺档，而不是工厂能力极限。「等于或严于」按 rule kind 定义：`minimum`/`maximum` 的选定界只能等于或收紧页面界，`allowed_set` 的选定集合必须是页面集合的子集，`exact` 的选定项必须属于页面允许集合，`forbidden` 只能增加禁止项。每行页面值都必须在 evidence snapshot 中有对应摘录；本表不是完整 Profile 文件的替代品：

| 项目 | 选定值 | JLCPCB 页面值 |
|---|---|---|
| 基材 / 层数 | FR-4 / 2 copper layers | 1–32 层 |
| 板厚 / finished copper | 1.6 mm / 1 oz | 0.4–2.0 mm FR-4 选项 / 1 oz 或 2 oz |
| soldermask / finish | green / lead-free HASL | 标准色 / HASL、ENIG、OSP |
| 最小 track width | 0.15 mm | 0.10 mm |
| 不同网络 track-track clearance | 0.15 mm | 0.10 mm |
| same-net track spacing | 0.25 mm | 0.25 mm |
| 不同网络 SMD pad-pad clearance | 0.15 mm | 0.15 mm |
| pad-track clearance | 0.15 mm | 0.10 mm |
| via diameter / drill | 0.60 mm / 0.30 mm | 0.25 mm / 0.15 mm |
| via hole edge 到 track copper | minimum 0.20 mm | 0.20 mm |
| PTH annular ring | 0.25 mm | 建议 0.25 mm，绝对最小 0.18 mm |
| PTH-track clearance | 0.35 mm | 最小 0.28 mm，建议 0.35 mm |
| routed edge/slot 到 copper | 0.30 mm | 0.20 mm |
| silkscreen 到 pad | 0.15 mm | 0.15 mm |
| silkscreen line / text height | 0.15 mm / 1.00 mm | 0.15 mm / 1.0 mm |
| green 1 oz soldermask bridge | 0.10 mm | 0.10 mm |
| soldermask opening 到相邻 trace | 0.09 mm | 0.09 mm |
| 最小 SMD pad | 0.25 × 0.25 mm | 0.25 × 0.25 mm |
| via hole-to-hole / pad hole-to-hole | 0.20 mm / 0.45 mm | 0.20 mm / 0.45 mm |
| 最小 NPTH | 0.50 mm | 0.50 mm |
| plated / non-plated slot width | 0.50 mm / 1.00 mm | 0.50 mm / 1.00 mm |
| 允许 via | through via only | 2 层仅 through via |
| 禁止：blind/buried via | forbidden | 主能力表明确不支持，仅 through hole；evidence snapshot 须记录主表与 FAQ 表述不一致处 |
| 禁止：microvia | forbidden | 主能力表没有给出可采用的 bounded contract（FAQ 另提 HDI/laser via），v1 保守禁止 |
| 禁止：via-in-pad | forbidden | 页面有条件支持（soldermask 填充 ≤0.5 mm；epoxy/铜浆填充加盖 0.15–0.55 mm，6 层以上默认），本 Profile 不采用 |
| 禁止：castellation | forbidden | 页面有条件支持（孔径 ≥0.5 mm、离板边 ≥1 mm、孔距 ≥0.5 mm、板 ≥10×10 mm、厚 ≥0.6 mm），本 Profile 不采用 |
| 禁止：plated edge | forbidden | 页面有条件支持（铜 + ENIG，不支持 HASL，板 ≥10×10 mm、厚 ≥0.6 mm、至少 3 处断口），与本 Profile 的 HASL 冲突 |
| 禁止：controlled-impedance claim | forbidden | 页面仅在 4 层及以上提供（标准 ±10%），2 层无此保证 |

机器可读 Profile 中每条规则都要携带两段 `provenance`：`provenance.factory` 记录工厂基线（evidence snapshot 的 sha256 与摘录 id，以及页面值），`provenance.selection` 记录 Profile 的选择（选定值，以及与基线不同时的 rationale）。二者缺一、或选定值违反上一段的「等于或严于」规则，都使 completeness gate 失败。

完整 Profile 还必须显式覆盖且不得留空：board 最小/最大尺寸、board/outline tolerance、drill-to-drill、NPTH/PTH-to-copper、slot 长宽/圆角、soldermask/paste aperture、legend、所有 pad/via/edge 条件分支，以及 fabrication-order 中不能从 Gerber 推出的 exact choice。`fabrication-order.json` 因而固定 FR-4、2 层、1.6 mm、1 oz、green、lead-free HASL 和其他下单选项，并进入 artifact root。任何适用类别为 `unspecified` 都使 Profile completeness gate 失败；在完整 machine-readable Profile 与 fixtures 落地前，本 RFC 不能 Accepted。

来源包保存 JLCPCB 官方 rigid PCB capabilities 的人工审核快照、页面标题、抓取时间、内容 SHA-256 与逐规则摘录映射；URL 只作导航。公开网页未来变化不修改 v1，需要变化时发布 v2。规则是 exact choice、允许集、禁止项或带测量语义的上下界，不统称 “hard floor”。Agent 永远不能自动降级 Profile 或选择加价特殊工艺。

### 8. Physical and manufacturing checks

独立 physical verifier 的 v1 闭合检查集为：

- 候选 Edge.Cuts 的规范化几何等于 contract 冻结的 `normalized_geometry_sha256` 对应几何（`OutlineMatchesContract`；否则 backend 可以只扩大候选板框来消除越界）；outline 闭合且不存在自交；所有要求在板内的 courtyard、pad、hole、track、via、zone 均位于允许区域；
- 同一装配面的不同实例 courtyard 不相交；top/bottom 的二维投影可重叠，THT 与 mechanical-only 对象的穿透从 `through_all` pad 与 mount-hole 几何保守推导为「两面都占用」，不声称有三维模型。v1 不允许 geometry/DFM waiver；
- locked instance 的 position/rotation/side 与 `resolved-contract.json` 冻结值完全一致（而不是与当前 baseline 比较，否则源修复可以搬走锁）；
- 从 LogicalIR 推导每个 net 的 `terminal_set`（所有显式连接的 pin，含接入的 optional pin）；多-pad logical pin 的全部 pad 连通，single-terminal net 无 route obligation，且没有额外 terminal 被吸入该分量；
- 不同网络只在共享 conductive layer 上比较实际 copper shape，相交或违反 shape-to-shape clearance 均失败；F.Cu/B.Cu 投影重叠本身不是短路，除非经 via/PTH 导通；
- track、via、drill、annular ring、slot、pad、edge 和 silkscreen 满足 Profile；
- 只验证 `zone_materialized` 后的新 candidate；zone hole、thermal 与孤岛按实际 polygon 处理，不能用于虚构连通；
- route layer 和 via span 在双层 Profile 中合法；track/via/zone 没有 dangling segment 或无 net 对象。pad 不在此谓词范围内：`nc` pin 与未连接 optional pin 的 pad 按 emitter 约定不带 `(net …)`，它们必须无 net，且不得接触任何有 net 的铜（由 `NoUnexpectedConnectivity` 关闭）；
- 所有 populated electrical footprint 非 placeholder，identity/geometry hash、pad number/net 与 placement provenance 均与 `proof_ir` 一致；
- Gerber 反查 layer/outline/copper geometry 以及 soldermask 开窗与 silkscreen 图形，Excellon 反查 drill/slot geometry 与 plating 分类，IPC-D-356 反查 pad/testpoint/net identity；三者联合与被证明 board 核对。

KiCad DRC 是独立交叉检查，不取代自有 verifier：KiCad 检查其完整内部板模型和版本特定规则，自有 verifier 检查 RFC 固定的 Profile 与 proof predicates。两者都必须通过；意见不一致时失败并报告差异，不选择较宽松结果。

Obligation evidence 的 v1 闭合语义为：

- `formal`：连接、短路、声明的长度差、显式最小线宽等由 `ValidBoard` predicate 关闭。`DiffPairLengthSkew` 的长度定义在 v1 收窄为：每个 net 恰好两个 terminal，去掉 zone 后的铜图是唯一一条无分支路径（任何度 ≥3 的节点、zone 参与或多余 terminal 都报 `unsupported_topology` 而不是给出结论），长度 = 沿该路径的 track 段长之和 + 每个 through via 的 Profile 板厚；这一定义是几何子 RFC 的验收条件；
- `review`：仅限第 2 节 obligation registry 中 evidence 为 `review` 的 kind（registry 是唯一枚举，此处不重复列举，避免漂移），由指定 reviewer 签署。

Required obligation 既没有 formal proof 又没有对应人工 approval 时，最终状态是 `review_pending` 或 `failed`，绝不是 `ready`。Schematic review 在 `schematic_confirmed` 签署 obligation coverage 声明；该签名只表示人类确认“已声明集合覆盖本次自由文本意图”，不把自由文本自动提升为形式化命题。

### 9. Repair loop

每次失败只给 agent 最小、结构化、可行动的信息：稳定诊断码、stage、对象 id、坐标/层/网络、规则期望、实测值和允许的 mutation tools。不得把“重新设计整块板”作为默认修复。

推荐修复优先级：

1. source error：修改 `.cohdl` 后创建新 baseline 并重新执行所有 gate；
2. locked placement/outline error：停止并请求修改 Job 或源设计，agent 无权解锁；
3. placement collision/containment：仅移动未锁定实例；
4. routing connectivity：补线、移动合法 via 或局部 rip-up/re-route；
5. clearance/profile error：局部扩大间距/线宽/via，必要时 rip-up；
6. KiCad disagreement：保留候选，标记 `external_tool` failure，不反馈给 agent；
7. formal/artifact-binding failure：拒绝候选，禁止模型修改 proof 或 manifest。

无进展时的策略切换与停止条件以第 4 节的三次规则为准，此处不重复。Run 结束必须是 `ready`、`failed` 或 `review_pending` 之一；崩溃恢复后根据 immutable attempt 和 hashes 重放状态，不能从模型的聊天历史推断已经通过的 gate。

### 10. Final evidence and manufacturing package

所有 export 先写入 `attempts/<n>/export-staging/payload/`。`export_verified` 对 staging 中的实际 bytes 做语义 parse-back，随后生成 proof manifest；reviewer 审查并签署该不可变 payload root。只有 `artifact_bound` 与 `review_complete` 都成立，Harness 才把**完全相同的 bytes**原子 promotion 到 `final/`（先写满 `final.tmp/` 并校验 sums，再一次目录 rename；崩溃在 rename 之前等于没有 final）：

```text
final/
  job.json
  resolved-contract.json
  baseline-manifest.json       # 产生最终 board 的 winning baseline，含 baseline_id
  fab-profile.json
  profile-evidence/
  design.cohdl.snapshot/
  dependencies/sha256/
  cohdl.toml
  cohdl.lock
  design.lock
  proof-ir.json
  fabrication-order.json
  board.kicad_pcb
  board.kicad_pro
  board.kicad_dru
  footprints.pretty/           # 仅当 spike 选择本地 footprint library 路线
  fp-lib-table                 # 同上
  bom.csv
  gerbers/
  drill/
  netlist.ipc-d-356
  reports/cohdl.json
  reports/physical.json
  reports/kicad-drc.json
  reports/fab-profile.json
  proof/BoardCertificate.lean
  proof/axioms.txt
  proof/lean4checker.txt
  proof-manifest.json
  scope-classification.json
  approvals/schematic-<baseline_id>.json
  approvals/pre-order.json
  review-authorization-chain.json
  review-root.pub              # 非可信便利副本，仅在与包外 hash 相等时使用
  revocations/<snapshot_sha256>.json   # 每份 approval 引用的 snapshot 各一份
  sha256sums.txt
  terminal-receipt.json        # 只允许 ready；不进入 hash DAG，见下
```

Hash DAG 必须无环：`proof-manifest.json` 记录全部 leaf payload hash，但排除自身、`approvals/` 与 `sha256sums.txt`；detached approvals 按 role 签署不同 root（`schematic` 签 schematic-review-root，`pre_order` 签 manifest root）；sums 覆盖 payload、manifest 和 approvals 但排除自身与 `terminal-receipt.json`。receipt 分两级：run 根目录的 `run-receipt.json` 记录三种终态之一（`ready|review_pending|failed`）；`final/terminal-receipt.json` 只在 `final/` 存在时存在，因此 `state` 只能是 `ready`（`final/` 只在 `review_complete` 后创建，失败 corpus 要求它不存在）。两者 schema 相同：`state`、`last_completed_stage`（ladder 中的 rung 名，必填）、`manifest_sha256`、每个 approval 文件的 sha256、role → `revocation_snapshot_sha256` 映射、`baseline_id`、verifier 版本。除 `state`、`last_completed_stage` 与 verifier 版本外，每个字段在其产生 rung 未到达时为 `null`（`baseline_id` 在第一个 draft baseline 提交后才非空，`manifest_sha256` 在 `export_verified` 后才非空，approval 字段在对应签署后才非空），字段必填性由 `last_completed_stage` 与已提交的记录集合（`draft-commit.json` 与 `baseline-commit.json` 两种）共同推导，因为 draft baseline 的提交不是 ladder 中的独立 rung：存在任一 draft 或冻结提交时 `baseline_id` 必填并指向最新一个；源修复创建新 baseline 时 `last_completed_stage` 回退到最后一个 run 级 rung `fabrication_order_frozen`，随后只记录新 baseline 实际通过的最后一级（新源码若解析失败，阶段停在 `fabrication_order_frozen`，不会被误记为 `source_valid`），后续字段按新 baseline 重新推导。receipt 是可重新生成的派生缓存：它不签名、不被 manifest 或 sums 引用，改动它不会使 manifest/approval/sums 失效，只会让 `verify` 的重新推导与之不一致而报告失败。`schematic` review 针对冻结的 logical baseline 与 scope classification，`pre_order` review 针对 staging 中的最终制造 payload bytes。

Approval v1 使用 Ed25519，签名字节为 `"cohdl-agent-approval-v1\0" || target_sha256 || obligations_sha256 || role || revocation_snapshot_sha256`；digest 是 raw 32-byte，role 是 `schematic|pre_order` ASCII enum，`target_sha256` 按 role 固定：`schematic` 签 schematic-review-root，`pre_order` 签 `proof-manifest.json` 的 sha256；每份 approval 文件自带 `baseline_id` 与它签署时的 `revocation_snapshot_sha256`。包外 organization root public-key hash 是信任锚；bundle 的 `review-authorization-chain.json` 是 canonical chain **set**，包含 `schematic` 与 `pre_order` 两种 role 各自所需的全部链（两份审批可以由不同 key、不同授权签署），不含 root 本身；root 的便利副本另存为 `review-root.pub`，不可信。Revocation snapshot 也由 root 签署，含 authority、单调 sequence、as-of 与 previous hash。`schematic` 与 `pre_order` 在不同时刻签署，期间 sequence 可能更新，所以 final 为每份 approval 保存它引用的那份 snapshot（`revocations/<snapshot_sha256>.json`），历史 snapshot 只用于验证各份签名字节在签署时有效；最终判定另有一条规则：`final_snapshot` 是被引用 snapshot 中 sequence 最大的一份（必须是 `pre_order` 引用的那份），`review_complete` 用它重新检查两种 role 的授权，`schematic` 签署者若在 `final_snapshot` 中已被撤销，其 approval 即使按历史 snapshot 验签通过也不再满足 gate，状态回到 `review_pending`，需要另一位有效签署者对同一 review root 重新签署（root 与 baseline 不变）。`verify` 可要求 `final_snapshot` 的最小 sequence。`verify` 必须从本地 policy 或 `--review-root-key FILE` 获得 root 公钥 bytes 并校验其 sha256，并可要求最小 sequence；绝不信任包内自声明 root。离线 verdict 只相对于绑定 snapshot 有效，后续撤销需联网另查。

`proof-manifest.json` 使用 canonical JSON，排序固定，不含 wall clock。时间、延迟、token 和费用进入非身份性的 `run-metadata.json`；原始 KiCad 日志也可旁存，但不进入 reproducible identity。Manifest 记录 schema、job/source/dependency/profile/board/tool hashes、到 `export_verified` 为止每个 gate 的 canonical report hash、Lean theorem、axiom audit、所有 obligation 及证据状态。`artifact_bound`、`review_complete` 与 `ready` 三个终态不进入 manifest（否则形成自环）：它们由 manifest 校验、detached approvals 与一份不进入 hash DAG 的 terminal receipt 派生。依赖 package bytes 随包保存，保证没有 registry cache 的第三方仍可离线重放。

首版 `ready` 只表示裸板制造包；BOM 不推出 PCBA-ready。制造流水线固定为 `raw export → allowlisted canonicalization → parse canonical bytes → semantic compare → leaf hash → manifest`。Canonicalizer 只处理 schema 列明的 wall-clock/path metadata；DRC date/path 与 Gerber/drill creation time 必须规范化。CI 跨时刻重复 export 比较 payload bytes。离线 `verify` 只读 final 内副本，不重新解释原始相对路径；任何后改 byte 使 manifest、approval 与 sums 失效。

### 11. CLI and machine interface

首版 CLI：

```text
cohdl-agent run JOB --review-root-key FILE [--model ID] [--run-dir DIR] [--json]
cohdl-agent resume RUN_DIR --review-root-key FILE [--json]
cohdl-agent verify RUN_DIR --review-root-key FILE [--json]
cohdl-agent explain RUN_DIR [--attempt N]
```

- `run` 创建冻结输入并执行完整循环；默认 attempt cap 来自 Job。`max_attempts` 在整个 run 内跨 baseline 累计，authoring 阶段的每个 draft baseline 也计一次；换 baseline 不重置计数。
- `resume` 只从已验证 hashes 与最后完整 attempt 继续。
- `verify` 不调用模型，只重放全部适用 gate；这是制造前和第三方复核入口。
- `explain` 输出 verdict ladder、失败 owner、证据路径和下一步，不改变状态。
- `--json` 只输出一个 versioned JSON document 到 stdout；日志进入 stderr。退出码：0=`ready`，1=`failed`，2=invocation/job error，3=`review_pending`。

模型 adapter 与 board backend 是内部 trait/protocol，不进入 Job 的正确性语义。更换 GPT、Claude、专用 placer 或 autorouter 只能改变搜索效率和候选质量，不能改变 validator 结果。离线 `verify` 永远不访问模型或网络。

### 12. Diagnostics

Harness 使用与 compiler E/D codes 分离的稳定命名空间：

| Family | Meaning |
|---|---|
| H0xx | invocation、Job schema、路径和 hash |
| H1xx | agent/backend refusal、timeout、malformed tool call |
| H2xx | attempt/action budget、重复无进展、resume state |
| P1xx | placement、outline、courtyard |
| P2xx | routing、connectivity、short、clearance、width、via、zone |
| F1xx | Profile、KiCad DRC、export、artifact binding |
| V1xx | Lean parse、proof、axiom audit、unsupported formal construct |
| R1xx | required human review、obligation coverage 与 approval |

每条诊断必须有 code、severity、stage、owner、稳定 object id、期望值、实际值、证据 span/geometry 和可允许的修复动作。坐标诊断同时提供机器精确值和人类单位显示。错误码只废弃、不复用。正式 code 表由后续 diagnostics contract 子 RFC 固定；在该表落地前 RFC-032 保持 Proposed，不能以临时字符串形成外部兼容承诺。

## Type-system-first test

本 RFC 引入大量检查，但没有一条应无差别进入传统 DRC：

- 名称、单位、trait、variant、required pin、part binding、非空 footprint 的 pad consistency 等**局部结构事实**已经由 CoHDL type/build checks 处理，Harness 必须复用并不得重写第二套宽松规则。现有 build 对空 placeholder footprint 的例外不满足制造终点，因此 `FabricationReadyFootprints` 必须额外拒绝它。
- 电压、极性与 driver 数量等**从完整逻辑网络涌现的电气事实**继续由 RFC-004 的四条 residual DRC 处理。
- 铜形状相交、网络物理连通、元件 containment、courtyard overlap、间距、via 与 edge clearance 是**整板、跨对象、数值几何事实**。它们无法表达成某个 declaration 的 trait bound 或 pin obligation，因此属于新的 physical verification stage。
- Fab Profile 判断是同一物理谓词在固定制造参数下的实例化，不是新的 CoHDL `rule` 语法。
- Lean 不新增用户可写规则；它证明上述 checker 对精确定义的 predicate sound，并对实际候选执行实例证明。

因此本设计遵守“type system over DRC over review”的顺序，同时拒绝把不适合类型系统的二维几何硬塞入类型机制。

## Conceptual impact

**High.** 本 RFC 新增八个长期概念：`PCB Job Contract`、run 级 `Resolved Contract`、immutable `Baseline`、content-addressed `Candidate`、`Routed Board IR`、`Fabrication Profile`、内容寻址的 `Board Class`、以及由 `Proof Manifest + detached Approvals + Scope Classification` 组成的 evidence root，并把 Agent Harness 从演示脚本提升为有正式 verdict 的 partner-layer 系统。成本真实且必要：没有这些边界，“完整 PCB”只能依赖 GUI 状态、模型自报和工厂上传后的偶然反馈，无法 grade。

概念边界刻意保持正交：

- Job 说明本次运行要什么和不能改什么；不复制 netlist。
- Baseline 冻结一次 source/compiler projection；Candidate 只表示一组 exact board bytes，两者都不可变。
- CoHDL Logical IR 说明器件、电气连接和显式意图；不承载铜线。
- Routed Board IR 是实际 KiCad 候选的验证投影；不成为第二种手写 PCB 语言。
- Profile 提供外部制造参数；不携带某块板的数据。
- Proof Manifest 绑定输入、候选、检查与输出；detached Approval 把 reviewer 绑定到各自 role 规定的 root（schematic-review-root 或 manifest root），不影响任何设计语义。

“proof”只表示命题在声明的模型与假设下被 Lean 接受，不与“功能已全面验证”或“工厂保证生产成功”混用。

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| High | Low | Crit | High | High | Low | Crit |

- **Concepts (High)：** 八个新概念均对应不可替代的责任边界；RFC 和教学材料必须始终使用同一组名称。
- **Oracle (Crit)：** `ready` 成为比 `cohdl build pass` 更强的新 verdict。缓解方式是单调 ladder、每级独立证据、无 force-pass、失败即停止。
- **Diagnostics (High)：** repair loop 完全依赖稳定机器诊断；因此独立 namespace、owner、object id 和修复动作是外部契约。
- **Netlist (High)：** placement/routing 绝不能改变 logical connectivity。通过 parse-back、`BoardMatchesLogical` 和 cross-net copper proof 门禁。
- **Trust (Crit)：** 系统声称可制造。缓解方式是不信任 agent、双 verifier、Lean proof、artifact hash binding、显式 assumptions 与人工义务。
- **Grammar (Low)：** 不增加 CoHDL syntax；Job/Profile/manifest 是独立 versioned schema。
- **Compat (Low)：** 新工具与新输出目录均为 additive；现有 `cohdl check/build` verdict 和字节不变。

## Gradeability

完成条件不是“板看起来合理”，而是以下可重放断言全部成立：

1. 固定 Job 和 source snapshot 能产生 clean CoHDL JSON verdict 与 byte-stable `proof_ir`。
2. 候选 KiCad 文件能完整解析为受支持 Routed Board IR。
3. component/pad/net projection 与 Logical IR 双向一致。
4. 未布网络计数为 0；每个 required net 的 pad 在实际铜图上连通。
5. cross-net short、Profile clearance、width、via、edge、containment 与 overlap 错误均为 0。
6. 固定 KiCad CLI DRC 的 error、warning、excluded violation 均为 0。
7. Lean 接受绑定六类实际 bytes 的 `ValidBoardBytes`，axiom audit 与 `lean4checker --fresh` 通过；`formal_valid`（含 `lean4checker --fresh`）的时间预算由 Decision 所列的 Lean 性能 spike 在 qualification board 上实测后固定，并连同 runner class 与 board 规模写入本 RFC；预算落地前本项不能记为通过。超出预算即视为第 6 节所说的"不可接受"，触发 certificate 路线而不是放宽 axiom policy。
8. Gerber/Excellon/IPC-D-356 与被证明候选语义核对，随后全部 hash 匹配。
9. 两份 required review 各自签署规定的 root（`schematic` 签 schematic-review-root，`pre_order` 签 manifest root），并关联同一 winning baseline 与 obligation set；所有已声明 required obligation 有 disposition，schematic reviewer 已签 coverage 声明。

三个规范性端到端 scenario 必须全部通过：

| Scenario | Input/first result | Required transition | Terminal state |
|---|---|---|---|
| First-pass clean | fixture backend 首次给出完整合法候选 | 依次通过全部 rung，无 repair | `ready`，存在 final bundle |
| Repair succeeds | 首候选有一条 0.10 mm track，命中稳定 Profile code | 保留 parent digest，仅以允许 mutation 生成新 digest 并全量重验 | 第二 attempt `ready`，动作链可重放 |
| Impossible constraints | locked connectors 与闭合 outline/courtyard 约束不可同时满足 | 诊断 owner 为不可由 agent 解锁的 placement contract；不得扩板或移动 lock | `failed`，不存在 final bundle |

扩展基准还必须包含以下 closed failure corpus：板框放不下、locked connector 被移动、一个未布 net、一个跨网短路、一条过细 track、一个过小 via、一个 copper-edge 违规、一个只改了候选 Edge.Cuts 而源码/DXF/锁定坐标全部不变的候选、一个只改了板内 `(setup …)` 值的候选、一个 placeholder footprint、一个 KiCad-only DRC 失败、一个 artifact 被篡改、一个 Lean proof/axiom 失败、一个 attempt-cap 失败。每个失败必须命中唯一稳定 code，并且最终目录不存在。

此外需要 metamorphic/property checks：平移未锁定的合法局部块不改变 logical net；输入顺序变化不改变规范化 verdict；同一候选验证两次时由 Harness 生成的 canonical report bytes 相同（外部工具的原始日志不进入 identity）；修改任一被绑定输入 byte 必使 proof manifest 校验失败；向 KiCad 文件加入不支持 construct 必须拒绝而不是忽略。

## AI-generatability

Agent 面对的是少量正交工具和局部诊断，不需要背诵 KiCad S-expression、Lean proof term 或完整厂商表。它选择坐标、层、宽度和路径；tool schema 检查单位、对象身份和下限；validator 返回精确冲突对象。这比“打开 GUI 看哪里红了”更适合模型，也比一次性要求生成完整 board bytes 更容易修复。

## Alternatives

- **只增强 system prompt。** 拒绝。Prompt 可以提高首稿质量，不能提供 completeness、artifact binding 或不可绕过的 verdict。
- **只运行 KiCad DRC。** 拒绝。它不替代 CoHDL 的单位/trait/pin/part 语义，也不固定本 RFC 的工厂 Profile、Job locks、proof assumptions 或 artifact provenance。
- **把全部 physical checks 加进 `src/drc.rs`。** 拒绝。它违反 RFC-004 的四规则边界，把逻辑网络 DRC 与物理制造验证混为一谈。
- **在 CoHDL 中新增完整 routing DSL。** 拒绝。永久 grammar/teaching cost 极高，也与既有“layout 是 partner concern”的架构边界冲突。受支持 KiCad subset + 验证投影足以关闭当前需求。
- **扩展 `cohdl docs` 的 API-docs JSON 充当 `proof_ir`。** 拒绝。`src/emit/docsjson.rs` 已是确定性投影，且含展开声明、pad/footprint 几何 hash 与 intents，但它是 package 级文档（按声明组织，含 `foreign` 段，为 registry 搜索投影服务），不是一次 build 的 design 级展开结果：没有 designator、展开后的 instance/net/NC 与 placement provenance。让文档格式同时承担证明输入会把两个变更节奏不同的契约绑在一起；`proof_ir` 可以复用 docsjson 的几何序列化与 `emit::silk` 展开，但必须是独立 schema。
- **用 Lean 重写整个 compiler、placer 和 router。** 拒绝作为首版。证明搜索算法远比验证有限候选昂贵，并扩大交付周期；proof-gated untrusted search 已能获得所需保证。
- **首版采用通用或多工厂 Profile。** 拒绝。规则交集会过度保守，按字段配置又会扩大组合状态；单一、内容寻址 Profile 最容易 grade。
- **使用工厂在线 DFM 作为唯一门禁。** 拒绝。在线结果不可完全复现、需要网络并可能随服务变化；可以记录为附加证据，不能替代本地固定 verifier。

## Compatibility

RFC-032 是 additive：

- `.cohdl` grammar、AST、Logical IR 语义和现有 E/D error codes 不变。
- `cohdl check`、默认 `cohdl build`、现有 emitters 与既有 artifact bytes 不变。必要的 `--emit proof_ir` 是 additive 选项，须先经独立 compiler RFC 接受。
- Rust compiler 不增加 Lean、KiCad、JSON library、LLM SDK 或 routing dependency。
- 现有 `harness/repair_loop.py` 继续作为 v2 MVP 的历史证明；新 harness 使用新命令和 run directory，不静默改变旧脚本语义。
- 现有 `.kicad_pcb` emitter 仍是 placement/net-bound starting point；新 harness 将它复制到 attempt workspace 后再物理完成，绝不在 `out/` 中原地布线。

未来若 CoHDL 新语法表达更多 physical constraints，必须经过独立语言 RFC；Harness 通过 versioned Logical IR/schema 消费，不能先实现隐藏语义再倒逼 grammar。

## Tooling & operations

- 所有 toolchain 版本固定并进入 manifest：CoHDL release hash、KiCad **10.0.4**、Lean toolchain、formal definitions/import allowlist、`PadPlan` schema version/hash、board setup 模板 hash、Profile hash、backend adapter version。
- `verify` 完全离线；generation 网络访问与 verification 分离。缺少本地依赖或工具是 hard failure。
- Prompt、模型回复、tool calls、diagnostics 和 candidate hashes 进入 append-only `actions.jsonl`；secret、token 和环境变量值必须在写盘前结构化删除。
- KiCad 在空隔离目录运行，同 stem 放置 `candidate.kicad_pcb/.kicad_pro/.kicad_dru`；拒绝其他邻接规则文件，固定 env/locale/timezone。Manifest 绑定三者与 executable/adapter hashes；canonical report 记录 staged rules hashes 与 severity map hash；在 Decision 所列 spike 二选一之前 ignore 的 test 为零。若 spike 选择 severity map 路线，ignore 的 test 恰好等于写回本 RFC 的清单；若选择本地 footprint library 路线，`.pretty` 目录与 `fp-lib-table` 同样进入 staging、manifest 与 final bundle，并与 board 同 stem 隔离。集成测试含一个仅固定 `.dru` 才触发的 canary，防止退回默认规则仍通过。
- JLCPCB Profile 来源固定为官方 rigid PCB capability evidence snapshot；抓取只进入 Profile 更新流程，普通 `run/verify` 不联网。
- Lean formal 工程使用 pinned `lean-toolchain`，首版优先仅依赖 Lean/Std；新增 mathlib 或 FFI 需要独立 trust-impact review。
- Harness 作为独立 crate 落在仓库新目录 `agent/`（`harness/` 已被 MVP 演示脚本占用），与 `explorer/`、`registry/` 同一地位：在 compiler 零依赖规则之外，有自己的 CI job。现有 CI 的 test、vscode-extension、registry、site、explorer 之外新增 harness、formal、KiCad integration 三个 job。Compiler release 不因开发环境缺少 KiCad/Lean 而增加运行依赖；harness release 必须通过全部四类门禁。

规范性外部参考：

- KiCad CLI PCB DRC（版本化 10.0 文档；spike 同时保存实际 10.0.4 `kicad-cli pcb drc --help` 快照）：<https://docs.kicad.org/10.0/en/cli/cli.html>
- JLCPCB rigid PCB capabilities：<https://jlcpcb.com/capabilities/pcb-capabilities/>
- Lean proof validation：<https://lean-lang.org/doc/reference/latest/ValidatingProofs/>
- Lean decision procedures：<https://lean-lang.org/doc/reference/latest/Tactic-Proofs/Tactic-Reference/>

## Teaching cost

人类与 agent 需要学习：

1. `cohdl build pass` 只表示 logical validity；
2. run、baseline、candidate、attempt、staging 和 final 的区别；
3. 一个固定 Fab Profile 与 `fabrication-order.json`，而不是整张工厂能力表；
4. `ready` ladder、obligation coverage 与 detached approval；
5. `proof_ir`、TCB、proof 命题、外部假设和 artifact hash 边界；
6. 只能通过 typed mutation tools 改物理候选，candidate id 就是 bytes digest。

Agent 不需要学习 Lean tactic、KiCad 文件语法或厂商网页。硬件 reviewer 需要能读 `resolved-contract.json`、Profile 摘要、verdict ladder 和 obligation；不要求阅读 generated proof term。教学材料必须始终使用 `logical_valid`、`fab_profile_valid`、`formal_valid`、`ready` 四个不同术语，禁止统称为 “valid”。

## Failure modes

- **模型输出看似完整但漏布一条网。** `EveryRequiredNetConnected` 与 KiCad ratsnest/DRC 双重拒绝。
- **Zone 声明存在但未 refill。** 固定 KiCad 在副本上 refill/save，保存 bytes 得到新 digest；其后所有 gate 只验证新候选。
- **遇到不支持的 KiCad construct。** hard-fail `unsupported_construct`，不忽略未知节点。
- **模型连续局部修复振荡。** 归一化 failure signature 三次触发策略切换/停止。
- **外部工具崩溃或版本不符。** `external_tool` failure；不得沿用旧报告。
- **自然语言要求没有结构化表达。** Harness 不自行猜测其含义；schematic reviewer 若不能签署 obligation coverage，则 `review_pending` 或失败。
- **设计含 v1 排除特征（RF、市电）却通过了全部机器 gate。** footprint family 推不出电路用途；防线是 reviewer 签署的 `scope-classification.json`。若声明为假，Lean certificate 本身仍可能成立，失效的是整体 `ready` 声明的范围假设；scope classification 由 artifact binding 与 human gate 保证，不是 Lean 定理，责任在签署者。
- **通过制造规则但电气设计不合理。** Certificate 只声明已建模命题；模拟、SI/PI、热/EMI obligation 仍需工具或人工证据。

## Migration path

**Source migration: N/A.** 这是 additive partner tool；现有 `.cohdl`、manifests、locks、`cohdl build` 输出均不迁移，旧 `harness/repair_loop.py` 继续原语义。只有选择创建 PCB Job 的项目进入新流程。

Implementation staging 分五个可独立验收的子项目推进：

1. **Job/verdict/evidence contract**：canonical schemas、immutable attempts、offline `verify`、无 agent 的失败 fixtures。
2. **Routed Board IR and physical verifier**：受支持 KiCad 10 subset、parse-back、connectivity/geometry/Profile checks。
3. **Agent orchestration and typed mutations**：placement/routing adapters、局部 repair、预算与停止条件。
4. **Lean proof core and artifact binding**：formal predicates、sound checkers、Lean-side parse、axiom audit、per-board certificate。
5. **Manufacturing package and real-board qualification**：固定 KiCad export、JLC Profile、成功/失败 corpus、真实上传前人工 checkpoint。

凡改变公共 schema、verdict、diagnostic、IR、Profile 或 trust boundary 的子项必须先有 Accepted RFC；agent-spec 只实现已接受设计，不能代替 RFC。每项还需明确 Allowed Changes、确定性 tests 和 exception scenarios；schema 变更必须版本化。

现有两个 example board 和 OpenMicroKBD revisions 可作为 importer/emitter fidelity fixtures，但首个端到端 qualification board 应是一个专门限制在 `mcu_iot_control` v1 的小型 sensor node。它必须真实完成 placement/routing、通过全部自动 gate，并由工程师在 KiCad 与工厂上传预览中完成 checkpoint。历史手工 routed board 只作差分参考，不自动成为 ground truth。

## Decision

**待审。** 头部 lifecycle status 为 `Proposed`；RFC 流程的最终决定尚未作出。

本 RFC 记录当前讨论形成的推荐架构：单一 `jlcpcb-standard-2l-v1` 裸板 Profile；首版常规 MCU/IoT 双层裸板；agent 是不可信搜索器；最终 PCB 由 pinned CoHDL projection、physical verifier、KiCad DRC、Profile DFM、Lean proof、semantic export verification、artifact binding 和显式 human review 共同 gate；首版证明候选相对于 TCB 与已声明假设有效，不证明 compiler、agent、placer 或 router 算法。

它不能自行标记为 Accepted。接受前必须完成：

- 在 conol.ai live source-of-truth 中预留编号、审查并同步本提案；本地 `RFC-032` 只是候选编号；
- 明确确认 `cohdl-agent` 是 Constitution 允许的 partner-layer tool；若要成为 compiler 内置 P&R，则先通过 Goal Change Proposal；
- 对八个新核心概念完成 coherence regression；
- 决定子 RFC/Task Contract 的编号与所有权；
- 完成五个阻塞 spike：canonical `proof_ir`、KiCad net ordinal + zone normalization、formal geometry 子集、staging/export/hash root、以及用 KiCad 10.0.4 对两个 example board 跑 `--severity-all` DRC，据实测在「hash 固定的 `.kicad_pro` severity map + 闭合 ignore 清单」与「生成本地 footprint library、零 ignore」两条路线中二选一写回本 RFC（当前机器没有 KiCad 10.0.4，该 spike 未开始）；
- 用一个现有 `.kicad_pcb` 验证 Lean 六类 bytes parser/certificate 的性能与 axiom policy，并据此固定 Gradeability 第 7 项的时间预算、runner class 与 board 规模；
- 补齐 machine-readable Profile 的全部适用规则、`fabrication-order.json`、官方 evidence snapshot 与成功/失败 fixtures。

在这些门禁完成前，本文件只是一份可审查的 Proposed draft，不改变语言、compiler verdict、发布承诺或制造责任。
