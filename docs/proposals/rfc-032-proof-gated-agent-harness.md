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
- **不支持首版高级板型。** BGA、HDI、盲埋孔、microvia、via-in-pad、DDR、RF matching、天线、市电、刚挠结合、高铜厚、受控阻抗保证和 panelization 均排除。
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
  "board_class": "mcu_iot_control_v1",
  "fab_profile": {
    "id": "jlcpcb-standard-2l",
    "version": 1,
    "sha256": "<64 lowercase hex>"
  },
  "locked_instances": ["SensorNode::usb", "SensorNode::mount_1", "SensorNode::mount_2"],
  "allowed_sides": ["top", "bottom"],
  "obligations": [
    {
      "id": "usb2-routing",
      "kind": "diff_pair_geometry",
      "params": {"positive_net": "USB_DP", "negative_net": "USB_DM", "max_skew": "0.25mm"},
      "evidence": "formal",
      "owner": "harness",
      "required": true
    },
    {
      "id": "usb2-impedance",
      "kind": "impedance_process_review",
      "params": {"target": "USB_DP,USB_DM", "nominal": "90ohm", "tolerance": "10percent"},
      "evidence": "review",
      "owner": "hardware_reviewer",
      "required": true
    }
  ],
  "required_reviews": ["schematic", "pre_order"],
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
- `locked_instances` 使用展开后的 stable hierarchical instance id，只能指向 CoHDL 中已经显式 `place` 的实例。位置、旋转与 side 从 CoHDL 读取，Job 不复制坐标。
- `intent` 完全是非规范化文本，只供生成和人工审查，永远不能直接生成 pass/fail。所有机器判定事项必须出现在闭合的 `obligations[]` 中；agent 可以提议 obligation，但 schematic reviewer 必须签署“集合已覆盖本次意图”的声明。
- obligation-kind registry 是 versioned closed enum；未知 kind、未知/缺失 param、错误 evidence/owner 均使 `job_valid` hard-fail。v1 只有下表六项，新增 kind 必须经 RFC：

| kind | params schema | evidence / owner | sole verifier |
|---|---|---|---|
| `diff_pair_geometry` | `positive_net, negative_net, max_skew: Length` | `formal / harness` | `DiffPairGeometry` |
| `impedance_process_review` | `target, nominal: Resistance, tolerance: Percent` | `review / hardware_reviewer` | signed `pre_order` review |
| `compiler_warning_review` | `code, object_id, reason` | `review / hardware_reviewer` | signed `schematic` review |
| `analog_stability_review` | `instance_ids, document_hashes` | `review / hardware_reviewer` | signed `schematic` review |
| `thermal_review` | `instance_ids, ambient: Temperature` | `review / hardware_reviewer` | signed `pre_order` review |
| `emi_review` | `object_ids, document_hashes` | `review / hardware_reviewer` | signed `pre_order` review |

未声明的自然语言含义既不被证明，也不被暗中当作 pass。
- Job 解析后生成 canonical `resolved-job.json`，补入 source、manifest、dependency lock、DXF 与 toolchain 的哈希。后续 attempt 只引用该冻结对象。

`mcu_iot_control_v1` 允许常见 THT/SMD、0402 及以上无源器件、0.4 mm 及以上引脚间距的 QFP/QFN、USB 2.0、晶振、LDO、低压 switching converter、连接器和双面元件放置；这描述设计子集，不代表贴装服务资格。USB 2.0 的差分几何与长度可以检查，但双层 Profile 不声明受控阻抗保证，因此 `#[impedance]` 必须对应一条 review obligation，或切换到未来的受控阻抗 Profile。

本 RFC 不新增 `.cohdl` grammar；下列现有 source 片段说明 Job 锁定的事实仍来自语言本身，而不是复制进 JSON：

```cohdl
layout {
    place usb at (6mm, 12mm) rotate 90 side top
    place mount_1 at (3mm, 3mm)
}
```

### 3. Immutable attempts and mutation tools

Run 内每次 source 修复产生新的不可变 baseline，每个候选按实际 bytes 内容寻址；attempt 只保存引用和动作链：

