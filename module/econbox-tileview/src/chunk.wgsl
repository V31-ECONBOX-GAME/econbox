struct FragmentInput {
    @location(2) uv: vec2<f32>,
};

@group(2) @binding(0) var<uniform> rim: vec4<f32>;
@group(2) @binding(1) var<uniform> ocean: vec4<f32>;
@group(2) @binding(2) var tiles: texture_2d<f32>;
@group(2) @binding(3) var tiles_sampler: sampler;
@group(2) @binding(4) var runup: texture_2d<f32>;
@group(2) @binding(5) var runup_sampler: sampler;
@group(2) @binding(6) var<uniform> shadow: vec4<f32>;

fn to_linear(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((color + vec3(0.055)) / 1.055, vec3(2.4));
    return select(high, low, color <= vec3(0.04045));
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let size = textureDimensions(tiles);
    let at = min(vec2<u32>(in.uv * vec2<f32>(size)), size - vec2(1u));
    let base = textureLoad(tiles, at, 0);
    let marks = textureLoad(runup, at, 0);
    let count = round(marks.r * 255.0);
    var color = mix(ocean.rgb, base.rgb, base.a);
    let cover = 1.0 - pow(1.0 - rim.a, count);
    color = mix(color, rim.rgb, cover);
    color = color * (1.0 - shadow.x * step(0.5, marks.g));
    return vec4(to_linear(color), 1.0);
}
