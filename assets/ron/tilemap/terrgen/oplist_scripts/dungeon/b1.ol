id: b1
tags: [portal]
size: (1, 1)

// Calculate max continentness from base continent and peninsula noises (with seed 4)
let continentness = max(fnl.conti.s4, fnl.penin.s4)

// Multiply by inherited variable from root
let continentness = *(continentness, b_diff)

// Determine dungeon floor vs wall based on threshold
out = idxmax(0.3, continentness)

[dublack] ""
[dublack] ""
