struct FragmentInput {
    @location(2) uv: vec2<f32>,
};

@group(2) @binding(0) var marks: texture_2d<f32>;
@group(2) @binding(1) var marks_sampler: sampler;

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    let size = textureDimensions(marks);
    let at = min(vec2<u32>(in.uv * vec2<f32>(size)), size - vec2(1u));
    return textureLoad(marks, at, 0);
}
