#ifndef 
SKINNING_LOC_VERTEX
    #error "not defined"
#endif

#ifndef SKINNING_LOC_INSTANCE
    #define SKINNING_LOC_INSTANCE SKINNING_LOC_VERTEX + 1
#endif

layout(location = SKINNING_LOC_INSTANCE) in uint joints_offset; // instance rate