```text
runs/<run-id>/
  input/
    job.json
    resolved-job.json
    fab-profile.json
    fabrication-order.json
  baselines/0001/
    source-snapshot/
    dependencies/sha256/
    proof-ir.json
  objects/sha256/
    <digest>.kicad_pcb
  attempts/0001/
    attempt.json
    actions.jsonl
    reports/
    export-staging/
  final/
    proof-manifest.json
    sha256sums.txt
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

如果 CoHDL source、manifest、lock 或 DXF 在一次修复中变化，旧 baseline、候选和报告继续保留，但 routing、proof、schematic coverage 与 approval 全部失效；Harness 创建新的 baseline id，从 compiler 阶段重新开始。Routing 修复不得顺便换 part；placement 修复不得改变 net；proof failure 不允许通过 prompt 争辩或降级。

### 4. Verdict ladder and state machine

Machine state 和 rung 名称统一使用 `snake_case`。每个 attempt 按以下固定顺序运行：

```text
job_valid
  ⊂ fabrication_order_frozen
  ⊂ source_valid
  ⊂ logical_valid
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

- `job_valid`：schema、路径、哈希、board class、Profile、closed obligations 和预算合法。
- `fabrication_order_frozen`：由固定 Profile 生成 canonical order bytes并冻结为 input leaf；任何变化使全部后续 rung 与 approval 失效。
- `source_valid`：项目可离线解析，依赖与 lock hash 验证通过；所有 compiler warning 均已映射到显式 obligation。
- `logical_valid`：CoHDL parses/resolves/type-checks/connects，四条 residual DRC 无 error，build-only part/footprint checks 通过，并由单独接受的 additive emitter 生成 canonical `proof_ir`。所有 populated electrical instance 必须有非空、非 placeholder footprint，且 geometry hash 与冻结依赖一致；mechanical-only 对象必须显式分类。
- `board_normalized`：从 logical net name 的 bytewise sort 确定唯一 net ordinal，补齐 `(net ordinal name)`；pad 同时携带一致的 ordinal/name，track、via、zone 只引用已知 ordinal。重复、未知或 name/ordinal 不一致均 hard-fail。
- `placement_complete`：每个实例有确定 position/rotation/side，locked placement 未变化；emitter shelf/grid 坐标带 `staged` provenance，只有被显式 placement action/solver 接管且 courtyard 位于允许区域后才算完成。
- `routing_constructed`：每个 required net 已有 route plan，zone declaration 存在但尚不用于连通性结论。
- `zone_materialized`：固定 KiCad 10.0.4 在候选副本上执行 `pcb drc --refill-zones --save-board --exit-code-violations`。这是 transform + preliminary report：exit 0 或 5 且产出可解析 board 时形成**新** digest，5 中的 violations 进入 repair；加载/参数/保存失败才是 `external_tool`。此后所有 gate 只检查新 digest，证明后禁止再次 save。
- `routing_complete`：从 materialized copper 计算 terminal connectivity 和 ratsnest；不能把 zone 声明当作铜。单 terminal net 没有 routing obligation；一个 logical pin 展开出的所有 required physical pads 都属于 terminal set。
- `physical_valid`：板框、同面 courtyard、keepout、铜短路、clearance、track/via geometry、层和 dangling copper 的闭合谓词集成立。
- `fab_profile_valid`：所有适用制造规则满足被哈希固定的闭合 Profile；不存在“Profile 未列即通过”。
- `kicad_drc_valid`：固定版本运行 `kicad-cli pcb drc --format json --severity-all --exit-code-violations`。工具执行状态与 policy verdict 分开记录；v1 不允许 DRC waiver，board 内 exclusion 不继承，error、warning、excluded/ignored/disabled test 均须符合固定的零项 policy。
- `formal_valid`：Lean 接受绑定 Job、Profile、`proof_ir`、fabrication order 与实际 board bytes 的 `ValidBoardBytes`；certificate 中的 order bytes 必须等于 final `fabrication-order.json`。
- `export_verified`：Gerber、Excellon 与 IPC-D-356 均从 staging 反向解析，并分别与已证明 board 核对 layer/outline/copper geometry、drill/slot/plating 分类和 pad/net identity；hash 不能替代这一语义检查。若 parser 未实现，最高状态只能是 `formal_valid`。
- `artifact_bound`：所有已验证 payload 的无环 hash DAG 完整，且未在验证后改 byte。
- `review_complete`：所有 required human review 签署同一 payload root 与 obligation set；未完成时状态只能是 `review_pending`。
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

