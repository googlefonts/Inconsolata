#!/bin/sh
set -e

if [ ! -f prod.glyphs ]
then
	printf "%s" \
	      "No production ready source! \
Please run gen_instances.py and inco_fix.py inside Glyphsapp \
with Inconsolata-vf.glyphs and save this file as prod.py. \
DO NOT OVERWRITE Inconsolata-vf.glyphs!"
	exit 1
fi

echo "Converting .glyphs to .ufo"
fontmake -g prod.glyphs -o ufo

echo "Generating Static fonts"
mkdir -p ../fonts ../fonts/ttf ../fonts/otf ../fonts/variable
fontmake -m master_ufo/Inconsolata.designspace -i -o ttf --output-dir ../fonts/ttf/
fontmake -m master_ufo/Inconsolata.designspace -i -o otf --output-dir ../fonts/otf/


echo "Generating VFs"
fontmake -m master_ufo/Inconsolata.designspace -o variable --output-path ../fonts/variable/Inconsolata[wdth,wght].ttf
python gen_stat.py
statmake --stylespace ./stat.stylespace --designspace master_ufo/Inconsolata.designspace ../fonts/variable/Inconsolata\[wdth\,wght\].ttf

rm -rf master_ufo/ instance_ufo/

echo "Post processing"
ttfs=$(ls ../fonts/ttf/*.ttf)
for ttf in $ttfs
do
	gftools fix-dsig -f $ttf;
	ttfautohint $ttf $ttf.fix
	[ -f $ttf.fix ] && mv $ttf.fix $ttf
	gftools fix-hinting $ttf $ttf.fix;
	[ -f $ttf.fix ] && mv $ttf.fix $ttf
done

echo "Post processing VFs"
vf=../fonts/variable/Inconsolata[wdth,wght].ttf
gftools fix-dsig -f $vf;
# Issue with DirectWrite. Causes
# https://github.com/google/fonts/issues/2085
gftools fix-unwanted-tables --tables MVAR $vf

echo "Transferring and compiling VTT hints"
python -m vttLib mergefile vtt_hinting.ttx $vf
python -m vttLib compile $vf $vf.fix --ship
[ -f $vf.fix ] && mv $vf.fix $vf
gftools fix-hinting $vf
[ -f $vf.fix ] && mv $vf.fix $vf

rm ../fonts/ttf/*gasp*.ttf ../fonts/variable/*gasp*.ttf prod.glyphs
