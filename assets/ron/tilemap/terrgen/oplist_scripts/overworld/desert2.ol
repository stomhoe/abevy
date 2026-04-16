id: desert2
size: (1, 1)
debug: [beach_avg_noise, shore_proximity, beachness, inlandness, continentness]
tags: [desert, ]

let inlandness = remap(continentness, 0.4, 1.007, 0.0, 1.0)

out = idxmax(inlandness, 0.05)

bif desert3 -> []
bif "" -> [sand] biomes: [desert=1.0(1.0,0.)]