达到 attempt/action 上限、工具超时或工具缺失时，run 以 `failed` 结束。同一 `(failure_signature, strategy_id)` 连续出现三次后只允许切换到下一个预先枚举的 strategy；没有下一 strategy，或新 strategy 再达到三次，立即 `failed`。所有状态保留证据，不继续无界循环。

### 5. Routed Board IR

Routed Board IR 是从实际 `.kicad_pcb` bytes 解析出的规范化验证模型，不是新的手工 authoring format。首版只接受 harness mutation tools 能产生的 KiCad 10.0.4 子集：straight track、through via、materialized polygon zone，以及 circle/rect/oval/roundrect pad；track arc、custom/chamfer pad 与其他 copper construct 均 fail closed。遇到未知、无法语义保持的 KiCad construct 必须报 `unsupported_construct`，不得忽略。

最小模型包括：

- board outline 的 line/arc 闭合轮廓；
- component identity、designator、footprint、position、rotation、side、courtyard；
- pad 的真实变换后几何、层、plating、drill 与 net；
- straight track segment、through via、filled zone polygon 与 net；
- keepout、Edge.Cuts、copper/soldermask/silkscreen 必需层；
- net 与 logical pin/pad 的 provenance；
- board setup 中参与 DRC 和制造的规则值。

KiCad decimal 精确解析为有理数，不得用平台 `f64`。若采用整数近似，必须使用有方向证明的双包络：内包络只能证明正向连通，外包络用于拒绝短路/间距违规；containment 比较对象外包络与合法区域内包络。几何子 RFC 必须逐 shape 给出近似结果到 KiCad 实际几何的方向性定理，并固定旋转误差、开闭边界、可靠接触裕量、共享 conductive layer、via/PTH 跨层语义，以及 zone hole/island/thermal 集合语义。输出相邻结构显式排序。

### 6. Formal invariants

独立 `formal/` Lean 工程定义 `BoardJob`、`FabProfile`、`LogicalIR`、`RoutedBoard` 与 `ValidBoard`。首版证明最终候选相对于**固定 compiler projection 与 Profile assumptions**有效，不证明搜索算法，也不宣称 source → IR compiler soundness。最终入口绑定所有规范输入 bytes：

```lean
def ValidBoardBytes
    (jobBytes profileBytes logicalIrBytes fabricationOrderBytes boardBytes : ByteArray) : Prop :=
  match parseJob jobBytes, parseProfile profileBytes,
        parseLogicalIR logicalIrBytes, parseFabricationOrder fabricationOrderBytes,
        parseBoard boardBytes with
  | .ok job, .ok profile, .ok logical, .ok order, .ok board =>
      ValidBoard job profile logical order board
  | _, _, _, _, _ => False

def ValidBoard (job : BoardJob) (profile : FabProfile)
    (logical : LogicalIR) (order : FabricationOrder)
    (board : RoutedBoard) : Prop :=
  ProfileIdentityMatches job profile ∧
  FabricationOrderMatchesProfile profile order ∧
  UniqueDesignators logical ∧
  RequiredPinsResolved logical ∧
  FabricationReadyFootprints logical ∧
  PartFootprintPadExact logical ∧
  BoardMatchesLogical logical board ∧
  LockedPlacementsPreserved job board ∧
  AllowedSidesRespected job board ∧
  OutlineWellFormed board ∧
  PlacementsInsideOutline board ∧
  SameFaceCourtyardsDisjoint board ∧
  KeepoutsRespected board ∧
  EveryRequiredNetConnected logical board ∧
  NoUnexpectedConnectivity logical board ∧
  NoCrossNetCopperIntersection board ∧
  CopperLayeringValid board ∧
  NoDanglingOrUnnettedCopper board ∧
  ZonesMaterialized board ∧
  FabricationRulesSatisfied profile board ∧
  FormalObligationsSatisfied job logical board ∧
  SupportedConstructsOnly board
```

这是 v1 的闭合 predicate set；physical/Profile/Rust report 的每一 pass 字段必须映射到其中一个 predicate，不能另有隐藏 verdict。Profile tuple → `ProfileIdentityMatches`，下单 exact choices → `FabricationOrderMatchesProfile`，locks/sides/formal obligations 分别映射同名 predicate；review obligations 由 review gate 关闭。v1 不允许 machine-check waiver；人工 review 不能覆盖上述任一 false predicate。

