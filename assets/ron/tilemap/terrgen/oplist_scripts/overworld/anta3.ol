id: anta3
tags: [portal, land, anta]
size: (1, 1)
debug: [cave_portal_feature]

let lakeness = max(fnl.lakebig)
let lakeness = *(lakeness, inlandness, 0.2)

out = idxmax(continent_threshold, lakeness, )

bif anta4 -> [snow]
bif "" -> [ice] biomes: [ice=1.5(15,0.35)]
bif "" -> [orange]
