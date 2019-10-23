#ifndef LTC_FRAG
#define LTC_FRAG

// Get uv coordinates into LTC lookup texture
//
// N: Normal
// V: View direction
// roughness
vec2 ltc1_coords(float NdotV, float roughness) {
    vec2 coords = vec2(roughness, sqrt(1.0 - saturate(NdotV)));

    /* Scale and bias coordinates, for correct filtered lookup */
    coords = coords*LUT_SCALE + LUT_BIAS;

    return coords;
}
// Get uv coords into LTC lookup tabled of precomputed clipped form factors
//
// dir: F direction
// len: form factor of the polygon
vec2 ltc2_coords(float dir, float len) {
    vec2 coords = vec2(dir*0.5 + 0.5, len);

    /* Scale and bias coordinates, for correct filtered lookup */
    coords = coords*LUT_SCALE + LUT_BIAS;

    return coords;
}

/** Get inverse matrix from LTC lookup texture */
mat3 ltc_matrix(sampler2D tex, vec2 coord) {
    const vec4 t = texture(tex, coord);
    mat3 Minv = mat3(
        vec3(  t.x,   0, t.y),
        vec3(  0, 1,   0),
        vec3(t.z,   0, t.w)
    );

    return Minv;
}
// An extended version of the implementation from
// "How to solve a cubic equation, revisited"
// http://momentsingraphics.de/?p=105
vec3 solve_cubic(vec4 Coefficient)
{
    // Normalize the polynomial
    Coefficient.xyz /= Coefficient.w;
    // Divide middle coefficients by three
    Coefficient.yz /= 3.0;

    float A = Coefficient.w;
    float B = Coefficient.z;
    float C = Coefficient.y;
    float D = Coefficient.x;

    // Compute the Hessian and the discriminant
    vec3 Delta = vec3(
        -Coefficient.z*Coefficient.z + Coefficient.y,
        -Coefficient.y*Coefficient.z + Coefficient.x,
        dot(vec2(Coefficient.z, -Coefficient.y), Coefficient.xy)
    );
    // In contrast to the reference implementation we add 0.0001 
    // to the discriminant to avoid some unstable results.
    float Discriminant = dot(vec2(4.0*Delta.x, -Delta.y), Delta.zy)+0.0001;

    vec3 RootsA, RootsD;

    vec2 xlc, xsc;

    // Algorithm A
    {
        float A_a = 1.0;
        float C_a = Delta.x;
        float D_a = -2.0*B*Delta.x + Delta.y;

        // Take the cubic root of a normalized complex number
        float Theta = atan(sqrt(Discriminant), -D_a)/3.0;

        float x_1a = 2.0*sqrt(-C_a)*cos(Theta);
        float x_3a = 2.0*sqrt(-C_a)*cos(Theta + (2.0/3.0)*PI);

        float xl;
        if ((x_1a + x_3a) > 2.0*B)
            xl = x_1a;
        else
            xl = x_3a;

        xlc = vec2(xl - B, A);
    }

    // Algorithm D
    {
        float A_d = D;
        float C_d = Delta.z;
        float D_d = -D*Delta.y + 2.0*C*Delta.z;

        // Take the cubic root of a normalized complex number
        float Theta = atan(D*sqrt(Discriminant), -D_d)/3.0;

        float x_1d = 2.0*sqrt(-C_d)*cos(Theta);
        float x_3d = 2.0*sqrt(-C_d)*cos(Theta + (2.0/3.0)*PI);

        float xs;
        if (x_1d + x_3d < 2.0*C)
            xs = x_1d;
        else
            xs = x_3d;

        xsc = vec2(-D, xs + C);
    }

    float E =  xlc.y*xsc.y;
    float F = -xlc.x*xsc.y - xlc.y*xsc.x;
    float G =  xlc.x*xsc.x;

    vec2 xmc = vec2(C*F - B*G, -B*F + C*E);

    vec3 Root = vec3(xsc.x/xsc.y, xmc.x/xmc.y, xlc.x/xlc.y);

    if (Root.x < Root.y && Root.x < Root.z)
        Root.xyz = Root.yxz;
    else if (Root.z < Root.x && Root.z < Root.y)
        Root.xyz = Root.xzy;

    return Root;
}


/*
 * Get intensity of light from the arealight given by `points` at the point `P`
 * with normal `N` when viewed from direction `P`.
 * @param N Normal
 * @param V View Direction
 * @param P Vertex Position
 * @param Minv Matrix to transform from BRDF distribution to clamped cosine distribution
 * @param points Light quad vertices
 * @param twoSided Whether the light is two sided
 */
