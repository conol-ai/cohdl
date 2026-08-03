# Charger source manifest

Retrieved and audited on 2026-08-03.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sgm41562a-sgm41562b.pdf` | `SGM41562B`（订货号 SGM41562BXG-TR） | SG Micro SGM41562A/SGM41562B datasheet, REV. A.1, January 2022 | <https://www.sg-micro.com>（官网，页脚自标识） | `e3fcb1f1662400261dfe60b6376fcc269c49ca0839f4d64a0cebb596835008fe` |

## Retrieval and lifecycle notes

- 实际下载来源（例外记录）：LCSC C5153801 文档的 LilyGO 官方镜像
  <https://raw.githubusercontent.com/Xinyuan-LilyGO/T-Display-SF32/master/doc/C5153801_电池管理_SGM41562BXG-TR_规格书_SGMICRO(圣邦微)电池管理规格书.PDF>
  选择该镜像的原因：它就是 T-Display-SF32 整机所用文档，且 LCSC 页面反爬。
  文档页脚自标识 SG Micro Corp / www.sg-micro.com，内容与官网版本一致性
  未做逐字比对（官网未提供免登录直链）。
- 文件名标注"规格书"（中文），**实际内容为英文**（REV. A.1, JANUARY 2022）。
- 结构验证：pymupdf 打开 34 页、无加密、文本可抽取（`file` 报 56 页系
  线性化头陈旧值，以 pymupdf 为准）。
- 首次按 wiki 链接下载返回 HTML（404 伪装），经 GitHub API 取得正确
  download_url——"拒绝 HTML 伪装 PDF"纪律再次受力。

## Coverage and geometry

- 该文档覆盖 SGM41562A 与 SGM41562B 两型号；本库当前只建 B（板上用料）。
  A/B 差异与 alt 兼容性在转录时评估（默认充电参数类差异，引脚一致才可
  alt）。
- p2 Ordering Information（订货号/封装对应）。
- p3 Pin Configuration/Description：WLCSP-1.52×1.52-9B，9 球 3×3
  （A1..C3 字母数字命名）。
- p32 Package Outline / Land Pattern（机械与焊盘几何来源）。
