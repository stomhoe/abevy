id: arctic3
tags: [portal, land, arctic]
size: (1, 1)
debug: [cave_portal_feature]

let lakeness = max(fnl.lakebig)
let lakeness = *(lakeness, continentness, 1.4)
let lakeness = *opo(lakeness, gravelness)

out = idxmax(continent_threshold, lakeness, )

bif arctic4 -> [snow]
bif "" -> [ice] biomes: [ice=1.5(1.0,0.35)]
bif "" -> [orange]
