#!/usr/bin/env python3
"""Reproduce the bundled, renamed CJK fallback from the upstream variable font."""
from pathlib import Path
import sys
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools import subset

font = TTFont(sys.argv[1])
instantiateVariableFont(font, {'wght':400}, inplace=True)
options = subset.Options()
options.name_IDs = ['*']; options.name_legacy = True; options.name_languages = ['*']
worker = subset.Subsetter(options=options)
worker.populate(unicodes=list(range(0x3000,0xa000))+list(range(0xff00,0xfff0)))
worker.subset(font)
for record in font['name'].names:
    if record.nameID in (1,3,4,6,16,17):
        value = 'InstallWizardCJKFallback' if record.nameID == 6 else 'InstallWizard CJK Fallback'
        record.string = value.encode(record.getEncoding())
output = Path(__file__).resolve().parents[1] / 'assets/fonts/InstallWizard-CJK.ttf'
font.save(output)
print(output)
