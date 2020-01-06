#!/bin/sh
set -e

if [ ! -f prod.glyphs ]
then
	printf "%s" \
	      "No production ready source! \
Please run gen_instances.py and inco_fix.py inside Glyphsapp \
with Inconsolata-vf.glyphs and save this file as prod.py"
	exit 1
fi

echo "Generating Static fonts"
mkdir -p ../fonts ../fonts/ttf ../fonts/otf ../fonts/variable
fontmake -g prod.glyphs -i -o ttf --output-dir ../fonts/ttf/
rm -rf master_ufo/ instance_ufo/
fontmake -g prod.glyphs -i -o otf --output-dir ../fonts/otf/


# echo "Generating VFs"
fontmake -g prod.glyphs -o variable --output-path ../fonts/variable/Inconsolata[wdth,wght].ttf
# 
rm -rf master_ufo/ instance_ufo/


echo "Post processing"
ttfs=$(ls ../fonts/ttf/*.ttf)
for ttf in $ttfs
do
	gftools fix-dsig -f $ttf;
	gftools fix-nonhinting $ttf $ttf.fix;
	[ -f $ttf.fix ] && mv $ttf.fix $ttf
done


echo "Post processing VFs"
vfs=$(ls ../fonts/variable/*.ttf)
gftools fix-dsig -f $vfs;
 
echo "Fixing VF Meta"
for vf in $vfs
do
	ttx -f -x "MVAR" $vf; # Drop MVAR. Table has issue in DW
	rtrip=$(basename -s .ttf $vf)
	new_file=../fonts/variable/$rtrip.ttx;
	rm $vf;
	ttx $new_file
	rm $new_file
done

rm prod.glyphs
