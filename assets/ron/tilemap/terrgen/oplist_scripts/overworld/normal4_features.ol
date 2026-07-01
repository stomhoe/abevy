id: normal4
tags: [portal, land, land_temp, river_candidate]
size: (3, 3)
debug: [cave_portal_feature, tree_feature, shared_pd]

let shared_pd = pd299

let bush_feature = idxmax(hp82, 0.10)
let bush_feature = *(bush_feature, pd12)
let bush_feature = +(bush_feature, -0.0)

let lucky_tree = idxmax(hp81, 0.04)
let tree_feature = max(fnl.forest_lf, lucky_tree)

let tree_feature = lerp(tree_feature, inlandness, 0.2)
let tree_feature = *(tree_feature, shared_pd, 0.9)


let lucky_light = idxmax(hp83, 0.005)

let cave_portal_feature = idxmax(hp82, 0.005)
let cave_portal_feature = lerp(cave_portal_feature, inlandness, 0.3)

out = idxmax(0.5, bush_feature, tree_feature, cave_portal_feature, )

[] normal5
[temp_bush_sampler, ] ""
[temp_tree_sampler, ] ""
[portal_cave, ] ""
//[lamp, ] ""
