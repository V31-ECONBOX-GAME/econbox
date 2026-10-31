struct Ambience {
    light: vec4<f32>,
    overlays: array<vec4<f32>, 9>,
    kinds: array<vec4<f32>, 9>,
};

struct FragmentInput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct BlobOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
};

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var light: texture_2d<f32>;
@group(0) @binding(2) var<uniform> ambience: Ambience;

const PEAK: f32 = 0.86;

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

fn store(color: vec3<f32>) -> vec3<f32> {
    return round(clamp(color, vec3(0.0), vec3(1.0)) * 255.0) / 255.0;
}

@vertex
fn blob_vertex(@builtin(vertex_index) index: u32, @location(0) quad: vec4<f32>) -> BlobOutput {
    let corner = vec2(f32(index & 1u), f32((index >> 1u) & 1u)) * 2.0 - vec2(1.0);
    var out: BlobOutput;
    out.position = vec4(quad.xy + corner * quad.zw, 0.0, 1.0);
    out.local = corner;
    return out;
}

@fragment
fn blob_fragment(in: BlobOutput) -> @location(0) vec4<f32> {
    let reach = length(in.local);
    let alpha = select(0.0, PEAK * pow(1.0 - reach, 1.5), reach < 1.0);
    return vec4(alpha, alpha, alpha, alpha);
}

@fragment
fn compose(in: FragmentInput) -> @location(0) vec4<f32> {
    let size = textureDimensions(scene);
    let at = min(vec2<u32>(in.position.xy), size - vec2(1u));
    var color = store(encode(textureLoad(scene, at, 0).rgb));

    if ambience.light.a > 0.0 {
        let lights = textureDimensions(light);
        let cell = min(vec2<u32>(in.uv * vec2<f32>(lights)), lights - vec2(1u));
        let glow = textureLoad(light, cell, 0);
        let tinted = glow.rgb * ambience.light.rgb;
        let alpha = glow.a * ambience.light.a;
        let source = clamp(vec3(0.5) + alpha * (3.0 * tinted - vec3(0.5)), vec3(0.0), vec3(1.0));
        color = store(2.0 * source * color);
    }

    for (var i = 0u; i < 9u; i++) {
        let overlay = ambience.overlays[i];
        let alpha = overlay.a;
        if alpha <= 0.0 {
            continue;
        }
        let kind = ambience.kinds[i].x;
        if kind < 0.5 {
            color = store(color * mix(vec3(1.0), overlay.rgb, alpha));
        } else if kind < 1.5 {
            color = store(color + overlay.rgb * alpha);
        } else {
            color = store(mix(color, overlay.rgb, alpha));
        }
    }

    return vec4(decode(color), 1.0);
}
