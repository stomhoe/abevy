id: normal2
debug: [inlandness, continentness, normal_diff]
tags: [inlandness, ]

let inlandness = remap(continentness, 0.40007, 1.007, 0.0, 1.0)

out = idxmax(inlandness, 0.03)

bif normal3 -> []
bif beach_gravel -> [] biomes: [beach=1.0(1.0,0.)]
