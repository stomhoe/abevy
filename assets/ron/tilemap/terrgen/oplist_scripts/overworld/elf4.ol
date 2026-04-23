id: elf4
size: (1, 1)
tags: [river_candidate, ]
debug: [tree_pd]

let bush_feature = idxmax(hp82, 0.04)
let bush_feature = *(bush_feature, pd12)
let bush_feature = +(bush_feature, -0.0)
let tree_pd = pd399

let lucky_tree = idxmax(hp81, 0.02)
let tree_feature = max(fnl.forest, lucky_tree)

let tree_feature = lerp(tree_feature, inlandness, 0.3)
let tree_feature = *(tree_feature, tree_pd, 1.2)


let cave_portal_feature = idxmax(hp82, 0.005)
let cave_portal_feature = *(inlandness, cave_portal_feature)

out = idxmax(0.5, bush_feature, tree_feature, cave_portal_feature)

[] ""
[elfveg_sampler] ""
[elftree_sampler] ""
[portal_cave, ] ""