每个 predicate 必须同时提供：可读的 `Prop`、可执行的 `Decidable`/Boolean checker、以及 checker soundness 定理。Rust 可以提供加速 witness；Lean 必须检查 witness，不能信任 Rust 布尔值。大文件不能由 kernel reduction 通过 IO 读取；生成器把 canonical bytes 作为固定分块 literal 嵌入，或提供 Lean parser 检查的分块/hash certificate。Job、Profile、`proof_ir`、fabrication order、board 五类输入一视同仁。

首版实例证明优先使用 kernel-checkable reduction（例如 `decide_cbv`）。若规模导致不可接受的证明时间，后续可以加入可独立检查的 certificate；不得为了速度默认采用扩大可信基的 `native_decide`。固定 axiom whitelist 只有 `propext`、`Classical.choice`、`Quot.sound`；`sorryAx`、`Lean.trustCompiler`、任何 per-invocation native axiom 和自定义 axiom 均 hard-fail。正式门禁固定允许的 imports，并同时保存 `#print axioms` 输出与 `lean4checker --fresh` 结果。

证明必须绑定**实际交付文件**，而不只是 Rust 声称从文件解析出的对象。Lean 侧 pure parsers 对嵌入 bytes 计算五类模型；artifact binder 重新计算 exact byte hashes。只有 Rust projection 通过而任一 Lean parser/certificate 未实现时，最高状态是 `physical_valid`，不能称 `formal_valid`。

`proof_ir` 是 v1 的必要 compiler projection：版本化、canonical、byte-deterministic，并包含 pin role、NC、展开 instance/pad/net、part/footprint identity、geometry hash、placement provenance 与 obligation facts；现有 `.net`、BOM、`layout.json` 不能无损替代它。该 additive emitter 必须由独立子 RFC 接受后才能实现。Compiler soundness、expansion preservation 和 source → IR 证明属于未来独立 RFC；它们落地前不得提高本证书的措辞。

### 7. Initial manufacturing Profile

`{id: "jlcpcb-standard-2l", version: 1, sha256: "<64 lowercase hex>"}` 是经过人工审核、内容寻址的不可变**裸板** Profile，显示名为 `jlcpcb-standard-2l-v1`。它绑定普通 routed single board，不绑定 panelization 或 assembly。Profile schema 的 rule kind 闭集为 `minimum | maximum | exact | allowed_set | forbidden | conditional`；每条几何规则还必须声明 `from_shape`、`to_shape`、`measurement`、适用层与边界是否包含，不能用一个含糊的 “clearance” 数字代替。

下表是已核实的首版核心规则，不是完整 Profile 文件的替代品：

| 项目 | v1 规则 |
|---|---|
| 基材 / 层数 | FR-4 / 2 copper layers |
| 板厚 / finished copper | 1.6 mm / 1 oz |
| soldermask / finish | green / lead-free HASL |
| 最小 track width | 0.15 mm |
| 不同网络 track-track clearance | 0.15 mm |
| same-net track spacing | 0.25 mm |
| SMD pad-pad / pad-track clearance | 0.15 mm |
| via diameter / drill | 0.60 mm / 0.30 mm |
| via hole edge 到 track copper | minimum 0.20 mm |
| PTH annular ring | 0.25 mm |
| PTH-track clearance | 0.35 mm |
| routed edge/slot 到 copper | 0.30 mm |
| silkscreen 到 pad | 0.15 mm |
| silkscreen line / text height | 0.15 mm / 1.00 mm |
| green 1 oz soldermask bridge | 0.10 mm |
| soldermask opening 到相邻 trace | 0.09 mm |
| 最小 SMD pad | 0.25 × 0.25 mm |
| via hole-to-hole / pad hole-to-hole | 0.20 mm / 0.45 mm |
| 最小 NPTH | 0.50 mm |
| plated / non-plated slot width | 0.50 mm / 1.00 mm |
| 允许 via | through via only |
| 禁止工艺 | blind/buried、microvia、via-in-pad、plated edge、castellation、controlled-impedance claim |

完整 Profile 还必须显式覆盖且不得留空：board 最小/最大尺寸、board/outline tolerance、drill-to-drill、NPTH/PTH-to-copper、slot 长宽/圆角、soldermask/paste aperture、legend、所有 pad/via/edge 条件分支，以及 fabrication-order 中不能从 Gerber 推出的 exact choice。`fabrication-order.json` 因而固定 FR-4、2 层、1.6 mm、1 oz、green、lead-free HASL 和其他下单选项，并进入 artifact root。任何适用类别为 `unspecified` 都使 Profile completeness gate 失败；在完整 machine-readable Profile 与 fixtures 落地前，本 RFC 不能 Accepted。

