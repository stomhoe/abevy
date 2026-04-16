id: elf2
debug: [ inlandness, continentness, elf_diff]
tags: []

let inlandness = remap(continentness, 0.40007, 1.007, 0.0, 1.0)

out = idxmax(inlandness, 0.03)

bif elf3 -> []
bif beach_gravel -> [] biomes: [beach=1.0(1.0,0.)]
