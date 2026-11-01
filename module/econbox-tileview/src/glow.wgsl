struct FragmentInput {
    @location(2) uv: vec2<f32>,
};

@group(2) @binding(0) var marks: texture_2d<f32>;
@group(2) @binding(1) var marks_sampler: sampler;
@group(2) @binding(2) var minimap: texture_2d<f32>;
@group(2) @binding(3) var minimap_sampler: sampler;
@group(2) @binding(4) var<uniform> origin: vec4<f32>;
@group(2) @binding(5) var<uniform> tint: vec4<f32>;

const FIRE: vec3<f32> = vec3(1.0, 105.0 / 255.0, 48.0 / 255.0);

fn to_linear(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((color + vec3(0.055)) / 1.055, vec3(2.4));
    return select(high, low, color <= vec3(0.04045));
}

fn to_gamma(color: vec3<f32>) -> vec3<f32> {
    let low = color * 12.92;
    let high = 1.055 * pow(color, vec3(1.0 / 2.4)) - vec3(0.055);
    return select(high, low, color <= vec3(0.0031308));
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let size = textureDimensions(marks);
    let at = min(vec2<u32>(in.uv * vec2<f32>(size)), size - vec2(1u));
    let mark = textureLoad(marks, at, 0);
    if all(mark.rgb == vec3(0.0)) {
        discard;
    }
    let map_size = textureDimensions(minimap);
    let tile = min(vec2<u32>(origin.xy) + at, map_size - vec2(1u));
    let base = to_gamma(textureLoad(minimap, tile, 0).rgb * tint.rgb);
    var color = base * (1.0 - mark.r);
    color = color + (vec3(1.0) - color) * mark.g;
    color = min(color + FIRE * mark.b, vec3(1.0));
    return vec4(to_linear(color), tint.a);
}