vec3 ltc_evaluate_ellipse(
    vec3 N, 
    vec3 V, 
    vec3 P, 
    float NdotV,
    mat3 Minv, 
    vec3 points[4], 
    bool two_sided,
    bool sphere
)
{


    if (sphere) {
        // Get center of our quad.
        vec3 Cr;
        Cr = (points[0] + points[2])/2.0;
        
        vec3 Cv0 = points[0] - Cr;
        vec3 Cv1 = points[1] - Cr;
        vec3 Cv2 = points[2] - Cr;
        
        vec3 Cn = normalize(cross(points[0]-points[2], points[1]- points[3]));
        vec3 Cp = normalize(P - Cr);
        if (Cn != Cp) {
            float costheta = dot(-Cn, Cp);
            vec3 rotAxis = cross(Cn, Cp);
            mat3 Z = rotationMatrix3(rotAxis, costheta);
            // rotate points around center to change the plane to be perpendicular to the vector from center to fragment.
            points[0] = Cr + (Z * Cv0);
            points[1] = Cr + (Z * Cv1);
            points[2] = Cr + (Z * Cv2);
        }
    }

    // construct orthonormal basis around N
    vec3 T1, T2;
    T1 = normalize(V - N*NdotV);
    T2 = cross(N, T1);

    // rotate area light in (T1, T2, N) basis
    mat3 R = transpose(mat3(T1, T2, N));

    // polygon (allocate 5 vertices for clipping)
    vec3 L_[3];
    
    L_[0] = R * (points[0] - P);
    L_[1] = R * (points[1] - P);
    L_[2] = R * (points[2] - P);


    // init ellipse
    vec3 C  = 0.5 * (L_[0] + L_[2]);
    vec3 V1 = 0.5 * (L_[1] - L_[2]);
    vec3 V2 = 0.5 * (L_[1] - L_[0]);

    C  = Minv * C;
    V1 = Minv * V1;
    V2 = Minv * V2;

    if(!two_sided && dot(cross(V1, V2), C) < 0.0)
        return vec3(0.0);

    // compute eigenvectors of ellipse
    float a, b;
    float d11 = dot(V1, V1);
    float d22 = dot(V2, V2);
    float d12 = dot(V1, V2);
    if (abs(d12)/sqrt(d11*d22) > 0.0001)
    {
        float tr = d11 + d22;
        float det = -d12*d12 + d11*d22;

        // use sqrt matrix to solve for eigenvalues
        det = sqrt(det);
        float u = 0.5*sqrt(tr - 2.0*det);
        float v = 0.5*sqrt(tr + 2.0*det);
        float e_max = sqr(u + v);
        float e_min = sqr(u - v);

        vec3 V1_, V2_;

        if (d11 > d22)
        {
            V1_ = d12*V1 + (e_max - d11)*V2;
            V2_ = d12*V1 + (e_min - d11)*V2;
        }
        else
        {
            V1_ = d12*V2 + (e_max - d22)*V1;
            V2_ = d12*V2 + (e_min - d22)*V1;
        }
        // Add error to avoud division by zero in solve_cubic
        a = 1.001 / e_max;
        b = 1.0 / e_min;
        V1 = normalize(V1_);
        V2 = normalize(V2_);
    }
    else
    {
        a = 1.0 / dot(V1, V1);
        b = 1.0 / dot(V2, V2);
        V1 *= sqrt(a);
        V2 *= sqrt(b);
    }

    vec3 V3 = cross(V1, V2);
    if (dot(C, V3) < 0.0)
        V3 *= -1.0;

    float L  = dot(V3, C);
    float x0 = dot(V1, C) / L;
    float y0 = dot(V2, C) / L;

    float E1 = inversesqrt(a);
    float E2 = inversesqrt(b);

    a *= L*L;
    b *= L*L;

    float c0 = a*b;
    float c1 = a*b*(1.0 + x0*x0 + y0*y0) - a - b;
    float c2 = 1.0 - a*(1.0 + x0*x0) - b*(1.0 + y0*y0);
    float c3 = 1.0;

    vec3 roots = solve_cubic(vec4(c0, c1, c2, c3));
    float e1 = roots.x;
    float e2 = roots.y;
    float e3 = roots.z;

    vec3 avgDir = vec3(a*x0/(a - e2), b*y0/(b - e2), 1.0);

    mat3 rotate = mat3(V1, V2, V3);

    avgDir = rotate*avgDir;
    avgDir = normalize(avgDir);

    float L1 = sqrt(-e2/e3);
    float L2 = sqrt(-e2/e1);

    float form_actor = L1*L2*inversesqrt((1.0 + L1*L1)*(1.0 + L2*L2));

    // use tabulated horizon-clipped sphere
    vec2 uv = ltc2_coords(avgDir.z, form_actor);
    float scale = texture(ltc_2, uv).w;

    float spec = form_actor*scale;

    return saturate3(vec3(spec));
}
vec3 EdgeIntegral(vec3 v1, vec3 v2)
{
    float x = dot(v1, v2);
    float y = abs(x);

    float a = 0.8543985 + (0.4965155 + 0.0145206*y)*y;
    float b = 3.4175940 + (4.1616724 + y)*y;
    float v = a / b;

    float theta_sintheta = (x > 0.0) ? v : 0.5*inversesqrt(max(1.0 - x*x, 1e-7)) - v;

    return cross(v1, v2)*theta_sintheta;
}


