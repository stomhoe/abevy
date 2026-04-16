id: beach_gravel
size: (1, 1)
tags: [beach, ]


let beach_avg_noise = avg(fnl.beachsmol, fnl.beachbig)

let beachness = beach_avg_noise
let gravelness = COMPL beach_avg_noise,

out = idxmax(0.5, beach_avg_noise)

bif "" -> [sand] 
bif "" -> [gravel]
