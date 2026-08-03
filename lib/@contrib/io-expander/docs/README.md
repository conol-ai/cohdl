# IO expander source manifest

Retrieved and audited on 2026-08-03.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `xl9535-xl9555.pdf` | `XL9555QF24` / `XL9555`（TSSOP24） | Xinluda XL9535/XL9555 Product datasheet Rev 2.4 | <http://www.xinluda.com>（页眉自标识） | `8d6805fbf1e452d674bc0d3868f5c047065e82317a9f6fc6d5bfd22fb6e58d78`|

## Retrieval and lifecycle notes

- 获取来源：公开分发镜像；结构验证：pymupdf 25 页、无加密、文本可抽取。
- 引脚表 Table 2（p3）文本可引——**SOP/SSOP/TSSOP 与 QFN 引脚号不同**，
  device 采用 variants 建模；XL9535/XL9555 差异为 XL9555 每个 I/O 多一个
  弱上拉（Table 2 note 1），当前只建 XL9555。
- 四个封装外形全部为文本表格（§13.1-13.4, p22-25）；本次交付
  QFN24（lib/qfn::QFN24N50P400X400_1EP27X27）与 TSSOP24
  （SOP24P65X640X120N）两个 part；SOP24/SSOP24 变体已在 device 就位，
  part 按需后补。
- 焊盘几何为披露式派生（手册无官方 land pattern）：QFN 按 lib/qfn 既有
  模式，TSSOP 按鸥翼脚通用派生（宽 0.35=lead max+0.05、长 1.1、
  0.2 跟/0.4 尖），方法见源码注释。