来源包保存 JLCPCB 官方 rigid PCB capabilities 的人工审核快照、页面标题、抓取时间、内容 SHA-256 与逐规则摘录映射；URL 只作导航。公开网页未来变化不修改 v1，需要变化时发布 v2。规则是 exact choice、允许集、禁止项或带测量语义的上下界，不统称 “hard floor”。Agent 永远不能自动降级 Profile 或选择加价特殊工艺。

### 8. Physical and manufacturing checks

独立 physical verifier 的 v1 闭合检查集为：

- outline 闭合且不存在自交；所有要求在板内的 courtyard、pad、hole、track、via、zone 均位于允许区域；
- 同一装配面的不同实例 courtyard 不相交；top/bottom 的二维投影可重叠，THT 与 mechanical-only 对象按其真实三维穿透分类处理。v1 不允许 geometry/DFM waiver；
- locked instance 的 position/rotation/side 与 frozen CoHDL baseline 完全一致；
- 从 LogicalIR 推导每个 net 的 required physical terminal set；多-pad logical pin 的 required pads 全部连通，single-terminal net 无 route obligation，且没有额外 terminal 被吸入该分量；
- 不同网络只在共享 conductive layer 上比较实际 copper shape，相交或违反 shape-to-shape clearance 均失败；F.Cu/B.Cu 投影重叠本身不是短路，除非经 via/PTH 导通；
- track、via、drill、annular ring、slot、pad、edge 和 silkscreen 满足 Profile；
- 只验证 `zone_materialized` 后的新 candidate；zone hole、thermal 与孤岛按实际 polygon 处理，不能用于虚构连通；
- route layer 和 via span 在双层 Profile 中合法；没有 dangling segment 或无 net copper；
- 所有 populated electrical footprint 非 placeholder，identity/geometry hash、pad number/net 与 placement provenance 均与 `proof_ir` 一致；
- Gerber 反查 layer/outline/copper geometry，Excellon 反查 drill/slot geometry 与 plating 分类，IPC-D-356 反查 pad/testpoint/net identity；三者联合与被证明 board 核对。

KiCad DRC 是独立交叉检查，不取代自有 verifier：KiCad 检查其完整内部板模型和版本特定规则，自有 verifier 检查 RFC 固定的 Profile 与 proof predicates。两者都必须通过；意见不一致时失败并报告差异，不选择较宽松结果。

Obligation evidence 的 v1 闭合语义为：

- `formal`：连接、短路、声明的长度差、显式最小线宽等由 `ValidBoard` predicate 关闭；
- `review`：仅限 registry 列出的阻抗、compiler warning、模拟稳定性、EMI 与热义务，由指定 reviewer 签署。

Required obligation 既没有 formal proof 又没有对应人工 approval 时，最终状态是 `review_pending` 或 `failed`，绝不是 `ready`。Schematic review 还必须签署 obligation coverage 声明；该签名只表示人类确认“已声明集合覆盖本次自由文本意图”，不把自由文本自动提升为形式化命题。

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

同一 `(failure_signature, strategy_id)` 连续三次出现表示该策略没有进展；Harness 仅可切换到预先枚举的下一 strategy，新 strategy 也达到三次或不存在下一项即结束。Run 结束必须是 `ready`、`failed` 或 `review_pending` 之一；崩溃恢复后根据 immutable attempt 和 hashes 重放状态，不能从模型的聊天历史推断已经通过的 gate。

### 10. Final evidence and manufacturing package

所有 export 先写入 `attempts/<n>/export-staging/payload/`。`export_verified` 对 staging 中的实际 bytes 做语义 parse-back，随后生成 proof manifest；reviewer 审查并签署该不可变 payload root。只有 `artifact_bound` 与 `review_complete` 都成立，Harness 才把**完全相同的 bytes**原子 promotion 到 `final/`：

```text
final/
  job.json
  resolved-job.json
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
  approvals/schematic.json
  approvals/pre-order.json
  review-trust-roots.json
  revocations.snapshot.json
  sha256sums.txt
```

Hash DAG 必须无环：`proof-manifest.json` 记录全部 leaf payload hash，但排除自身、`approvals/` 与 `sha256sums.txt`；detached approvals 签署 manifest root；sums 覆盖 payload、manifest 和 approvals但排除自身。任何 review 都针对 staging 的最终 bytes。