// N: Vertex Normal
// V: View Direction
// P: Vertex Position
// Minv: Inversed Matrix
vec3 ltc_evaluate_rect(
    vec3 N, 
    vec3 V, 
    vec3 P,
    float NdotV,
    mat3 Minv, 
    vec3 points[4], 
    bool two_sided
)
{
    // construct orthonormal basis around N
    vec3 T1, T2;
    T1 = normalize(V - N*NdotV);
    T2 = cross(N, T1);

    // rotate area light in (T1, T2, N) basis
    Minv = Minv * transpose(mat3(T1, T2, N));

    // polygon (allocate 5 vertices for clipping)
    vec3 L[4];
    L[0] = Minv * (points[0] - P);
    L[1] = Minv * (points[1] - P);
    L[2] = Minv * (points[2] - P);
    L[3] = Minv * (points[3] - P);

    float sum = 0.0;

    vec3 dir = points[0].xyz - P;
    vec3 lightNormal = cross(points[1] - points[0], points[3] - points[0]);
    bool behind = (dot(dir, lightNormal) < 0.0);

    L[0] = normalize(L[0]);
    L[1] = normalize(L[1]);
    L[2] = normalize(L[2]);
    L[3] = normalize(L[3]);

    vec3 vsum = vec3(0.0);
    vsum += EdgeIntegral(L[0], L[1]);
    vsum += EdgeIntegral(L[1], L[2]);
    vsum += EdgeIntegral(L[2], L[3]);
    vsum += EdgeIntegral(L[3], L[0]);

    float len = length(vsum);
    float avgDir = vsum.z/len;

    if (behind)
        avgDir = -avgDir;

    vec2 uv = ltc2_coords(avgDir, len);

    float scale = texture(ltc_2, uv).w;

    sum = len*scale;

    if (behind && !two_sided)
        sum = 0.0;

    return saturate3(vec3(sum));
}

// float Fpo(float d, float l)
// {
//     return l/(d*(d*d + l*l)) + atan(l/d)/(d*d);
// }

// float Fwt(float d, float l)
// {
//     return l*l/(d*(d*d + l*l));
// }

// float integrate_diffuse_line(vec3 p1, vec3 p2)
// {
//     // tangent
//     vec3 wt = normalize(p2 - p1);

//     // clamping
//     if (p1.z <= 0.0 && p2.z <= 0.0) return 0.0;
//     if (p1.z < 0.0) p1 = (+p1*p2.z - p2*p1.z) / (+p2.z - p1.z);
//     if (p2.z < 0.0) p2 = (-p1*p2.z + p2*p1.z) / (-p2.z + p1.z);

//     // parameterization
//     float l1 = dot(p1, wt);
//     float l2 = dot(p2, wt);

//     // shading point orthonormal projection on the line
//     vec3 po = p1 - l1*wt;

//     // distance to line
//     float d = length(po);

//     // integral
//     float I = (Fpo(d, l2) - Fpo(d, l1)) * po.z +
//               (Fwt(d, l2) - Fwt(d, l1)) * wt.z;
//     return I / PI;
// }


// float integrate_ltc_line(vec3 p1, vec3 p2)
// {
//     // transform to diffuse configuration
//     vec3 p1o = Minv * p1;
//     vec3 p2o = Minv * p2;
//     float area = I_diffuse_line(p1o, p2o);

//     // width factor
//     vec3 ortho = normalize(cross(p1, p2));
//     float w =  1.0 / length(inverse(transpose(Minv)) * ortho);

//     return w * area;
// }


// float integrate_ltc_disks(vec3 p1, vec3 p2, float radius)
// {
//     float A = PI * R * R;
//     vec3 wt  = normalize(p2 - p1);
//     vec3 wp1 = normalize(p1);
//     vec3 wp2 = normalize(p2);
//     float area = A * (
//     D(wp1) * max(0.0, dot(+wt, wp1)) / dot(p1, p1) +
//     D(wp2) * max(0.0, dot(-wt, wp2)) / dot(p2, p2));
//     return area;
// }


