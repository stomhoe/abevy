id: normal2
debug: [inlandness, continentness, normal_diff]
tags: [inlandness, ]

let inlandness = remap(continentness, shared_continentness_min, shared_continentness_max, 0.0, 1.0)

out = idxmax(inlandness, 0.05)

[] normal3
[] beach_gravel biomes: [beach=1.0(1.0,0.)]
