id: elf4
size: (1, 1)
tags: [river_candidate]

let tree_feature = idxmax(hp81, 0.14)
let tree_feature = avg(fnl.forest.s3, tree_feature)
let tree_feature = *(tree_feature, pd499)

let bush_feature = idxmax(hp82, 0.05)
let bush_feature = *(bush_feature, pd12)
let bush_feature = +(bush_feature, -0.0)

let cave_portal_feature = idxmax(hp82, 0.005)
let cave_portal_feature = *(inlandness, cave_portal_feature)

out = idxmax(0.5, bush_feature, tree_feature, cave_portal_feature)

bif "" -> []
bif "" -> [elfveg_sampler]
bif "" -> [elftree_sampler]
bif "" -> [portal_cave, ]
