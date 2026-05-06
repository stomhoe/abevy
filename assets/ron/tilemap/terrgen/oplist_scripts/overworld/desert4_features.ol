id: desert4
tags: [portal, land, desert, river_candidate]
size: (3, 3)
debug: [tree_feature, cave_portal_feature]

let shared_pd = pd399
//todo poner noises para las rocas

let lucky_tree = idxmax(hp81, 0.02)
let tree_feature = max(fnl.forest_lf, lucky_tree)

let tree_feature = lerp(tree_feature, inlandness, 0.1)
let tree_feature = *(tree_feature, shared_pd, 0.8)

let cave_portal_feature = idxmax(hp82, 0.007)
let cave_portal_feature = *(inlandness, cave_portal_feature)

out = idxmax(0.5, tree_feature, cave_portal_feature)

[] ""
[desert_tree_sampler, ] ""
[portal_cave, ] ""
