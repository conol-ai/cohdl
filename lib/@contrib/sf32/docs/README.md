# SF32 source manifest

Retrieved and audited on 2026-08-03.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `ds0052-sf32lb52x-cn.pdf` | `SF32LB52BU36` / `SF32LB52BU56` / `SF32LB52DUB6` / `SF32LB52EUB6` | SiFli DS0052-SF32LB52X-CN V0.1, 2024 | <http://www.sifli.com>（页脚自标识） | `68ad87f5846c47908614e980eaa56b0eea5e4ae036d0a5cee9357f80db7357c2` |

## Retrieval and lifecycle notes

- 实际下载来源（例外记录）：LilyGO 官方仓库镜像
  <https://raw.githubusercontent.com/Xinyuan-LilyGO/T-Display-SF32/master/doc/DS0052-SF32LB52X-芯片技术规格书%20V0p1.pdf>
  （T-Display-SF32 整机所用文档）。结构验证：pymupdf 61 页、无加密、文本可抽取。
- 交叉文档：UM5201 用户手册（同仓库）文本证实 VDD_VOUT1/2 为内部 LDO
  输出（"由芯片内部的LDO产生，是HPSYS/LPSYS的主要供电"）。

## 数据出处与已知缺口（V0.1 早期手册）

- 引脚：45×PA GPIO 来自表5-2（文本）；专用管脚来自表5-3（文本）。
- **pin 15/16（VDD_VOUT2/VDD_VOUT1）在 V0.1 两张表中均缺失**，
  来自图5-1 管脚分布图（视觉读取，全部交叉锚点 PA32-22/VDD_RTC/
  VDD_RET/AVDD33/BUCK_FB 复核吻合），并经 UM5201 文本佐证。
- 封装：图5-2 为位图；尺寸经视觉读取 + 链式闭合校验
  （(7-5.49)/2 = 0.755 = L 0.4 + K 0.355 精确成立）。
- 焊盘几何：手册无官方 land pattern；按 lib/qfn 既有 7mm 体先例派生
  （0.2 宽覆盖 lead b max、0.875 长、±3.4375），方法已在 qfn 注释披露。
- **板上变体未确认**：wiki 称 16MB Flash + 8MB PSRAM，但 V0.1 四个变体
  最大 32Mb(=4MB) pSRAM 且无 16MB NOR——存在出入，四个变体全部建模，
  由设计选用。此差异已记录于 cohdl-doc reports/schematic-crossref.md。
