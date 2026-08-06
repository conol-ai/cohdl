# Skill: explorer-view-gen

# CoHDL Explorer 视图配置生成 — AI 为看板生成/维护分区展示配置

Use when: 为 CoHDL design 生成/更新 schematic explorer 的 view.json
(分区/多页展示配置);用户说"给这个板子分区"、"生成看板视图"、
"整理电源分区"、partition/region/view config for explorer。

## 定位红线

view.json **只管展示,不碰电气**。源码(.cohdl)是唯一电气真相;配置错了
最多显示难看,不可能弄坏电路。因此本 skill 允许大胆自动生成、人工随意
微调,无需任何电气验证门禁。

## 输入

1. design 源码(`src/*.cohdl`)——语义主来源
2. `cohdl-explorer <project> -o model.json` 的 ExplorerModel——机读事实:
   - `instances[].designator/device_fq/impl_traits/part.mpn`
   - `derived.fn_groups`(fn 子电路展开分组——天然区域种子)
   - `derived.rails`(电源轨列表)与 `derived.two_terminal`
   - net `voltage`/`is_gnd` 注解

## 输出契约(schema v1)

`views/<Design>.view.json`(部署到 web/public/views/ 或 serve dist):

```json
{
  "schema_version": 1,
  "design": "<design 名,必须与模型 design 字段一致(文件名同名)>",
  "views": [
    { "name": "<页名>", "regions": [
      { "name": "<区域名>", "members": ["U2", "path:Top::mcu", "agg:GND + V3V3"] }
    ]}
  ]
}
```

成员规则(按优先序匹配,首个命中区域获得该节点):
- `"U2"` — designator(首选,稳定可读)
- `"path:<instance path>"` — 精确实例路径
- `"agg:host:<IC path>"` — 某 IC 的去耦聚合节点(带 #[bypass] 标注的
  电容自动按宿主分组,如 `agg:host:Pico2::mcu`;UI 会画虚线连到宿主,
  分区时把它和宿主放同一区)
- `"agg:<rail组合key>"` — 无宿主的 rail 组合聚合(sorted rails 以
  " + " 连接,如 `agg:GND + V3V3`)
- 未匹配节点自动落入 "Other" 区,不会丢

**语言规范:view 名/区域名一律英文**(UI 默认英文;Blocks/Power/
Host-USB/MCU Core/USB Input 这类短名)。

注意:**所有带非 rail 端的阻容感都是独立迷你节点**(串联件坐在线路
中间、对轨件挂一根线),必须按 designator 写进 members(串阻跟它服务
的功能链走:USB 串阻归 USB 区、SWD 串阻归 Debug 区、晶振串阻归时钟
区);只有两端全 rail 的去耦/上拉才用 `agg:` 规则整组分派。

## 分区惯例(生成时遵循)

1. **页(views)**:第一页永远是"分区总览"(全部器件分入功能区);
   随后按需出专题页:Power / Host·USB / 传感器 / 射频 / 调试。
   同一器件可出现在多页(原理图惯例)。
2. **区域(regions)命名**:功能导向中文短名("3V3 电源"、"USB 输入"、
   "主控核心"),不用器件名当区名。
3. **归组信号**(按证据强度排序):
   - `fn_groups`:同一 fn 展开的实例几乎总在同区(去耦组归宿主 IC 区)
   - 电源链:LDO/DCDC(device 名/trait)+ 其电感电容 + 对应 rail 的
     `agg:` 节点 → 电源区
   - 接口链:连接器 + 其 ESD/CC 电阻 → 接口区
   - MCU/SoC + 晶振 + flash + boot strap → 主控区
   - `#[intent]`/`placement_hint` 文本是分区意图的直接证据
4. **拆细优先**:每区 2-4 个节点为宜(硬上限 6,单件成区也允许——
   如 "User LED"/"GPIO Edge");超了必须按功能链再拆——例:"Power"
   拆成 "VBUS Input"/"VSYS Monitor"/"3V3 Buck"/"1V1 Core";"接口"拆成
   "USB Connector"/"Debug Header"/"GPIO Edge"。Blocks 页分区数以
   5-10 个为宜。UI 支持区域级二级 tab(Combined/单区切换),细分区
   直接成为可聚焦单元,拆细没有可读性代价。
   归组必须以 **net 证据**为准,不凭元件类型直觉:同为电阻,
   SWDIO 串阻属 Debug 链、VSYS_DIV 分压属电源监测,放错区即返工。
5. 去耦聚合(`agg:host:X`)永远和宿主 IC 同区;晶振及其负载电容与
   MCU 同区;ESD 与它保护的连接器同区。
6. 第一页 "Blocks" 必须全覆盖(Other 区节点占比 >20% = 分区不合格,
   回炉重分)。

## 生成后自检(必做)

1. `python3 -c "import json;json.load(open('<file>'))"` — 语法
2. 文件名 = `<design>.view.json` 且内 `design` 字段一致
3. 每个 designator 在模型 instances 里存在(用 model.json 对照);
   agg key 与前端聚合 key 生成规则一致(sorted + " + ")
4. 部署后浏览器/截图确认页签出现、无大面积"其他"区(>30% 节点落
   "其他" = 分区质量不合格,回炉)

## 已验证样例

`explorer/views/Pico2.view.json`(rpi-pico2 金标:分区总览/Power/
Host·USB 三页)。
