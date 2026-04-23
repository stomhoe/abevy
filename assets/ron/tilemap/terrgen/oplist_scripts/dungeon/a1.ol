id: a1
tags: [dungeon]
size: (1, 1)

// Calculate max continentness from base continent and peninsula noises
let continentness = max(fnl.conti, fnl.penin)

// Multiply by inherited variable from root
let continentness = *(continentness, a_diff)

// Determine dungeon floor vs wall based on threshold
out = idxmax(0.3, continentness)

[dublack] ""
[dublack] ""
