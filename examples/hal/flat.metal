using namespace metal;

struct VsInput {
  float3 position [[attribute(0)]];
  float4 color [[attribute(1)]];
};

struct VsOutput {
    float4 position [[position]];
    float4 color;
};

vertex VsOutput vs_main(VsInput in [[stage_in]]) {
    VsOutput out;
    out.position = float4(in.position, 1.0);
    out.color = in.color;
    return out;
}

fragment float4 ps_main(VsOutput in [[stage_in]]) {
    return in.color;
}