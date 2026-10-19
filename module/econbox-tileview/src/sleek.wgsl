struct Pass {
    texel: vec4<f32>,
    threshold: vec4<f32>,
    bloom: vec4<f32>,
    shape: vec4<f32>,
    color: vec4<f32>,
};

struct FragmentInput {
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var smooth_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: Pass;

@group(0) @binding(3) var scene: texture_2d<f32>;

fn encode(color: vec3<f32>) -> vec3<f32> {
    let low = color * 12.92;
    let high = 1.055 * pow(max(color, vec3(0.0)), vec3(1.0 / 2.4)) - vec3(0.055);
    return select(high, low, color <= vec3(0.0031308));
}

fn decode(color: vec3<f32>) -> vec3<f32> {
    let low = color / 12.92;
    let high = pow((max(color, vec3(0.0)) + vec3(0.055)) / 1.055, vec3(2.4));
    return select(high, low, color <= vec3(0.04045));
}

fn bright(uv: vec2<f32>) -> vec4<f32> {
    let sample = textureSample(source, smooth_sampler, uv);
    let color = vec4(encode(sample.rgb), sample.a);
    let weight = clamp(dot(vec4(color.rgb, 1.0), settings.threshold), 0.0, 1.0);
    return color * weight;
}

fn plain(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(source, smooth_sampler, uv);
}

@fragment
fn brightpass(in: FragmentInput) -> @location(0) vec4<f32> {
    let t = settings.texel.xy;
    var sum = bright(in.uv + vec2(-2.0 * t.x, 0.0));
    sum += bright(in.uv + vec2(-t.x, t.y)) * 2.0;
    sum += bright(in.uv + vec2(0.0, 2.0 * t.y));
    sum += bright(in.uv + vec2(t.x, t.y)) * 2.0;
    sum += bright(in.uv + vec2(2.0 * t.x, 0.0));
    sum += bright(in.uv + vec2(t.x, -t.y)) * 2.0;
    sum += bright(in.uv + vec2(0.0, -2.0 * t.y));
    sum += bright(in.uv + vec2(-t.x, -t.y)) * 2.0;
    return sum * 0.0833333358;
}

@fragment
fn downsample(in: FragmentInput) -> @location(0) vec4<f32> {
    let t = settings.texel.xy;
    var sum = plain(in.uv + vec2(-2.0 * t.x, 0.0));
    sum += plain(in.uv + vec2(-t.x, t.y)) * 2.0;
    sum += plain(in.uv + vec2(0.0, 2.0 * t.y));
    sum += plain(in.uv + vec2(t.x, t.y)) * 2.0;
    sum += plain(in.uv + vec2(2.0 * t.x, 0.0));
    sum += plain(in.uv + vec2(t.x, -t.y)) * 2.0;
    sum += plain(in.uv + vec2(0.0, -2.0 * t.y));
    sum += plain(in.uv + vec2(-t.x, -t.y)) * 2.0;
    return sum * 0.0833333358;
}

@fragment
fn precompose(in: FragmentInput) -> @location(0) vec4<f32> {
    let bloom = textureSample(source, smooth_sampler, in.uv).rgb;
    let centre = in.uv - vec2(0.5);
    var fade = clamp(dot(centre, centre) * settings.shape.x + settings.shape.y, 0.0, 1.0);
    fade = fade * settings.shape.w;
    let keep = 1.0 - settings.color.w * fade;
    let dark = fade * settings.color.rgb;
    let glow = settings.shape.z;
    let lit = 1.0 - glow * dot(bloom, vec3(0.212599993, 0.715200007, 0.0722000003));
    let added = glow * bloom * settings.bloom.w * settings.bloom.rgb;
    return vec4(keep * added + dark, keep * lit);
}

@fragment
fn compose(in: FragmentInput) -> @location(0) vec4<f32> {
    let pre = textureSample(source, smooth_sampler, in.uv);
    let color = encode(textureSample(scene, smooth_sampler, in.uv).rgb);
    return vec4(decode(color * pre.a + pre.rgb), 1.0);
}
