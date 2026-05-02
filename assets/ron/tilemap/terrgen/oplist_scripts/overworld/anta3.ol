id: anta3
tags: [portal, land, anta]
size: (1, 1)
debug: [cave_portal_feature]

let lakeness = avg(fnl.lakebig)
let lakeness = *(lakeness, inlandness, 0.2)

out = idxmax(continent_min, lakeness, )

[snow] anta4
[ice] "" biomes: [ice=1.5(15,0.35)]
