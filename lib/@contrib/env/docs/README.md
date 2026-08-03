# Env sensor source manifest

Retrieved and audited on 2026-08-03.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `bst-bme280-ds001.pdf` | `BME280` | Bosch Sensortec BST-BME280-DS001-24 rev 1.24, February 2024 | <https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bme280-ds002.pdf> | `a2ccdb449fec94380742fe8eec851a11d9bd4142252d332b34682b4deecd7d89` |

## Retrieval and lifecycle notes

- 2026-08-03 二轮：从 Bosch 官方链接重新获取，**rev 1.24 (2024-02)**，
  60 页、无加密、文本可抽取。此前曾用 LilyGO 仓库镜像（rev 1.23,
  2022-01）——替换前逐页比对了转录引用的 p2/p38/p39/p43，正文与
  rev 1.23 完全一致（仅页眉文档号与空白差异），转录结论不受影响。
- 引脚表 p38（Table 35）文本可引；封装外形 2.5x2.5x0.93 mm（p2 特性列表）。
- 焊盘几何：Figure 21（p43）为内嵌位图无文本层。尺寸值经视觉读取 +
  原生位图像素测量互证（378 px/mm 自 0.65mm 节距标定，复现
  0.507/0.358/2.049），视觉挂线解读有歧义处以像素链式闭合为准。
  编号方向为顶视图顺时针（p38 显式说明，非典型方向），pin 1 顶视图右上。