Approval v1 使用 Ed25519，签名字节为 `"cohdl-agent-approval-v1\0" || manifest_sha256 || obligations_sha256 || role || revocation_snapshot_sha256`；digest 是 raw 32-byte，role 是 `schematic|pre_order` ASCII enum。包外 organization root public-key hash 是信任锚；bundle 只携带由该 root 签署的 role authorization chain。Revocation snapshot 也由 root 签署，含 authority、单调 sequence、as-of 与 previous hash。`verify` 必须从本地 policy 或 `--review-root-sha256` 获得预期 root，并可要求最小 sequence；绝不信任包内自声明 root。离线 verdict 只相对于绑定 snapshot 有效，后续撤销需联网另查。

`proof-manifest.json` 使用 canonical JSON，排序固定，不含 wall clock。时间、延迟、token 和费用进入非身份性的 `run-metadata.json`；原始 KiCad 日志也可旁存，但不进入 reproducible identity。Manifest 记录 schema、job/source/dependency/profile/board/tool hashes、每个 gate 的 canonical report hash、Lean theorem、axiom audit、所有 obligation 及证据状态。依赖 package bytes 随包保存，保证没有 registry cache 的第三方仍可离线重放。

首版 `ready` 只表示裸板制造包；BOM 不推出 PCBA-ready。制造流水线固定为 `raw export → allowlisted canonicalization → parse canonical bytes → semantic compare → leaf hash → manifest`。Canonicalizer 只处理 schema 列明的 wall-clock/path metadata；DRC date/path 与 Gerber/drill creation time 必须规范化。CI 跨时刻重复 export 比较 payload bytes。离线 `verify` 只读 final 内副本，不重新解释原始相对路径；任何后改 byte 使 manifest、approval 与 sums 失效。

### 11. CLI and machine interface

首版 CLI：

```text
cohdl-agent run JOB --review-root-sha256 HEX [--model ID] [--run-dir DIR] [--json]
cohdl-agent resume RUN_DIR --review-root-sha256 HEX [--json]
cohdl-agent verify RUN_DIR --review-root-sha256 HEX [--json]
cohdl-agent explain RUN_DIR [--attempt N]
```

- `run` 创建冻结输入并执行完整循环；默认 attempt cap 来自 Job。
- `resume` 只从已验证 hashes 与最后完整 attempt 继续。
- `verify` 不调用模型，只重放全部适用 gate；这是制造前和第三方复核入口。
- `explain` 输出 verdict ladder、失败 owner、证据路径和下一步，不改变状态。
- `--json` 只输出一个 versioned JSON document 到 stdout；日志进入 stderr。退出码：0=`ready`，1=`failed`，2=invocation/job error，3=`review_pending`。

模型 adapter 与 board backend 是内部 trait/protocol，不进入 Job 的正确性语义。更换 GPT、Claude、专用 placer 或 autorouter只能改变搜索效率和候选质量，不能改变 validator 结果。离线 `verify` 永远不访问模型或网络。

### 12. Diagnostics

Harness 使用与 compiler E/D codes 分离的稳定命名空间：

| Family | Meaning |
|---|---|
| H0xx | invocation、Job schema、路径和 hash |
| H1xx | agent/backend refusal、timeout、malformed tool call |
| H2xx | attempt/action budget、重复无进展、resume state |
| P1xx | placement、outline、courtyard、keepout |
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

**High.** 本 RFC 新增六个长期概念：`PCB Job Contract`、immutable `Baseline`、content-addressed `Candidate`、`Routed Board IR`、`Fabrication Profile`、以及由 `Proof Manifest + detached Approvals` 组成的 evidence root，并把 Agent Harness 从演示脚本提升为有正式 verdict 的 partner-layer 系统。成本真实且必要：没有这些边界，“完整 PCB”只能依赖 GUI 状态、模型自报和工厂上传后的偶然反馈，无法 grade。

概念边界刻意保持正交：

- Job 说明本次运行要什么和不能改什么；不复制 netlist。
- Baseline 冻结一次 source/compiler projection；Candidate 只表示一组 exact board bytes，两者都不可变。
- CoHDL Logical IR 说明器件、电气连接和显式意图；不承载铜线。
- Routed Board IR 是实际 KiCad 候选的验证投影；不成为第二种手写 PCB 语言。
- Profile 提供外部制造参数；不携带某块板的数据。
- Proof Manifest 绑定输入、候选、检查与输出；detached Approval 绑定 reviewer 与同一 root，不影响任何设计语义。

