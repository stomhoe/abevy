id: desert4
tags: [portal, land, desert, river_candidate]
size: (3, 3)
debug: [tree_feature, cave_portal_feature]

let shared_pd = pd299
//todo poner noises para las rocas

let tree_feature = idxmax(hp82, 1.2)
let tree_feature = *(tree_feature, pd32)
let tree_feature = *(tree_feature, shared_pd, )
let tree_feature = +(tree_feature, -0.0)

let cave_portal_feature = idxmax(hp82, 0.007)
let cave_portal_feature = *(inlandness, cave_portal_feature)

out = idxmax(0.5, tree_feature, cave_portal_feature)

bif "" -> []
bif "" -> [sand]
bif "" -> [portal_cave, ]