// vec3 ltc_evaluate_line(
//     vec3 N, 
//     vec3 V, 
//     vec3 P,
//     float NdotV,
//     mat3 Minv,
//     vec3 points[2],
//     float radius,
//     bool end_caps)
// {
//     // construct orthonormal basis around N
//     vec3 T1, T2;
//     T1 = normalize(V - N*NdotV);
//     T2 = cross(N, T1);

//     mat3 R = transpose(mat3(T1, T2, N));

//     vec3 p1 = R * (points[0] - P);
//     vec3 p2 = R * (points[1] - P);

//     float Iline = radius * I_ltc_line(p1, p2);
//     float Idisks = end_caps ? I_ltc_disks(p1, p2, radius) : 0.0;
//     return vec3(min(1.0, Iline + Idisks));
// }


vec3 compute_round_area_light(float roughness, vec3 normal, vec3 view_direction, float NdotV, vec3 albedo) {
    vec3 color = vec3(0);
    for (int i = 0; i < round_area_light_count; i++) {
        vec3 diffuse_color = ellipse_area_light[i].diffuse_color;
        vec3 spec_color = ellipse_area_light[i].spec_color;
        
        vec2 uv = ltc1_coords(NdotV, roughness);
        
        mat3 Minv = ltc_matrix(ltc_1, uv);
        
        vec4 t2 = texture(ltc_2, uv);

        vec3 spec = ltc_evaluate_ellipse(normal, view_direction, vertex.position, NdotV, Minv, ellipse_area_light[i].quad_points, ellipse_area_light[i].two_sided, ellipse_area_light[i].sphere);

        // // BRDF shadowing and Fresnel
        spec *= spec_color*t2.r + (1.0 - spec_color)*t2.g;

        vec3 diff = ltc_evaluate_ellipse(normal, view_direction, vertex.position, NdotV, mat3(1), ellipse_area_light[i].quad_points, ellipse_area_light[i].two_sided, ellipse_area_light[i].sphere);
        color += ellipse_area_light[i].intensity*(spec + diffuse_color*albedo*diff);
    }
    return color;
}

vec3 compute_rect_area_light(float roughness, vec3 normal, vec3 view_direction, float NdotV, vec3 albedo) {    
    vec3 color = vec3(0);
    for (int i = 0; i < rect_area_light_count; i++) {

        vec3 diffuse_color = rect_area_light[i].diffuse_color;
        vec3 spec_color = rect_area_light[i].spec_color;

        vec2 uv = ltc1_coords(NdotV, roughness);
        
        mat3 Minv = ltc_matrix(ltc_1, uv);
        
        vec4 t2 = texture(ltc_2, uv);

        vec3 spec = ltc_evaluate_rect(normal, view_direction, vertex.position, NdotV, Minv, rect_area_light[i].quad_points, rect_area_light[i].two_sided);

        // // BRDF shadowing and Fresnel
        spec *= spec_color*t2.r + (1.0 - spec_color)*t2.g;

        vec3 diff = ltc_evaluate_rect(normal, view_direction, vertex.position, NdotV, mat3(1), rect_area_light[i].quad_points, rect_area_light[i].two_sided);
        color += rect_area_light[i].intensity*(spec + diffuse_color*albedo*diff);
    }
    return color;
}


// vec3 compute_line_area_light(float roughness, vec3 normal, vec3 view_direction, float NdotV, vec3 albedo) {    
//     vec3 color = vec3(0);
//     for (int i = 0; i < line_area_light_count; i++) {

//         vec3 diffuse_color = line_area_light[i].diffuse_color;
//         vec3 spec_color = line_area_light[i].spec_color;
        
//         vec2 uv = ltc1_coords(NdotV, roughness);
        
//         mat3 Minv = ltc_matrix(ltc_1, uv);
        
//         vec4 t2 = texture(ltc_2, uv);

//         vec3 spec = ltc_evaluate_rect(normal, view_direction, vertex.position, NdotV, Minv, );

//         // // BRDF shadowing and Fresnel
//         spec *= spec_color*t2.r + (1.0 - spec_color)*t2.g;

//         vec3 diff = ltc_evaluate_rect(normal, view_direction, vertex.position, NdotV, mat3(1), line_area_light[i].end_caps);
//         color += line_area_light[i].intensity*(spec + diffuse_color*albedo*diff) / (2.0*PI);
//     }
//     return color;
// }


#endif