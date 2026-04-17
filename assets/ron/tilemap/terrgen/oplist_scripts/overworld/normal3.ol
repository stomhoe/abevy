id: normal3
size: (1, 1)
debug: [continentness]




let lakeness = max(fnl.lakesmol, fnl.lakebig)
let lakeness = *(lakeness, inlandness, 0.3)

out = idxmax(continent_threshold, lakeness, )

bif normal4 -> [tempgrass] biomes: [temp=1.(16, 2.)]
bif "" -> [lake]
bif "" -> [orange]
