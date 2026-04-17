id: desert2
size: (1, 1)
debug: [inlandness, continentness]
tags: [desert, ]

let inlandness = remap(continentness, shared_continentness_min, shared_continentness_max, 0.0, 1.0)

out = idxmax(inlandness, 0.05)

bif desert3 -> []
bif "" -> [sand] biomes: [desert=1.0(1.0,0.)]
