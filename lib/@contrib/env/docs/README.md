# Env sensor source manifest

Retrieved and audited on 2026-08-03.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `bst-bme280-ds001.pdf` | `BME280` | Bosch Sensortec BST-BME280-DS001-23 rev 1.23, January 2022 | <https://www.bosch-sensortec.com/products/environmental-sensors/humidity-sensors-bme280/> | `5be31e7713077646b11a4a35b3930cb6629f15402ae672bf4e1be9e2f8b6c7d9` |

## Retrieval and lifecycle notes

- 实际下载来源（例外记录）：LilyGO 官方仓库镜像
  <https://raw.githubusercontent.com/Xinyuan-LilyGO/T-Display-SF32/master/doc/BME280.pdf>
  （T-Display-SF32 整机所用文档）。文档页眉自标识 Bosch Sensortec。
- 结构验证：pymupdf 打开 60 页、无加密、文本可抽取。
- 引脚表 p38（Table 35）文本可引；封装外形 2.5x2.5x0.93 mm（p2 特性列表）。
- 焊盘几何：Figure 21（p43）为内嵌位图无文本层。尺寸值经视觉读取 +
  原生位图像素测量互证（378 px/mm 自 0.65mm 节距标定，复现
  0.507/0.358/2.049），视觉挂线解读有歧义处以像素链式闭合为准。
  编号方向为顶视图顺时针（p38 显式说明，非典型方向），pin 1 顶视图右上。
