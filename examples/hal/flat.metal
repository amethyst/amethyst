using namespace metal;

// set of three orthogonal matrices
struct TrProjView {
    float4x4 transform;
    // float4x4 view;
    // float4x4 projection;
};

struct VsInput {
  float3 position [[attribute(0)]];
  float4 color [[attribute(1)]];
};

struct VsOutput {
    float4 position [[position]];
    float4 color;
};

vertex VsOutput vs_main(VsInput in [[stage_in]], constant TrProjView& tr_proj_view [[buffer(0)]]) {
    VsOutput out;
    out.position = float4(in.position, 1.0) * tr_proj_view.transform;
    out.color = in.color;
    return out;
}

fragment float4 ps_main(VsOutput in [[stage_in]]) {
    return in.color;
}
