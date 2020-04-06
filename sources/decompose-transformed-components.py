#MenuTitle: Decompose Transformed Components
"""TTFautohint doesn't like components which have been flipped"""

def find_transformed_component_glyphs(font):
	found = set()
	for idx, master in enumerate(font.masters):
		for glyph in font.glyphs:
			components = glyph.layers[idx].components
			for comp in components:
				if sum(comp.scale) != 2.0:
					found.add(glyph.name)
				if comp.rotation != 0.0:
					found.add(glyph.name)
	return found


def main():
	font = Glyphs.font
	bad_components = find_transformed_component_glyphs(font)
	if not bad_components:
		print "Skipping. No transformed components"
		return
	
	for idx, master in enumerate(font.masters):
		for name in bad_components:
			print "Decomposing transformed %s in %s" % (
				name, master.name
			)
			font.glyphs[name].layers[idx].decomposeComponents()

		
if __name__ == "__main__":
	main()