“proof”只表示命题在声明的模型与假设下被 Lean 接受，不与“功能已全面验证”或“工厂保证生产成功”混用。

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| High | Low | Crit | High | High | Low | Crit |

- **Concepts (High)：** 六个新概念均对应不可替代的责任边界；RFC 和教学材料必须始终使用同一组名称。
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
5. cross-net short、Profile clearance、width、via、edge、containment、overlap 与 keepout 错误均为 0。
6. 固定 KiCad CLI DRC 的 error、warning、excluded violation 均为 0。
7. Lean 接受绑定五类实际 bytes 的 `ValidBoardBytes`，axiom audit 与 `lean4checker --fresh` 通过。
8. Gerber/Excellon/IPC-D-356 与被证明候选语义核对，随后全部 hash 匹配。
9. required review 签署同一 evidence root；所有已声明 required obligation 有 disposition，schematic reviewer 已签 coverage 声明。

三个规范性端到端 scenario 必须全部通过：

| Scenario | Input/first result | Required transition | Terminal state |
|---|---|---|---|
| First-pass clean | fixture backend 首次给出完整合法候选 | 依次通过全部 rung，无 repair | `ready`，存在 final bundle |
| Repair succeeds | 首候选有一条 0.10 mm track，命中稳定 Profile code | 保留 parent digest，仅以允许 mutation 生成新 digest并全量重验 | 第二 attempt `ready`，动作链可重放 |
| Impossible constraints | locked connectors 与闭合 outline/courtyard 约束不可同时满足 | 诊断 owner 为不可由 agent 解锁的 placement contract；不得扩板或移动 lock | `failed`，不存在 final bundle |

扩展基准还必须包含以下 closed failure corpus：板框放不下、locked connector 被移动、一个未布 net、一个跨网短路、一条过细 track、一个过小 via、一个 copper-edge 违规、一个 placeholder footprint、一个 KiCad-only DRC 失败、一个 artifact 被篡改、一个 Lean proof/axiom 失败、一个 attempt-cap 失败。每个失败必须命中唯一稳定 code，并且最终目录不存在。

此外需要 metamorphic/property checks：平移未锁定的合法局部块不改变 logical net；输入顺序变化不改变规范化 verdict；同一候选验证两次时由 Harness 生成的 canonical report bytes 相同（外部工具的原始日志不进入 identity）；修改任一被绑定输入 byte 必使 proof manifest 校验失败；向 KiCad 文件加入不支持 construct 必须拒绝而不是忽略。

## AI-generatability

Agent 面对的是少量正交工具和局部诊断，不需要背诵 KiCad S-expression、Lean proof term 或完整厂商表。它选择坐标、层、宽度和路径；tool schema 检查单位、对象身份和下限；validator 返回精确冲突对象。这比“打开 GUI 看哪里红了”更适合模型，也比一次性要求生成完整 board bytes 更容易修复。

## Alternatives

- **只增强 system prompt。** 拒绝。Prompt 可以提高首稿质量，不能提供 completeness、artifact binding 或不可绕过的 verdict。
- **只运行 KiCad DRC。** 拒绝。它不替代 CoHDL 的单位/trait/pin/part 语义，也不固定本 RFC 的工厂 Profile、Job locks、proof assumptions 或 artifact provenance。
- **把全部 physical checks 加进 `src/drc.rs`。** 拒绝。它违反 RFC-004 的四规则边界，把逻辑网络 DRC 与物理制造验证混为一谈。
- **在 CoHDL 中新增完整 routing DSL。** 拒绝。永久 grammar/teaching cost 极高，也与既有“layout 是 partner concern”的架构边界冲突。受支持 KiCad subset + 验证投影足以关闭当前需求。
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

