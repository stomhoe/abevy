
#define_import_path bevy_shader_utils::voronoise

fn hash_three( p: vec2<f32> ) -> vec3<f32>
{
    let q: vec3<f32> = vec3( dot(p,vec2(127.1,311.7)), 
                   dot(p,vec2(269.5,183.3)), 
                   dot(p,vec2(419.2,371.9)) );
    return fract(sin(q)*43758.5453);
}

fn voronoise( p: vec2<f32>, u: f32, v: f32 ) -> f32
{
    let k: f32 = 1.0+63.0*pow(1.0-v,6.0);

    let i: vec2<f32>= floor(p);
    let f: vec2<f32>= fract(p);
    
    var a: vec2<f32>= vec2(0.0,0.0);
    for (var y = -2; y<=2; y++) {
    for (var x = -2; x<=2; x++)
    {
       let g: vec2<f32> = vec2<f32>( f32(x), f32(y) );
        let o: vec3<f32> = hash_three( i + g ) * vec3(u,u,1.0);
        let d: vec2<f32> = g - f + o.xy;
        let w: f32 = pow( 1.0-smoothstep(0.0,1.414,length(d)), k );
        a += vec2(o.z*w,w);
    }
    }
    
    return a.x/a.y;
}