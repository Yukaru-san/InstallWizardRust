# CJK fallback font

This renamed font is derived from Noto Sans JP, maintained by the Noto project.
It is a bundled fallback for Japanese kana, CJK ideographs and full-width forms.
The regular 400-weight instance is subset to U+3000–U+9FFF and U+FF00–U+FFEF.
The original proportional UI fonts remain first in the fallback chain.

Upstream source:
https://github.com/notofonts/noto-cjk/blob/main/Sans/Variable/TTF/Subset/NotoSansJP-VF.ttf

Original file SHA-256: `f4b373b226668ee33a6e54b02823dcd2d1209f17159f777421ae8c2275160369`.

The subset is renamed InstallWizard CJK Fallback. Reproduce it with FontTools
using `python3 tools/subset-font.py <original-file>`. The original copyright and
SIL Open Font License are in OFL.txt and in the embedded dependency notices.