- 所有 toolchain 版本固定并进入 manifest：CoHDL release hash、KiCad **10.0.4**、Lean toolchain、formal definitions/import allowlist、Profile hash、backend adapter version。
- `verify` 完全离线；generation 网络访问与 verification 分离。缺少本地依赖或工具是 hard failure。
- Prompt、模型回复、tool calls、diagnostics 和 candidate hashes 进入 append-only `actions.jsonl`；secret、token 和环境变量值必须在写盘前结构化删除。
- KiCad 在空隔离目录运行，同 stem 放置 `candidate.kicad_pcb/.kicad_pro/.kicad_dru`；拒绝其他邻接规则文件，固定 env/locale/timezone。Manifest 绑定三者与 executable/adapter hashes；canonical report 记录 staged rules hashes，disabled/ignored tests 为零。集成测试含一个仅固定 `.dru` 才触发的 canary，防止退回默认规则仍通过。
- JLCPCB Profile 来源固定为官方 rigid PCB capability evidence snapshot；抓取只进入 Profile 更新流程，普通 `run/verify` 不联网。
- Lean formal 工程使用 pinned `lean-toolchain`，首版优先仅依赖 Lean/Std；新增 mathlib 或 FFI 需要独立 trust-impact review。
- CI 分为 compiler、harness、formal、KiCad integration 四个 job。Compiler release 不因开发环境缺少 KiCad/Lean而增加运行依赖；harness release 必须通过全部四类门禁。

规范性外部参考：

- KiCad CLI PCB DRC：<https://docs.kicad.org/master/en/cli/cli.html>
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

Agent 不需要学习 Lean tactic、KiCad 文件语法或厂商网页。硬件 reviewer 需要能读 `resolved-job.json`、Profile 摘要、verdict ladder 和 obligation；不要求阅读 generated proof term。教学材料必须始终使用 `logical_valid`、`fab_profile_valid`、`formal_valid`、`ready` 四个不同术语，禁止统称为 “valid”。

## Failure modes

- **模型输出看似完整但漏布一条网。** `EveryRequiredNetConnected` 与 KiCad ratsnest/DRC 双重拒绝。
- **Zone 声明存在但未 refill。** 固定 KiCad 在副本上 refill/save，保存 bytes 得到新 digest；其后所有 gate 只验证新候选。
- **遇到不支持的 KiCad construct。** hard-fail `unsupported_construct`，不忽略未知节点。
- **模型连续局部修复振荡。** 归一化 failure signature 三次触发策略切换/停止。
- **外部工具崩溃或版本不符。** `external_tool` failure；不得沿用旧报告。
- **自然语言要求没有结构化表达。** Harness 不自行猜测其含义；schematic reviewer 若不能签署 obligation coverage，则 `review_pending` 或失败。
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

现有两个 example board 和 OpenMicroKBD revisions 可作为 importer/emitter fidelity fixtures，但首个端到端 qualification board 应是一个专门限制在 `mcu_iot_control_v1` 的小型 sensor node。它必须真实完成 placement/routing、通过全部自动 gate，并由工程师在 KiCad 与工厂上传预览中完成 checkpoint。历史手工 routed board 只作差分参考，不自动成为 ground truth。

## Decision

**Pending review.** Header lifecycle status is `Proposed`; the RFC-process final decision has not yet been made.

本 RFC 记录当前讨论形成的推荐架构：单一 `jlcpcb-standard-2l-v1` 裸板 Profile；首版常规 MCU/IoT 双层裸板；agent 是不可信搜索器；最终 PCB 由 pinned CoHDL projection、physical verifier、KiCad DRC、Profile DFM、Lean proof、semantic export verification、artifact binding 和显式 human review 共同 gate；首版证明候选相对于 TCB 与已声明假设有效，不证明 compiler、agent、placer 或 router 算法。

它不能自行标记为 Accepted。接受前必须完成：

- 在 conol.ai live source-of-truth 中预留编号、审查并同步本提案；本地 `RFC-032` 只是候选编号；
- 明确确认 `cohdl-agent` 是 Constitution 允许的 partner-layer tool；若要成为 compiler 内置 P&R，则先通过 Goal Change Proposal；
- 对六个新核心概念完成 coherence regression；
- 决定子 RFC/Task Contract 的编号与所有权；
- 完成四个阻塞 spike：canonical `proof_ir`、KiCad net ordinal + zone normalization、formal geometry 子集、staging/export/hash root；
- 用一个现有 `.kicad_pcb` 验证 Lean 五类 bytes parser/certificate 的性能与 axiom policy；
- 补齐 machine-readable Profile 的全部适用规则、`fabrication-order.json`、官方 evidence snapshot 与成功/失败 fixtures。

在这些门禁完成前，本文件只是一份可审查的 Proposed draft，不改变语言、compiler verdict、发布承诺或制造责任。
