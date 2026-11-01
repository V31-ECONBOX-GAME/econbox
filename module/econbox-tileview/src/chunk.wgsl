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
@group(2) @binding(7) var<uniform> tint: vec4<f32>;
@group(2) @binding(8) var burns: texture_2d<f32>;
@group(2) @binding(9) var burns_sampler: sampler;
@group(2) @binding(10) var<uniform> margin: vec4<f32>;

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
    let night = select(tint.rgb, vec3(1.0), marks.b > 0.5);
    var color = mix(ocean.rgb, base.rgb * night, base.a);
    let cover = 1.0 - pow(1.0 - rim.a, count);
    color = mix(color, rim.rgb, cover);
    color = color * (1.0 - shadow.x * step(0.5, marks.g));
    let spot = vec2<i32>(at) - vec2<i32>(margin.xy);
    let cells = vec2<i32>(textureDimensions(burns));
    if all(spot >= vec2(0)) {
        let cell = spot / 4;
        if all(cell < cells) {
            let burn = textureLoad(burns, vec2<u32>(cell), 0);
            color = color * (1.0 - burn.r);
            color = color + (vec3(1.0) - color) * burn.g;
        }
    }
    return vec4(to_linear(color), 1.0);
}
